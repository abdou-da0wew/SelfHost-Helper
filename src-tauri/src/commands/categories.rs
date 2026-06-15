use crate::error::{AppError, AppResult};
use crate::{DbState, ServicesState};

#[tauri::command]
pub fn get_categories(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
) -> AppResult<Vec<serde_json::Value>> {
    services.audit_logger.timed("category:get_all", || {
        state
            .categories_repo
            .get_all()
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
pub fn add_category(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    name: String,
    color: Option<String>,
) -> AppResult<i64> {
    services.audit_logger.timed("category:create", || {
        state
            .categories_repo
            .create(&name, color.as_deref())
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
pub fn update_category(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
) -> AppResult<()> {
    services.audit_logger.timed("category:update", || {
        state
            .categories_repo
            .update(id, name.as_deref(), color.as_deref())
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
pub fn delete_category(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    id: i64,
) -> AppResult<()> {
    services.audit_logger.timed("category:delete", || {
        state
            .categories_repo
            .delete(id)
            .map_err(|e| AppError::Database(e.to_string()))
    })
}
