use std::path::PathBuf;
use tauri::Manager;

use crate::error::{AppError, AppResult};

/// Resolves the path to a sidecar binary.
///
/// Resolution order:
/// 1. `{resource_dir}/binaries/{name}-{target_triple}` (bundled in production)
/// 2. `{exe_dir}/../binaries/{name}-{target_triple}` (dev mode fallback)
/// 3. `which(name)` — system PATH fallback
pub fn resolve_sidecar(app_handle: &tauri::AppHandle, name: &str) -> AppResult<PathBuf> {
    let target_triple = get_target_triple();
    let binary_name = format!("{}-{}", name, target_triple);

    // 1. Try resource dir (production / bundled)
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let path = resource_dir.join("binaries").join(&binary_name);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Try relative to executable (dev mode)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // In dev, exe is in target/debug/ or target/release/
            // binaries/ is at src-tauri/binaries/
            let dev_path = exe_dir
                .join("..")
                .join("..")
                .join("..")
                .join("binaries")
                .join(&binary_name);
            if dev_path.exists() {
                return Ok(dev_path);
            }

            // Also try alongside the exe (flat layout)
            let flat_path = exe_dir.join("binaries").join(&binary_name);
            if flat_path.exists() {
                return Ok(flat_path);
            }
        }
    }

    // 3. PATH fallback
    if let Ok(path) = which::which(name) {
        return Ok(path);
    }

    Err(AppError::Process(format!(
        "Could not find sidecar '{}'. Ensure it is bundled or available in PATH.",
        name
    )))
}

fn get_target_triple() -> &'static str {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else {
        "unknown-unknown-unknown"
    }
}
