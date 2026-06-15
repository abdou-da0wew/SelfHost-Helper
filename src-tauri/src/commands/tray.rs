use crate::error::{AppError, AppResult};
use crate::native::tray;
use crate::services::audit_logger::AuditEntry;
use crate::{DbState, ServicesState};

#[tauri::command]
pub async fn refresh_tray_menu(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let __start = std::time::Instant::now();
    let __result = async {
        let projects_repo = db_state.projects_repo.clone();
        let projects = tokio::task::spawn_blocking(move || {
            projects_repo
                .get_all()
                .map_err(|e| AppError::Database(e.to_string()))
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
        tray::rebuild_tray_with_projects(&app, projects)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "tray:refresh".into(),
        actor: "system".into(),
        target: None,
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
