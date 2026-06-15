use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::error::{AppError, AppResult};
use crate::utils::crypto;
use crate::utils::path_security;
use crate::utils::path_security::sanitize_filename;
use log::info;

pub struct BackupService {
    backup_dir: PathBuf,
    #[allow(dead_code)]
    settings_service: std::sync::Arc<crate::services::settings_service::SettingsService>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: Option<String>,
}

impl BackupService {
    pub fn new(
        backup_dir: PathBuf,
        settings_service: std::sync::Arc<crate::services::settings_service::SettingsService>,
    ) -> Self {
        std::fs::create_dir_all(&backup_dir).ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(()) = std::fs::set_permissions(&backup_dir, PermissionsExt::from_mode(0o700)) {
            }
        }
        Self {
            backup_dir,
            settings_service,
        }
    }

    pub fn create_backup(
        &self,
        projects: &[serde_json::Value],
        categories: &[serde_json::Value],
        passphrase: &str,
        output_path: Option<&Path>,
    ) -> AppResult<PathBuf> {
        let payload = crypto::BackupPayload {
            projects: projects.to_vec(),
            categories: categories.to_vec(),
            settings: serde_json::json!({}),
        };
        let encrypted_json = crypto::encrypt_to_json(
            payload,
            passphrase,
            Some(crypto::BackupAppMeta {
                name: "SelfHost Helper".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        )?;
        let zip_path = output_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let name = sanitize_filename(&format!("backup_{}", timestamp));
                self.backup_dir.join(format!("{}.zip", name))
            });
        let zip_file = File::create(&zip_path).map_err(AppError::Io)?;
        let mut zip = ZipWriter::new(zip_file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("backup.json", options)?;
        zip.write_all(encrypted_json.as_bytes())
            .map_err(AppError::Io)?;
        zip.finish()?;
        info!("backup:create path={}", zip_path.display());
        Ok(zip_path)
    }

    pub fn validate_backup_path(&self, path: &Path) -> AppResult<PathBuf> {
        let resolved = std::fs::canonicalize(path).map_err(AppError::Io)?;
        if !resolved.exists() {
            return Err(AppError::PathSecurity(format!(
                "backup path does not exist: {}",
                path.display()
            )));
        }
        path_security::validate_inside_roots(&resolved, std::slice::from_ref(&self.backup_dir))?;
        Ok(resolved)
    }

    pub fn restore_backup(
        &self,
        zip_path: &Path,
        passphrase: &str,
    ) -> AppResult<crypto::BackupPayload> {
        let zip_file = File::open(zip_path).map_err(AppError::Io)?;
        let mut archive =
            ZipArchive::new(zip_file).map_err(|e| AppError::Internal(e.to_string()))?;
        let mut backup_json = String::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if file.name() == "backup.json" {
                file.read_to_string(&mut backup_json)
                    .map_err(AppError::Io)?;
                break;
            }
        }
        if backup_json.is_empty() {
            return Err(AppError::Validation(
                "Invalid backup: no backup.json found".into(),
            ));
        }
        let envelope = crypto::decrypt_from_json(&backup_json, passphrase)?;
        info!("backup:restore path={}", zip_path.display());
        Ok(envelope.payload)
    }

    pub fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        if self.backup_dir.exists() {
            for entry in
                std::fs::read_dir(&self.backup_dir).map_err(AppError::Io)?
            {
                let entry = entry.map_err(AppError::Io)?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                    let metadata = entry.metadata().map_err(AppError::Io)?;
                    backups.push(BackupInfo {
                        filename: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len(),
                        created_at: metadata.modified().ok().and_then(|t| {
                            let duration = t
                                .duration_since(std::time::UNIX_EPOCH)
                                .ok()?;
                            Some(
                                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)?
                                    .to_rfc3339(),
                            )
                        }),
                    });
                }
            }
        }
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub fn delete_backup(&self, filename: &str) -> AppResult<()> {
        validate_backup_filename(filename)?;

        let path = self.backup_dir.join(filename);
        // Canonicalize both paths and verify the target is within backup_dir
        let canonical_target =
            std::fs::canonicalize(&path).map_err(AppError::Io)?;
        let canonical_backup_dir =
            std::fs::canonicalize(&self.backup_dir).map_err(AppError::Io)?;
        if !crate::utils::path_security::is_within_base(
            &canonical_target,
            &canonical_backup_dir,
        ) {
            return Err(AppError::Validation(
                "Backup path is outside backup directory".into(),
            ));
        }

        if path.exists() {
            std::fs::remove_file(&path).map_err(AppError::Io)?;
        }
        info!("backup:delete path={}", path.display());
        Ok(())
    }
}

/// Validates a backup filename to prevent path traversal attacks.
/// Rejects path separators, `..` components, null bytes, and empty strings.
fn validate_backup_filename(filename: &str) -> AppResult<()> {
    if filename.is_empty() {
        return Err(AppError::Validation(
            "Backup filename cannot be empty".into(),
        ));
    }
    if filename.contains('/') || filename.contains('\\') {
        return Err(AppError::Validation(
            "Backup filename contains path separators".into(),
        ));
    }
    if filename.contains("..") {
        return Err(AppError::Validation(
            "Backup filename contains invalid characters".into(),
        ));
    }
    if filename.contains('\0') {
        return Err(AppError::Validation(
            "Backup filename contains invalid characters".into(),
        ));
    }
    Ok(())
}
