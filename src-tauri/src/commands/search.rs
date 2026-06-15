use crate::error::AppResult;
use crate::services::audit_logger::AuditEntry;
use crate::{validate_project_path, DbState, ServicesState};

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn search_files(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    directory: String,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    regex_mode: bool,
    max_results: Option<usize>,
) -> AppResult<serde_json::Value> {
    let __start = std::time::Instant::now();
    let __result = async {
        let _resolved = validate_project_path(&directory, &db_state)?;
        let (results, stats) = services_state
            .search_service
            .search(
                &directory,
                &query,
                case_sensitive,
                whole_word,
                regex_mode,
                max_results.unwrap_or(500),
            )
            .await?;
        Ok(serde_json::json!({ "results": results, "stats": stats }))
    }
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "search:files".into(),
        actor: "system".into(),
        target: Some(directory),
        details: serde_json::json!({"query": query, "case_sensitive": case_sensitive, "whole_word": whole_word, "regex_mode": regex_mode}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}
