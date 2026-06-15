use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::ServicesState;

#[tauri::command]
pub fn get_app_settings(
    state: tauri::State<ServicesState>,
) -> AppResult<crate::services::settings_service::AppSettings> {
    state
        .audit_logger
        .timed("settings:get", || state.settings_service.get_app_settings())
}

#[tauri::command]
pub fn set_app_setting(
    state: tauri::State<ServicesState>,
    key: String,
    value: String,
) -> AppResult<()> {
    state.audit_logger.timed("settings:set", || {
        state.settings_service.set_app_setting(&key, &value)
    })
}

#[tauri::command]
pub fn update_app_settings(
    state: tauri::State<ServicesState>,
    settings: HashMap<String, String>,
) -> AppResult<()> {
    state.audit_logger.timed("settings:update", || {
        state.settings_service.update_app_settings(&settings)
    })
}

#[tauri::command]
pub fn get_project_settings(
    state: tauri::State<ServicesState>,
    project_id: String,
) -> AppResult<Option<crate::services::settings_service::ProjectSettings>> {
    state.audit_logger.timed("settings:get_project", || {
        state.settings_service.get_project_settings(&project_id)
    })
}

#[tauri::command]
pub fn set_project_settings(
    state: tauri::State<ServicesState>,
    settings: crate::services::settings_service::ProjectSettings,
) -> AppResult<()> {
    state.audit_logger.timed("settings:set_project", || {
        state.settings_service.set_project_settings(&settings)
    })
}

#[tauri::command]
pub fn delete_project_settings(
    state: tauri::State<ServicesState>,
    project_id: String,
) -> AppResult<()> {
    state.audit_logger.timed("settings:delete_project", || {
        state.settings_service.delete_project_settings(&project_id)
    })
}

#[tauri::command]
pub fn get_settings(state: tauri::State<ServicesState>) -> AppResult<serde_json::Value> {
    state.audit_logger.timed("settings:get_json", || {
        let s = state.settings_service.get_app_settings()?;
        serde_json::to_value(s).map_err(|e| AppError::Internal(e.to_string()))
    })
}
