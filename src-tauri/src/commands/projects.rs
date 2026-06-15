use crate::error::{AppError, AppResult};
use crate::{DbState, ServicesState};

#[tauri::command]
pub fn get_projects(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
) -> AppResult<Vec<serde_json::Value>> {
    services.audit_logger.timed("project:get_all", || {
        state
            .projects_repo
            .get_all()
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
pub fn add_project(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    name: String,
    path: String,
    category_id: Option<i64>,
    tags: Option<String>,
) -> AppResult<i64> {
    services.audit_logger.timed("project:create", || {
        state
            .projects_repo
            .create(&name, &path, category_id, tags.as_deref())
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_project(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    id: i64,
    name: Option<String>,
    path: Option<String>,
    category_id: Option<i64>,
    tags: Option<String>,
    icon: Option<String>,
) -> AppResult<()> {
    services.audit_logger.timed("project:update", || {
        state
            .projects_repo
            .update(
                id,
                name.as_deref(),
                path.as_deref(),
                category_id,
                tags.as_deref(),
                icon.as_deref(),
            )
            .map_err(|e| AppError::Database(e.to_string()))
    })
}

#[tauri::command]
pub fn delete_project(
    state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    id: i64,
) -> AppResult<()> {
    services.audit_logger.timed("project:delete", || {
        state
            .projects_repo
            .delete(id)
            .map_err(|e| AppError::Database(e.to_string()))
    })
}
