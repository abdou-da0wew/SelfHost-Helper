use crate::error::AppResult;
use crate::services::audit_logger::AuditEntry;
use crate::ServicesState;

#[tauri::command]
pub async fn check_for_updates(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Option<crate::services::update_service::UpdateInfo>> {
    let __start = std::time::Instant::now();
    let __result = state.update_service.check_for_updates().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "update:check".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn download_and_install_update(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = state.update_service.download_and_install().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "update:install".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_update_status(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<crate::services::update_service::UpdateStatus> {
    let __start = std::time::Instant::now();
    let __result = Ok(state.update_service.get_status().await);
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "update:status".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_current_version(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<String> {
    let __start = std::time::Instant::now();
    let __result = Ok(state.update_service.get_current_version().await);
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "update:version".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
