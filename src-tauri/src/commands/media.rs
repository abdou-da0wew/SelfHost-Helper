use crate::error::{AppError, AppResult};
use crate::{AppConfig, ServicesState};

#[tauri::command]
pub fn validate_media_path(
    config: tauri::State<AppConfig>,
    services: tauri::State<ServicesState>,
    path: String,
) -> AppResult<String> {
    services.audit_logger.timed("media:validate", || {
        let file_path = std::path::Path::new(&path);
        config
            .media_allowlist
            .validate_media_path(file_path)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| AppError::MediaAllowlist(e.to_string()))
    })
}
