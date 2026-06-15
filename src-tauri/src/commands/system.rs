use tauri::Manager;

use crate::error::{AppError, AppResult};
use crate::services::audit_logger::AuditEntry;
use crate::{validate_project_path, AppConfig, DbState, ServicesState};

#[tauri::command]
pub fn open_file(
    db_state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    path: String,
) -> AppResult<()> {
    services.audit_logger.timed("system:open_file", || {
        let _resolved = validate_project_path(&path, &db_state)?;
        opener::open(&path).map_err(|e| AppError::Internal(e.to_string()))
    })
}

#[tauri::command]
pub fn open_folder(
    db_state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    path: String,
) -> AppResult<()> {
    services.audit_logger.timed("system:open_folder", || {
        let _resolved = validate_project_path(&path, &db_state)?;
        opener::open(&path).map_err(|e| AppError::Internal(e.to_string()))
    })
}

#[tauri::command]
pub fn open_external(url: String) -> AppResult<()> {
    opener::open(&url).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub fn get_platform_info() -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": if cfg!(target_os = "windows") { "windows" }
                  else if cfg!(target_os = "macos") { "macos" }
                  else { "linux" },
    }))
}

#[tauri::command]
pub fn set_debug_mode(
    config: tauri::State<AppConfig>,
    services: tauri::State<ServicesState>,
    enabled: bool,
) -> AppResult<()> {
    services.audit_logger.timed("debug:set_mode", || {
        use std::sync::atomic::Ordering;
        config.debug_mode.store(enabled, Ordering::SeqCst);
        Ok(())
    })
}

#[tauri::command]
pub fn get_debug_mode(config: tauri::State<AppConfig>) -> bool {
    use std::sync::atomic::Ordering;
    config.debug_mode.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn set_log_level(level: String) -> crate::error::AppResult<()> {
    let filter = match level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            return Err(crate::error::AppError::Validation(format!(
                "Invalid log level: {}. Must be one of: error, warn, info, debug, trace",
                level
            )))
        }
    };
    log::set_max_level(filter);
    Ok(())
}

#[tauri::command]
pub async fn get_logs(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<Vec<crate::services::log_store::LogEntry>> {
    let __start = std::time::Instant::now();
    let __result = Ok(state.log_store.get_history(&project_id).await);
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "log:get".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_all_logs(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Vec<crate::services::log_store::LogBatch>> {
    let __start = std::time::Instant::now();
    let __result = Ok(state.log_store.drain_batch().await);
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "log:get_all".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn clear_logs(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    state.log_store.clear_history(&project_id).await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "log:clear".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    Ok(())
}

#[tauri::command]
pub async fn export_console_logs_project(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<String> {
    let __start = std::time::Instant::now();
    let logs = state.log_store.get_history(&project_id).await;
    let __result = serde_json::to_string_pretty(&logs)
        .map_err(|e| AppError::Internal(e.to_string()));
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "export:console_logs".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn export_console_logs_all() -> AppResult<String> {
    Ok("{}".to_string())
}

#[tauri::command]
pub fn close_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        w.close()
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn minimize_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        w.minimize()
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_maximize(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        let is_maximized = w
            .is_maximized()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if is_maximized {
            w.unmaximize()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            w.maximize()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn select_directory() -> AppResult<Option<String>> {
    let picker = rfd::AsyncFileDialog::new().pick_folder().await;
    Ok(picker.map(|p| p.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn select_file() -> AppResult<Option<String>> {
    let picker = rfd::AsyncFileDialog::new().pick_file().await;
    Ok(picker.map(|p| p.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub fn join_path(base: String, parts: Vec<String>) -> String {
    let mut path = std::path::PathBuf::from(base);
    for p in parts {
        path.push(p);
    }
    path.to_string_lossy().to_string()
}
