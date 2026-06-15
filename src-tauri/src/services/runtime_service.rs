use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::error::{AppError, AppResult};

/// Validates a version string to prevent path traversal and injection attacks.
/// Only allows alphanumeric characters, dots, hyphens, and underscores.
/// This covers semver (1.2.3), pre-release (1.2.3-beta.1), and similar formats.
fn validate_version_string(version: &str) -> AppResult<()> {
    if version.is_empty() {
        return Err(AppError::Validation(
            "Version string cannot be empty".into(),
        ));
    }
    let is_valid = version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    if !is_valid {
        return Err(AppError::Validation(
            "Version string contains invalid characters".into(),
        ));
    }
    // Must start with a digit (rejects things like "-beta" or "_something")
    if !version
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
    {
        return Err(AppError::Validation(
            "Version string must start with a digit".into(),
        ));
    }
    Ok(())
}

pub struct RuntimeService {
    node_versions_dir: PathBuf,
    python_versions_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeProgress {
    pub phase: String,
    pub progress: u32,
    pub message: String,
}

impl RuntimeService {
    pub fn new() -> Self {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SelfHost Helper");
        Self {
            node_versions_dir: base.join("node-versions"),
            python_versions_dir: base.join("python-versions"),
        }
    }

    pub async fn get_installed_node_versions(&self) -> AppResult<Vec<String>> {
        self.list_versions(&self.node_versions_dir).await
    }

    pub async fn get_installed_python_versions(&self) -> AppResult<Vec<String>> {
        self.list_versions(&self.python_versions_dir).await
    }

    async fn list_versions(&self, base_dir: &Path) -> AppResult<Vec<String>> {
        let mut versions = Vec::new();
        match tokio::fs::read_dir(base_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
                    if entry.file_type().await.map_err(AppError::Io)?.is_dir() {
                        versions.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(AppError::Io(e)),
        }
        versions.sort();
        Ok(versions)
    }

    pub async fn install_node_version(
        &self,
        version: &str,
        progress_tx: Option<tokio::sync::mpsc::Sender<RuntimeProgress>>,
    ) -> AppResult<PathBuf> {
        validate_version_string(version)?;
        let target_dir = self.node_versions_dir.join(version);
        if tokio::fs::metadata(&target_dir).await.is_ok() {
            return Ok(target_dir);
        }
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(AppError::Io)?;

        let url = self.get_node_download_url(version)?;

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "downloading".into(),
                    progress: 0,
                    message: format!("Downloading Node.js {}", version),
                })
                .await;
        }

        let response = match reqwest::get(&url).await {
            Ok(r) => r,
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&target_dir).await;
                return Err(AppError::Http(e));
            }
        };
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&target_dir).await;
                return Err(AppError::Http(e));
            }
        };
        let zip_path = target_dir.with_extension("zip");
        if let Err(e) = tokio::fs::write(&zip_path, &bytes).await {
            let _ = tokio::fs::remove_dir_all(&target_dir).await;
            return Err(AppError::Io(e));
        }

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "extracting".into(),
                    progress: 50,
                    message: "Extracting...".into(),
                })
                .await;
        }

        let extract_target = target_dir.clone();
        let extract_zip = zip_path.clone();
        let extract_result = tokio::task::spawn_blocking(move || -> AppResult<()> {
            let file = std::fs::File::open(&extract_zip).map_err(AppError::Io)?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| AppError::Internal(e.to_string()))?;
            archive
                .extract(&extract_target)
                .map_err(|e| AppError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let _ = tokio::fs::remove_file(&zip_path).await;

        if let Err(e) = extract_result {
            let _ = tokio::fs::remove_dir_all(&target_dir).await;
            return Err(e);
        }

        // Verify the downloaded binary
        let verify_target = target_dir.clone();
        let node_name = if cfg!(windows) {
            "node.exe"
        } else {
            "node"
        };
        let node_binary = tokio::task::spawn_blocking(move || {
            find_binary(&verify_target, node_name)
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| {
            AppError::Runtime("Node.js binary not found after extraction".into())
        })?;
        verify_binary(&node_binary).await?;

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "done".into(),
                    progress: 100,
                    message: format!("Node.js {} installed", version),
                })
                .await;
        }

        Ok(target_dir)
    }

    pub async fn install_python_version(
        &self,
        version: &str,
        progress_tx: Option<tokio::sync::mpsc::Sender<RuntimeProgress>>,
    ) -> AppResult<PathBuf> {
        validate_version_string(version)?;
        let target_dir = self.python_versions_dir.join(version);
        if tokio::fs::metadata(&target_dir).await.is_ok() {
            return Ok(target_dir);
        }
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(AppError::Io)?;

        let url = self.get_python_download_url(version)?;

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "downloading".into(),
                    progress: 0,
                    message: format!("Downloading Python {}", version),
                })
                .await;
        }

        let response = match reqwest::get(&url).await {
            Ok(r) => r,
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&target_dir).await;
                return Err(AppError::Http(e));
            }
        };
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&target_dir).await;
                return Err(AppError::Http(e));
            }
        };
        let zip_path = target_dir.with_extension("zip");
        if let Err(e) = tokio::fs::write(&zip_path, &bytes).await {
            let _ = tokio::fs::remove_dir_all(&target_dir).await;
            return Err(AppError::Io(e));
        }

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "extracting".into(),
                    progress: 50,
                    message: "Extracting...".into(),
                })
                .await;
        }

        let extract_target = target_dir.clone();
        let extract_zip = zip_path.clone();
        let extract_result = tokio::task::spawn_blocking(move || -> AppResult<()> {
            let file = std::fs::File::open(&extract_zip).map_err(AppError::Io)?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| AppError::Internal(e.to_string()))?;
            archive
                .extract(&extract_target)
                .map_err(|e| AppError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let _ = tokio::fs::remove_file(&zip_path).await;

        if let Err(e) = extract_result {
            let _ = tokio::fs::remove_dir_all(&target_dir).await;
            return Err(e);
        }

        // Verify the downloaded binary
        let verify_target = target_dir.clone();
        let python_name = if cfg!(windows) {
            "python.exe"
        } else {
            "python3"
        };
        let python_binary = tokio::task::spawn_blocking(move || {
            find_binary(&verify_target, python_name)
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| {
            AppError::Runtime("Python binary not found after extraction".into())
        })?;
        verify_binary(&python_binary).await?;

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(RuntimeProgress {
                    phase: "done".into(),
                    progress: 100,
                    message: format!("Python {} installed", version),
                })
                .await;
        }

        Ok(target_dir)
    }

    pub async fn uninstall_version(&self, runtime: &str, version: &str) -> AppResult<()> {
        validate_version_string(version)?;
        let base_dir = match runtime {
            "node" => &self.node_versions_dir,
            "python" => &self.python_versions_dir,
            _ => return Err(AppError::Validation("Unknown runtime".into())),
        };
        let target = base_dir.join(version);
        match tokio::fs::remove_dir_all(&target).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::Io(e)),
        }
    }

    pub async fn get_available_node_versions(&self) -> AppResult<Vec<String>> {
        let output = Command::new("node")
            .arg("--list-remote")
            .arg("lts")
            .output()
            .await
            .map_err(|e| {
                AppError::Process(format!("Failed to list Node versions: {}", e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let versions: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with('v') {
                    Some(line.strip_prefix('v').unwrap_or(line).to_string())
                } else {
                    None
                }
            })
            .rev()
            .take(20)
            .collect();
        Ok(versions)
    }

    pub async fn get_available_python_versions(&self) -> AppResult<Vec<String>> {
        Ok(vec![
            "3.12.4".into(),
            "3.12.3".into(),
            "3.12.2".into(),
            "3.12.1".into(),
            "3.12.0".into(),
            "3.11.9".into(),
            "3.11.8".into(),
            "3.11.7".into(),
            "3.10.14".into(),
            "3.10.13".into(),
            "3.10.12".into(),
        ])
    }

    fn get_node_download_url(&self, version: &str) -> AppResult<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let (os_name, arch_name) = match (os, arch) {
            ("linux", "x86_64") => ("linux", "x64"),
            ("linux", "aarch64") => ("linux", "arm64"),
            ("macos", "x86_64") => ("darwin", "x64"),
            ("macos", "aarch64") => ("darwin", "arm64"),
            ("windows", "x86_64") => ("win", "x64"),
            ("windows", "aarch64") => ("win", "arm64"),
            _ => {
                return Err(AppError::Runtime(format!(
                    "Unsupported platform: {}-{}",
                    os, arch
                )))
            }
        };
        let ext = if os == "windows" { "zip" } else { "tar.gz" };
        Ok(format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}-{}.{}",
            version, version, os_name, arch_name, ext
        ))
    }

    fn get_python_download_url(&self, version: &str) -> AppResult<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let (os_name, arch_name) = match (os, arch) {
            ("linux", "x86_64") => ("linux", "x86_64"),
            ("linux", "aarch64") => ("linux", "aarch64"),
            ("macos", "x86_64") => ("macos", "x86_64"),
            ("macos", "aarch64") => ("macos", "aarch64"),
            ("windows", "x86_64") => ("win", "amd64"),
            ("windows", "aarch64") => ("win", "arm64"),
            _ => {
                return Err(AppError::Runtime(format!(
                    "Unsupported platform: {}-{}",
                    os, arch
                )))
            }
        };
        let suffix = format!("{}-{}", os_name, arch_name);
        Ok(format!(
            "https://www.python.org/ftp/python/{}/python-{}-{}.zip",
            version, version, suffix
        ))
    }
}

fn find_binary(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    stack.push(path);
                } else if entry.file_name().to_string_lossy() == name {
                    return Some(path);
                }
            }
        }
    }
    None
}

async fn verify_binary(binary_path: &std::path::Path) -> AppResult<()> {
    let metadata = tokio::fs::metadata(binary_path).await.map_err(|e| {
        AppError::Runtime(format!("Binary not found after download: {}", e))
    })?;
    if !metadata.is_file() {
        return Err(AppError::Runtime("Binary path is not a file".into()));
    }
    if metadata.len() == 0 {
        return Err(AppError::Runtime("Downloaded binary is empty".into()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            return Err(AppError::Runtime(
                "Downloaded binary is not executable".into(),
            ));
        }
        let mut magic = [0u8; 4];
        let mut file = tokio::fs::File::open(binary_path)
            .await
            .map_err(AppError::Io)?;
        tokio::io::AsyncReadExt::read_exact(&mut file, &mut magic)
            .await
            .map_err(AppError::Io)?;
        if magic != [0x7f, 0x45, 0x4c, 0x46] {
            let _ = tokio::fs::remove_file(binary_path).await;
            return Err(AppError::Runtime(
                "Downloaded binary is not a valid ELF file".into(),
            ));
        }
    }
    Ok(())
}
