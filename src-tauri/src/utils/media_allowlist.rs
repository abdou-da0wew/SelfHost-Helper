use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

const EXTERNAL_MEDIA_DIRS_ENV_KEY: &str = "SELFHOST_MEDIA_ALLOWED_DIRS";

#[derive(Debug, Clone)]
pub struct MediaAllowlist {
    configured_bases: Vec<PathBuf>,
    session_bases: Arc<RwLock<HashSet<PathBuf>>>,
    project_icon_bases: Arc<RwLock<Vec<PathBuf>>>,
    has_logged_missing: Arc<RwLock<bool>>,
}

#[derive(Debug, thiserror::Error)]
pub enum MediaAllowlistError {
    #[error("non-absolute path rejected")]
    NonAbsolute,
    #[error("path is outside all allowed directories")]
    PathOutsideAllowlist,
    #[error("no external media allowlist configured")]
    NoAllowlistConfigured,
    #[error("file not found")]
    FileNotFound,
}

impl MediaAllowlist {
    pub fn new() -> Self {
        let configured_bases = Self::parse_env_dirs();
        Self {
            configured_bases,
            session_bases: Arc::new(RwLock::new(HashSet::new())),
            project_icon_bases: Arc::new(RwLock::new(Vec::new())),
            has_logged_missing: Arc::new(RwLock::new(false)),
        }
    }

    fn parse_env_dirs() -> Vec<PathBuf> {
        let raw = std::env::var(EXTERNAL_MEDIA_DIRS_ENV_KEY).unwrap_or_default();
        let delimiter = if cfg!(target_os = "windows") { ';' } else { ':' };
        raw.split(delimiter)
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| {
                let p = PathBuf::from(e);
                dunce::canonicalize(&p).unwrap_or(p)
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn add_session_path(&self, file_path: &Path) {
        if let Some(parent) = file_path.parent() {
            let resolved = dunce::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            self.session_bases
                .write()
                .expect("lock poisoned")
                .insert(resolved);
        }
    }

    #[allow(dead_code)]
    pub fn set_project_icon_bases(&self, bases: Vec<PathBuf>) {
        *self
            .project_icon_bases
            .write()
            .expect("lock poisoned") = bases;
    }

    fn all_bases(&self) -> Vec<PathBuf> {
        let mut bases = self.configured_bases.clone();
        bases.extend(
            self.session_bases
                .read()
                .expect("lock poisoned")
                .iter()
                .cloned(),
        );
        bases.extend(
            self.project_icon_bases
                .read()
                .expect("lock poisoned")
                .iter()
                .cloned(),
        );
        bases
    }

    #[allow(dead_code)]
    pub fn is_allowed(&self, file_path: &Path) -> bool {
        let bases = self.all_bases();
        crate::utils::path_security::is_within_any_base(file_path, &bases)
    }

    pub fn validate_media_path(
        &self,
        file_path: &Path,
    ) -> Result<PathBuf, MediaAllowlistError> {
        if !file_path.is_absolute() {
            return Err(MediaAllowlistError::NonAbsolute);
        }
        let bases = self.all_bases();
        if bases.is_empty() {
            let mut logged = self.has_logged_missing.write().expect("lock poisoned");
            if !*logged {
                *logged = true;
                log::warn!(
                    "External media requests require an allowlist. Set {}",
                    EXTERNAL_MEDIA_DIRS_ENV_KEY
                );
            }
            return Err(MediaAllowlistError::NoAllowlistConfigured);
        }
        let resolved = crate::utils::path_security::resolve_and_validate(file_path, &bases)
            .ok_or(MediaAllowlistError::PathOutsideAllowlist)?;
        if !resolved.exists() {
            return Err(MediaAllowlistError::FileNotFound);
        }
        Ok(resolved)
    }
}

impl Default for MediaAllowlist {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub type SharedMediaAllowlist = Arc<MediaAllowlist>;

#[allow(dead_code)]
pub fn mime_type_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".svg" => "image/svg+xml",
        ".mp4" => "video/mp4",
        ".mp3" => "audio/mpeg",
        ".pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}
