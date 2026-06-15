use std::path::PathBuf;

use crate::error::AppResult;
use crate::services::audit_logger::AuditEntry;
use crate::ServicesState;

#[tauri::command]
pub async fn get_installed_node_versions(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Vec<String>> {
    let __start = std::time::Instant::now();
    let __result = state.runtime_service.get_installed_node_versions().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:node_versions".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_installed_python_versions(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Vec<String>> {
    let __start = std::time::Instant::now();
    let __result = state.runtime_service.get_installed_python_versions().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:python_versions".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn install_node_version(
    state: tauri::State<'_, ServicesState>,
    version: String,
) -> AppResult<PathBuf> {
    let __start = std::time::Instant::now();
    let __result = state
        .runtime_service
        .install_node_version(&version, None)
        .await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:install_node".into(),
        actor: "system".into(),
        target: Some(version),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn install_python_version(
    state: tauri::State<'_, ServicesState>,
    version: String,
) -> AppResult<PathBuf> {
    let __start = std::time::Instant::now();
    let __result = state
        .runtime_service
        .install_python_version(&version, None)
        .await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:install_python".into(),
        actor: "system".into(),
        target: Some(version),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn uninstall_runtime_version(
    state: tauri::State<'_, ServicesState>,
    runtime: String,
    version: String,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = state
        .runtime_service
        .uninstall_version(&runtime, &version)
        .await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:uninstall".into(),
        actor: "system".into(),
        target: Some(format!("{}/{}", runtime, version)),
        details: serde_json::json!({"runtime": runtime, "version": version}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_available_node_versions(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Vec<String>> {
    let __start = std::time::Instant::now();
    let __result = state.runtime_service.get_available_node_versions().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:available_node".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn get_available_python_versions(
    state: tauri::State<'_, ServicesState>,
) -> AppResult<Vec<String>> {
    let __start = std::time::Instant::now();
    let __result = state.runtime_service.get_available_python_versions().await;
    state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "runtime:available_python".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
