use crate::error::{AppError, AppResult};
use crate::services::audit_logger::AuditEntry;
use crate::{validate_project_path, DbState, ServicesState};

#[tauri::command]
pub async fn start_lsp_server(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    project_path: String,
    server_command: String,
    args: Vec<String>,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = async {
        let _resolved = validate_project_path(&project_path, &db_state)?;
        services_state
            .lsp_session_manager
            .start_server(&project_path, &server_command, args)
            .await
            .map_err(AppError::Internal)
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "lsp:start".into(),
        actor: "system".into(),
        target: Some(project_path),
        details: serde_json::json!({"server_command": server_command}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn stop_lsp_server(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    project_path: String,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = async {
        let _resolved = validate_project_path(&project_path, &db_state)?;
        services_state
            .lsp_session_manager
            .stop_server(&project_path)
            .await
            .map_err(AppError::Internal)
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "lsp:stop".into(),
        actor: "system".into(),
        target: Some(project_path),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn lsp_request(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    project_path: String,
    method: String,
    params: serde_json::Value,
) -> AppResult<serde_json::Value> {
    let __start = std::time::Instant::now();
    let __result = async {
        let _resolved = validate_project_path(&project_path, &db_state)?;
        services_state
            .lsp_session_manager
            .request(&project_path, &method, params)
            .await
            .map_err(AppError::Internal)
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "lsp:request".into(),
        actor: "system".into(),
        target: Some(project_path),
        details: serde_json::json!({"method": method}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn start_lsp_proxy(
    services_state: tauri::State<'_, ServicesState>,
) -> AppResult<u16> {
    let __start = std::time::Instant::now();
    let __result = services_state
        .lsp_proxy
        .start()
        .await
        .map_err(AppError::Internal);
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "lsp:proxy".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
