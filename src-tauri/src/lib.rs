mod db;
mod error;
mod native;
mod services;
mod utils;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

use crate::db::connection::establish_connection;
use crate::db::projects_repo::ProjectsRepo;
use crate::db::categories_repo::CategoriesRepo;
use crate::error::{AppError, AppResult};
use crate::native::tray;
use crate::services::backup_service::BackupService;
use crate::services::file_watcher::FileWatcher;
use crate::services::git_service::GitService;
use crate::services::lsp_bridge::LspBridge;
use crate::services::log_store::LogStore;
use crate::services::runtime_service::RuntimeService;
use crate::services::search_service::SearchService;
use crate::services::settings_service::SettingsService;
use crate::services::tunnel_manager::TunnelManager;
use crate::services::update_service::UpdateService;
use crate::utils::media_allowlist::MediaAllowlist;
use crate::utils::path_security;

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub projects_repo: Arc<ProjectsRepo>,
    pub categories_repo: Arc<CategoriesRepo>,
    pub settings_service: Arc<SettingsService>,
    pub git_service: Arc<GitService>,
    pub search_service: Arc<SearchService>,
    pub lsp_bridge: Arc<LspBridge>,
    pub runtime_service: Arc<RuntimeService>,
    pub file_watcher: Arc<FileWatcher>,
    pub backup_service: Arc<BackupService>,
    pub tunnel_manager: Arc<TunnelManager>,
    pub update_service: Arc<UpdateService>,
    pub media_allowlist: Arc<MediaAllowlist>,
    pub log_store: Arc<LogStore>,
    pub dev_mode: bool,
    pub debug_mode: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let is_dev = handle.config().build.dev_url.is_some();
            let app_base = handle
                .path()
                .app_data_dir()
                .unwrap_or_default();
            let db_path = app_base.join("data").join("projects.sqlite");
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let db = establish_connection(&db_path)?;
            let projects_repo = Arc::new(ProjectsRepo::new(db.clone()));
            let categories_repo = Arc::new(CategoriesRepo::new(db.clone()));
            let settings_service = Arc::new(SettingsService::new(db.clone()));
            settings_service.init_tables().ok();
            let git_service = Arc::new(GitService::new());
            let search_service = Arc::new(SearchService::new(handle.clone()));
            let lsp_bridge = Arc::new(LspBridge::new());
            let runtime_service = Arc::new(RuntimeService::new());
            let media_allowlist = Arc::new(MediaAllowlist::new());
            let log_store = Arc::new(LogStore::new());
            let (tunnel_manager, mut tunnel_rx) = TunnelManager::new(handle.clone());
            tokio::spawn(async move {
                while tunnel_rx.recv().await.is_some() {}
            });
            let backup_dir = app_base.join("backups");
            let backup_service = Arc::new(BackupService::new(
                backup_dir,
                settings_service.clone(),
            ));
            let (file_watcher_tx, mut file_watcher_rx) = tokio::sync::mpsc::channel(64);
            let file_watcher = Arc::new(FileWatcher::new(file_watcher_tx));
            let update_service = Arc::new(UpdateService::new(handle.clone()));
            update_service.start_periodic_check();

            let app_state = AppState {
                db,
                projects_repo,
                categories_repo,
                settings_service,
                git_service,
                search_service,
                lsp_bridge,
                runtime_service,
                file_watcher,
                backup_service,
                tunnel_manager: Arc::new(tunnel_manager),
                update_service,
                media_allowlist,
                log_store,
                dev_mode: is_dev,
                debug_mode: Arc::new(AtomicBool::new(false)),
            };
            app.manage(app_state);

            let handle_clone = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(change) = file_watcher_rx.recv().await {
                    let _ = handle_clone.emit("file-changed", &change);
                }
            });

            tray::create_tray(&handle).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_projects,
            add_project,
            update_project,
            delete_project,
            get_categories,
            add_category,
            update_category,
            delete_category,
            start_process,
            stop_process,
            restart_process,
            get_process_status,
            start_tunnel,
            stop_tunnel,
            get_tunnel_status,
            get_tunnel_logs,
            clear_tunnel_logs,
            git_get_branches,
            git_checkout_branch,
            git_create_branch,
            git_delete_branch,
            git_get_status,
            git_stage_file,
            git_unstage_file,
            git_stage_all,
            git_commit,
            git_push,
            git_pull,
            git_clone,
            git_get_remotes,
            git_add_remote,
            git_get_diff_summary,
            git_get_log,
            git_stash,
            git_stash_list,
            git_stash_pop,
            git_stash_drop,
            git_get_tags,
            git_diff_file,
            git_get_commit_diff,
            git_get_branch_ahead_behind,
            git_get_current_branch,
            git_stash_apply,
            search_files,
            get_installed_node_versions,
            get_installed_python_versions,
            install_node_version,
            install_python_version,
            uninstall_runtime_version,
            get_available_node_versions,
            get_available_python_versions,
            get_app_settings,
            set_app_setting,
            update_app_settings,
            get_project_settings,
            set_project_settings,
            delete_project_settings,
            create_backup,
            restore_backup,
            list_backups,
            delete_backup,
            check_for_updates,
            download_and_install_update,
            get_update_status,
            get_current_version,
            watch_directory,
            unwatch_all,
            is_watching,
            start_lsp_server,
            stop_lsp_server,
            lsp_request,
            start_lsp_proxy,
            open_file,
            open_folder,
            open_external,
            get_platform_info,
            validate_media_path,
            refresh_tray_menu,
            set_debug_mode,
            get_debug_mode,
            get_logs,
            get_all_logs,
            clear_logs,
            export_tunnel_logs_project,
            export_tunnel_logs_all,
            export_console_logs_project,
            export_console_logs_all,
            close_window,
            minimize_window,
            toggle_maximize,
            select_directory,
            select_file,
            get_settings,
            join_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== PATH VALIDATION =====
fn validate_project_path(path: &str, state: &AppState) -> AppResult<std::path::PathBuf> {
    let candidate = std::path::Path::new(path);
    if !candidate.exists() {
        return Err(AppError::PathSecurity(format!(
            "path does not exist: {}",
            path
        )));
    }
    let resolved = dunce::canonicalize(candidate)
        .map_err(|e| AppError::PathSecurity(format!("failed to resolve path: {}", e)))?;
    let projects = state
        .projects_repo
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
    path_security::validate_inside_roots(&resolved, &project_roots)?;
    Ok(resolved)
}

// ===== PROJECT COMMANDS =====
#[tauri::command]
fn get_projects(state: tauri::State<AppState>) -> AppResult<Vec<serde_json::Value>> {
    state
        .projects_repo
        .get_all()
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn add_project(
    state: tauri::State<AppState>,
    name: String,
    path: String,
    category_id: Option<i64>,
    tags: Option<String>,
) -> AppResult<i64> {
    state
        .projects_repo
        .create(&name, &path, category_id, tags.as_deref())
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn update_project(
    state: tauri::State<AppState>,
    id: i64,
    name: Option<String>,
    path: Option<String>,
    category_id: Option<i64>,
    tags: Option<String>,
    icon: Option<String>,
) -> AppResult<()> {
    state
        .projects_repo
        .update(
            id,
            name.as_deref(),
            path.as_deref(),
            category_id,
            tags.as_deref(),
            icon.as_deref(),
        )
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn delete_project(state: tauri::State<AppState>, id: i64) -> AppResult<()> {
    state
        .projects_repo
        .delete(id)
        .map_err(|e| AppError::Database(e.to_string()))
}

// ===== CATEGORY COMMANDS =====
#[tauri::command]
fn get_categories(state: tauri::State<AppState>) -> AppResult<Vec<serde_json::Value>> {
    state
        .categories_repo
        .get_all()
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn add_category(
    state: tauri::State<AppState>,
    name: String,
    color: Option<String>,
) -> AppResult<i64> {
    state
        .categories_repo
        .create(&name, color.as_deref())
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn update_category(
    state: tauri::State<AppState>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
) -> AppResult<()> {
    state
        .categories_repo
        .update(id, name.as_deref(), color.as_deref())
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn delete_category(state: tauri::State<AppState>, id: i64) -> AppResult<()> {
    state
        .categories_repo
        .delete(id)
        .map_err(|e| AppError::Database(e.to_string()))
}

// ===== PROCESS COMMANDS =====
#[tauri::command]
async fn start_process(
    state: tauri::State<'_, AppState>,
    _project_id: i64,
    project_path: String,
    command: String,
) -> AppResult<String> {
    let _resolved = validate_project_path(&project_path, &state)?;
    let dir = std::path::Path::new(&project_path);
    if !dir.is_dir() {
        return Err(AppError::Validation("Invalid project path".into()));
    }
    let parts = shlex::split(&command)
        .ok_or_else(|| AppError::Validation("Invalid command syntax".into()))?;
    if parts.is_empty() {
        return Err(AppError::Validation("Empty command".into()));
    }
    let output = tokio::process::Command::new(&parts[0])
        .args(&parts[1..])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| AppError::Process(e.to_string()))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
async fn stop_process(pid: u32) -> AppResult<()> {
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
async fn restart_process(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    project_path: String,
    command: String,
) -> AppResult<String> {
    let _ = stop_process(0).await;
    start_process(state, project_id, project_path, command).await
}

#[tauri::command]
fn get_process_status(pid: u32) -> AppResult<bool> {
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

// ===== TUNNEL COMMANDS =====
#[tauri::command]
async fn start_tunnel(
    state: tauri::State<'_, AppState>,
    project_id: String,
    mode: String,
    port: u16,
    token: Option<String>,
    config: Option<HashMap<String, String>>,
) -> AppResult<()> {
    state
        .tunnel_manager
        .start_tunnel(project_id, mode, port, token, config)
        .await
}

#[tauri::command]
async fn stop_tunnel(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    state.tunnel_manager.stop_tunnel(&project_id).await
}

#[tauri::command]
async fn get_tunnel_status(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<Option<serde_json::Value>> {
    Ok(state
        .tunnel_manager
        .get_status(&project_id)
        .await
        .and_then(|s| serde_json::to_value(s).ok()))
}

#[tauri::command]
async fn get_tunnel_logs(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<serde_json::Value>> {
    state
        .tunnel_manager
        .get_logs(&project_id)
        .await
        .into_iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect::<Vec<_>>()
        .pipe(Ok)
}

#[tauri::command]
async fn clear_tunnel_logs(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    state.tunnel_manager.clear_logs(&project_id).await;
    Ok(())
}

// ===== GIT COMMANDS =====
#[tauri::command]
fn git_get_branches(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitBranch>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_branches(&project_path)
}

#[tauri::command]
fn git_checkout_branch(
    state: tauri::State<AppState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.checkout_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_create_branch(
    state: tauri::State<AppState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.create_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_delete_branch(
    state: tauri::State<AppState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.delete_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_get_status(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitStatusEntry>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_status(&project_path)
}

#[tauri::command]
fn git_stage_file(
    state: tauri::State<AppState>,
    project_path: String,
    file_path: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stage_file(&project_path, &file_path)
}

#[tauri::command]
fn git_unstage_file(
    state: tauri::State<AppState>,
    project_path: String,
    file_path: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.unstage_file(&project_path, &file_path)
}

#[tauri::command]
fn git_stage_all(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stage_all(&project_path)
}

#[tauri::command]
fn git_commit(
    state: tauri::State<AppState>,
    project_path: String,
    message: String,
    author_name: String,
    author_email: String,
) -> AppResult<String> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state
        .git_service
        .commit(&project_path, &message, &author_name, &author_email)
}

#[tauri::command]
fn git_push(
    state: tauri::State<AppState>,
    project_path: String,
    remote_name: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.push(&project_path, &remote_name)
}

#[tauri::command]
fn git_pull(state: tauri::State<AppState>, project_path: String) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.pull(&project_path)
}

#[tauri::command]
fn git_clone(
    _state: tauri::State<AppState>,
    url: String,
    dest_path: String,
) -> AppResult<()> {
    crate::services::git_service::GitService::git_clone_remote(&url, &dest_path)
}

#[tauri::command]
fn git_get_remotes(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitRemote>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_remotes(&project_path)
}

#[tauri::command]
fn git_add_remote(
    state: tauri::State<AppState>,
    project_path: String,
    name: String,
    url: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.add_remote(&project_path, &name, &url)
}

#[tauri::command]
fn git_get_diff_summary(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitDiffEntry>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_diff_summary(&project_path)
}

#[tauri::command]
fn git_get_log(
    state: tauri::State<AppState>,
    project_path: String,
    max_count: usize,
) -> AppResult<Vec<crate::services::git_service::GitCommit>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_log(&project_path, max_count)
}

#[tauri::command]
fn git_stash(
    state: tauri::State<AppState>,
    project_path: String,
    message: Option<String>,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stash(&project_path, message.as_deref())
}

#[tauri::command]
fn git_stash_list(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitStashEntry>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stash_list(&project_path)
}

#[tauri::command]
fn git_stash_pop(
    state: tauri::State<AppState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stash_pop(&project_path, stash_index)
}

#[tauri::command]
fn git_stash_drop(
    state: tauri::State<AppState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stash_drop(&project_path, stash_index)
}

#[tauri::command]
fn git_stash_apply(
    state: tauri::State<AppState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.stash_apply(&project_path, stash_index)
}

#[tauri::command]
fn git_get_tags(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<Vec<crate::services::git_service::GitTag>> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_tags(&project_path)
}

#[tauri::command]
fn git_diff_file(
    state: tauri::State<AppState>,
    project_path: String,
    file_path: String,
) -> AppResult<String> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.diff_file(&project_path, &file_path)
}

#[tauri::command]
fn git_get_commit_diff(
    state: tauri::State<AppState>,
    project_path: String,
    commit_oid: String,
) -> AppResult<String> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state
        .git_service
        .get_commit_diff(&project_path, &commit_oid)
}

#[tauri::command]
fn git_get_branch_ahead_behind(
    state: tauri::State<AppState>,
    project_path: String,
    branch_name: String,
) -> AppResult<(usize, usize)> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state
        .git_service
        .get_branch_ahead_behind(&project_path, &branch_name)
}

#[tauri::command]
fn git_get_current_branch(
    state: tauri::State<AppState>,
    project_path: String,
) -> AppResult<String> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state.git_service.get_current_branch(&project_path)
}

// ===== SEARCH COMMANDS =====
#[tauri::command]
async fn search_files(
    state: tauri::State<'_, AppState>,
    directory: String,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    regex_mode: bool,
    max_results: Option<usize>,
) -> AppResult<serde_json::Value> {
    let _resolved = validate_project_path(&directory, &state)?;
    let (results, stats) = state
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

// ===== RUNTIME COMMANDS =====
#[tauri::command]
async fn get_installed_node_versions(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<String>> {
    state.runtime_service.get_installed_node_versions().await
}

#[tauri::command]
async fn get_installed_python_versions(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<String>> {
    state.runtime_service.get_installed_python_versions().await
}

#[tauri::command]
async fn install_node_version(
    state: tauri::State<'_, AppState>,
    version: String,
) -> AppResult<PathBuf> {
    state
        .runtime_service
        .install_node_version(&version, None)
        .await
}

#[tauri::command]
async fn install_python_version(
    state: tauri::State<'_, AppState>,
    version: String,
) -> AppResult<PathBuf> {
    state
        .runtime_service
        .install_python_version(&version, None)
        .await
}

#[tauri::command]
async fn uninstall_runtime_version(
    state: tauri::State<'_, AppState>,
    runtime: String,
    version: String,
) -> AppResult<()> {
    state
        .runtime_service
        .uninstall_version(&runtime, &version)
        .await
}

#[tauri::command]
async fn get_available_node_versions(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<String>> {
    state.runtime_service.get_available_node_versions().await
}

#[tauri::command]
async fn get_available_python_versions(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<String>> {
    state.runtime_service.get_available_python_versions().await
}

// ===== SETTINGS COMMANDS =====
#[tauri::command]
fn get_app_settings(
    state: tauri::State<AppState>,
) -> AppResult<crate::services::settings_service::AppSettings> {
    state.settings_service.get_app_settings()
}

#[tauri::command]
fn set_app_setting(
    state: tauri::State<AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    state.settings_service.set_app_setting(&key, &value)
}

#[tauri::command]
fn update_app_settings(
    state: tauri::State<AppState>,
    settings: HashMap<String, String>,
) -> AppResult<()> {
    state.settings_service.update_app_settings(&settings)
}

#[tauri::command]
fn get_project_settings(
    state: tauri::State<AppState>,
    project_id: String,
) -> AppResult<Option<crate::services::settings_service::ProjectSettings>> {
    state.settings_service.get_project_settings(&project_id)
}

#[tauri::command]
fn set_project_settings(
    state: tauri::State<AppState>,
    settings: crate::services::settings_service::ProjectSettings,
) -> AppResult<()> {
    state.settings_service.set_project_settings(&settings)
}

#[tauri::command]
fn delete_project_settings(
    state: tauri::State<AppState>,
    project_id: String,
) -> AppResult<()> {
    state.settings_service.delete_project_settings(&project_id)
}

// ===== BACKUP COMMANDS =====
#[tauri::command]
fn create_backup(
    state: tauri::State<AppState>,
    passphrase: String,
) -> AppResult<String> {
    let projects = state
        .projects_repo
        .get_all()
        .map_err(|e| AppError::Database(e.to_string()))?;
    let categories = state
        .categories_repo
        .get_all()
        .map_err(|e| AppError::Database(e.to_string()))?;
    let path = state
        .backup_service
        .create_backup(&projects, &categories, &passphrase, None)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn restore_backup(
    state: tauri::State<AppState>,
    zip_path: String,
    passphrase: String,
) -> AppResult<crate::utils::crypto::BackupPayload> {
    let zip = std::path::Path::new(&zip_path);
    state.backup_service.validate_backup_path(zip)?;
    state
        .backup_service
        .restore_backup(zip, &passphrase)
}

#[tauri::command]
fn list_backups(
    state: tauri::State<AppState>,
) -> AppResult<Vec<crate::services::backup_service::BackupInfo>> {
    state.backup_service.list_backups()
}

#[tauri::command]
fn delete_backup(
    state: tauri::State<AppState>,
    filename: String,
) -> AppResult<()> {
    state.backup_service.delete_backup(&filename)
}

// ===== UPDATE COMMANDS =====
#[tauri::command]
async fn check_for_updates(
    state: tauri::State<'_, AppState>,
) -> AppResult<Option<crate::services::update_service::UpdateInfo>> {
    state.update_service.check_for_updates().await
}

#[tauri::command]
async fn download_and_install_update(
    state: tauri::State<'_, AppState>,
) -> AppResult<()> {
    state.update_service.download_and_install().await
}

#[tauri::command]
async fn get_update_status(
    state: tauri::State<'_, AppState>,
) -> AppResult<crate::services::update_service::UpdateStatus> {
    Ok(state.update_service.get_status().await)
}

#[tauri::command]
async fn get_current_version(
    state: tauri::State<'_, AppState>,
) -> AppResult<String> {
    Ok(state.update_service.get_current_version().await)
}

// ===== FILE WATCHER COMMANDS =====
#[tauri::command]
async fn watch_directory(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> AppResult<()> {
    for p in &paths {
        let _resolved = validate_project_path(p, &state)?;
    }
    state
        .file_watcher
        .watch(paths)
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
async fn unwatch_all(state: tauri::State<'_, AppState>) -> AppResult<()> {
    state
        .file_watcher
        .unwatch_all()
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
async fn is_watching(state: tauri::State<'_, AppState>) -> AppResult<bool> {
    Ok(state.file_watcher.is_watching().await)
}

// ===== LSP COMMANDS =====
#[tauri::command]
async fn start_lsp_server(
    state: tauri::State<'_, AppState>,
    project_path: String,
    server_command: String,
    args: Vec<String>,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state
        .lsp_bridge
        .start_server(&project_path, &server_command, args)
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
async fn stop_lsp_server(
    state: tauri::State<'_, AppState>,
    project_path: String,
) -> AppResult<()> {
    let _resolved = validate_project_path(&project_path, &state)?;
    state
        .lsp_bridge
        .stop_server(&project_path)
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
async fn lsp_request(
    state: tauri::State<'_, AppState>,
    method: String,
    params: serde_json::Value,
) -> AppResult<serde_json::Value> {
    state
        .lsp_bridge
        .request(&method, params)
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
async fn start_lsp_proxy(
    state: tauri::State<'_, AppState>,
) -> AppResult<u16> {
    state
        .lsp_bridge
        .start_proxy()
        .await
        .map_err(AppError::Internal)
}

// ===== SYSTEM COMMANDS =====
#[tauri::command]
fn open_file(path: String) -> AppResult<()> {
    opener::open(&path).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn open_folder(path: String) -> AppResult<()> {
    opener::open(&path).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn open_external(url: String) -> AppResult<()> {
    opener::open(&url).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn get_platform_info() -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": if cfg!(target_os = "windows") { "windows" }
                  else if cfg!(target_os = "macos") { "macos" }
                  else { "linux" },
    }))
}

// ===== MEDIA COMMANDS =====
#[tauri::command]
fn validate_media_path(
    state: tauri::State<AppState>,
    path: String,
) -> AppResult<String> {
    let file_path = std::path::Path::new(&path);
    state
        .media_allowlist
        .validate_media_path(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| AppError::MediaAllowlist(e.to_string()))
}

// ===== TRAY COMMANDS =====
#[tauri::command]
async fn refresh_tray_menu(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let projects = state
        .projects_repo
        .get_all()
        .map_err(|e| AppError::Database(e.to_string()))?;
    tray::rebuild_tray_with_projects(&app, projects)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

// ===== DEBUG COMMANDS =====
#[tauri::command]
fn set_debug_mode(state: tauri::State<AppState>, enabled: bool) -> AppResult<()> {
    state.debug_mode.store(enabled, Ordering::SeqCst);
    // TODO: dynamically change log level between INFO and DEBUG
    Ok(())
}

#[tauri::command]
fn get_debug_mode(state: tauri::State<AppState>) -> bool {
    state.debug_mode.load(Ordering::SeqCst)
}

// ===== LOG COMMANDS =====
#[tauri::command]
async fn get_logs(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<crate::services::log_store::LogEntry>> {
    Ok(state.log_store.get_history(&project_id).await)
}

#[tauri::command]
async fn get_all_logs(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<crate::services::log_store::LogBatch>> {
    Ok(state.log_store.drain_batch().await)
}

#[tauri::command]
async fn clear_logs(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    state.log_store.clear_history(&project_id).await;
    Ok(())
}

// ===== TUNNEL LOG EXPORT =====
#[tauri::command]
async fn export_tunnel_logs_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<String> {
    let logs = state.tunnel_manager.get_logs(&project_id).await;
    Ok(serde_json::to_string_pretty(&logs).unwrap_or_default())
}

#[tauri::command]
async fn export_tunnel_logs_all() -> AppResult<String> {
    Ok("{}".to_string())
}

// ===== CONSOLE LOG EXPORT =====
#[tauri::command]
async fn export_console_logs_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> AppResult<String> {
    let logs = state.log_store.get_history(&project_id).await;
    Ok(serde_json::to_string_pretty(&logs).unwrap_or_default())
}

#[tauri::command]
async fn export_console_logs_all() -> AppResult<String> {
    Ok("{}".to_string())
}

// ===== WINDOW MANAGEMENT COMMANDS =====
#[tauri::command]
fn close_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        w.close()
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
fn minimize_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        w.minimize()
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
fn toggle_maximize(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_maximized().unwrap_or(false) {
            w.unmaximize()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            w.maximize()
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(())
}

// ===== DIALOG COMMANDS =====
#[tauri::command]
async fn select_directory() -> AppResult<Option<String>> {
    let picker = rfd::AsyncFileDialog::new().pick_folder().await;
    Ok(picker.map(|p| p.path().to_string_lossy().to_string()))
}

#[tauri::command]
async fn select_file() -> AppResult<Option<String>> {
    let picker = rfd::AsyncFileDialog::new().pick_file().await;
    Ok(picker.map(|p| p.path().to_string_lossy().to_string()))
}

// ===== SETTINGS GET (JSON) =====
#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppResult<serde_json::Value> {
    let s = state.settings_service.get_app_settings()?;
    Ok(serde_json::to_value(s).unwrap_or_default())
}

// ===== PATH JOIN UTILITY =====
#[tauri::command]
fn join_path(base: String, parts: Vec<String>) -> String {
    let mut path = std::path::PathBuf::from(base);
    for p in parts {
        path.push(p);
    }
    path.to_string_lossy().to_string()
}

// Helper trait for pipe
trait Pipe<T> {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R;
}

impl<T> Pipe<T> for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        f(self)
    }
}
