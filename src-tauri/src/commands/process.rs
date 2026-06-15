use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::services::audit_logger::AuditEntry;
use crate::{DbState, ServicesState};

#[tauri::command]
pub async fn start_process(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    _project_id: i64,
    project_path: String,
    command: String,
) -> AppResult<String> {
    let pp_for_log = project_path.clone();
    let __start = std::time::Instant::now();
    let projects_repo = db_state.projects_repo.clone();
    let (dir, parts) = tokio::task::spawn_blocking(move || {
        let candidate = std::path::Path::new(&project_path);
        let resolved = dunce::canonicalize(candidate).map_err(|_| {
            AppError::PathSecurity(format!(
                "path does not exist or cannot be resolved: {}",
                project_path
            ))
        })?;
        let projects = projects_repo
            .get_all()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let project_roots: Vec<PathBuf> = projects
            .iter()
            .filter_map(|p| p.get("path").and_then(|v| v.as_str()).map(PathBuf::from))
            .collect();
        if project_roots.is_empty() {
            return Err(AppError::PathSecurity(
                "no project roots registered".into(),
            ));
        }
        if !resolved.is_dir() {
            return Err(AppError::Validation("Invalid project path".into()));
        }
        let parts = shlex::split(&command)
            .ok_or_else(|| AppError::Validation("Invalid command syntax".into()))?;
        if parts.is_empty() {
            return Err(AppError::Validation("Empty command".into()));
        }
        Ok::<_, AppError>((resolved, parts))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        tokio::process::Command::new(&parts[0])
            .args(&parts[1..])
            .current_dir(dir)
            .output(),
    )
    .await
    .map_err(|_| AppError::Process("Command timed out after 300s".into()))?
    .map_err(|e| AppError::Process(e.to_string()))?;
    let __result = Ok(String::from_utf8_lossy(&output.stdout).to_string());
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "process:start".into(),
        actor: "system".into(),
        target: Some(pp_for_log),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub async fn stop_process(pid: u32) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output()
            .await
            .map_err(|e| AppError::Process(e.to_string()))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
            .map_err(|e| AppError::Process(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_process(
    db_state: tauri::State<'_, DbState>,
    services_state: tauri::State<'_, ServicesState>,
    project_id: i64,
    project_path: String,
    command: String,
) -> AppResult<String> {
    let pp_for_log = project_path.clone();
    let __start = std::time::Instant::now();
    let __result = start_process(
        db_state,
        services_state.clone(),
        project_id,
        project_path,
        command,
    )
    .await;
    services_state.audit_logger.log(AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "process:restart".into(),
        actor: "system".into(),
        target: Some(pp_for_log),
        details: serde_json::json!({}),
        outcome: if __result.is_ok() { "success".into() } else { "error".into() },
        duration_ms: __start.elapsed().as_millis() as u64,
    });
    __result
}

#[tauri::command]
pub fn get_process_status(pid: u32) -> AppResult<bool> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 {
                return Ok(false);
            }
            let mut exit_code = 0u32;
            let success = GetExitCodeProcess(handle, &mut exit_code);
            windows_sys::Win32::Foundation::CloseHandle(handle);
            Ok(success != 0 && exit_code == 259) // STILL_ACTIVE
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), None).is_ok())
    }
}
