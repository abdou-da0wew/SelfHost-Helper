use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::services::audit_logger::AuditEntry;
use crate::{ServicesState, Pipe};

#[tauri::command]
pub async fn start_tunnel(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
    mode: String,
    port: u16,
    token: Option<String>,
    config: Option<HashMap<String, String>>,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = state
        .tunnel_manager
        .start_tunnel(project_id.clone(), mode, port, token, config)
        .await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tunnel:start".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn stop_tunnel(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = state.tunnel_manager.stop_tunnel(&project_id).await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tunnel:stop".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_tunnel_status(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<Option<serde_json::Value>> {
    let __start = std::time::Instant::now();
    let __result = Ok(state
        .tunnel_manager
        .get_status(&project_id)
        .await
        .and_then(|s| serde_json::to_value(s).ok()));
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tunnel:status".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_tunnel_logs(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<Vec<serde_json::Value>> {
    let __start = std::time::Instant::now();
    let __result = state
        .tunnel_manager
        .get_logs(&project_id)
        .await
        .into_iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect::<Vec<_>>()
        .pipe(Ok);
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tunnel:logs".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn clear_tunnel_logs(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    state.tunnel_manager.clear_logs(&project_id).await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tunnel:clear_logs".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: "success".into(),
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    Ok(())
}

#[tauri::command]
pub async fn export_tunnel_logs_project(
    state: tauri::State<'_, ServicesState>,
    project_id: String,
) -> AppResult<String> {
    let __start = std::time::Instant::now();
    let logs = state.tunnel_manager.get_logs(&project_id).await;
    let __result = serde_json::to_string_pretty(&logs)
        .map_err(|e| AppError::Internal(e.to_string()));
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "export:tunnel_logs".into(),
        actor: "system".into(),
        target: Some(project_id),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn export_tunnel_logs_all() -> AppResult<String> {
    Ok("{}".to_string())
}
