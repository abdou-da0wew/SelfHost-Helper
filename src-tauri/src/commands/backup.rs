use crate::error::{AppError, AppResult};
use crate::{DbState, ServicesState};

#[tauri::command]
pub fn create_backup(
    db_state: tauri::State<DbState>,
    services_state: tauri::State<ServicesState>,
    passphrase: String,
) -> AppResult<String> {
    services_state.audit_logger.timed("backup:create", || {
        let projects = db_state
            .projects_repo
            .get_all()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let categories = db_state
            .categories_repo
            .get_all()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let path = services_state
            .backup_service
            .create_backup(&projects, &categories, &passphrase, None)?;
        Ok(path.to_string_lossy().to_string())
    })
}

#[tauri::command]
pub fn restore_backup(
    services_state: tauri::State<ServicesState>,
    zip_path: String,
    passphrase: String,
) -> AppResult<crate::utils::crypto::BackupPayload> {
    services_state.audit_logger.timed("backup:restore", || {
        let zip = std::path::Path::new(&zip_path);
        services_state.backup_service.validate_backup_path(zip)?;
        services_state
            .backup_service
            .restore_backup(zip, &passphrase)
    })
}

#[tauri::command]
pub fn list_backups(
    services_state: tauri::State<ServicesState>,
) -> AppResult<Vec<crate::services::backup_service::BackupInfo>> {
    services_state
        .audit_logger
        .timed("backup:list", || services_state.backup_service.list_backups())
}

#[tauri::command]
pub fn delete_backup(
    services_state: tauri::State<ServicesState>,
    filename: String,
) -> AppResult<()> {
    services_state.audit_logger.timed("backup:delete", || {
        services_state.backup_service.delete_backup(&filename)
    })
}
