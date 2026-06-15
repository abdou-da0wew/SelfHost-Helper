use crate::error::AppError;
use crate::error::AppResult;
use crate::services::audit_logger::AuditEntry;
use crate::{validate_project_path, DbState, ServicesState};

#[tauri::command]
pub async fn watch_directory(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    paths: Vec<String>,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = async {
        for p in &paths {
            let _resolved = validate_project_path(p, &db_state)?;
        }
        services_state
            .file_watcher
            .watch(paths)
            .await
            .map_err(AppError::Internal)
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "file_watcher:watch".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn unwatch_all(
    services_state: tauri::State<'_, ServicesState>,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = services_state
        .file_watcher
        .unwatch_all()
        .await
        .map_err(AppError::Internal);
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "file_watcher:unwatch_all".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn is_watching(
    services_state: tauri::State<'_, ServicesState>,
) -> AppResult<bool> {
    let __start = std::time::Instant::now();
    let __result = Ok(services_state.file_watcher.is_watching().await);
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "file_watcher:is_watching".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
