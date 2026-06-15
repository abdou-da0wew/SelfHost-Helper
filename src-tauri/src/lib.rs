mod commands;
mod db;
mod error;
mod native;
mod services;
mod utils;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Emitter, Manager};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::db::connection::establish_connection;
use crate::db::projects_repo::ProjectsRepo;
use crate::db::categories_repo::CategoriesRepo;
use crate::error::{AppError, AppResult};
use crate::native::tray;
use crate::services::audit_logger::AuditLogger;
use crate::services::backup_service::BackupService;
use crate::services::file_watcher::FileWatcher;
use crate::services::git_service::GitService;
use crate::services::lsp_bridge::{LspProxy, LspSessionManager};
use crate::services::log_store::LogStore;
use crate::services::runtime_service::RuntimeService;
use crate::services::search_service::SearchService;
use crate::services::settings_service::SettingsService;
use crate::services::tunnel_manager::TunnelManager;
use crate::services::update_service::UpdateService;
use crate::utils::media_allowlist::MediaAllowlist;
use crate::utils::path_security;

pub struct DbState {
    pub conn: Arc<std::sync::Mutex<rusqlite::Connection>>,
    pub projects_repo: Arc<ProjectsRepo>,
    pub categories_repo: Arc<CategoriesRepo>,
}

pub struct GitState {
    pub git_service: Arc<GitService>,
}

pub struct ServicesState {
    pub settings_service: Arc<SettingsService>,
    pub search_service: Arc<SearchService>,
    pub lsp_session_manager: Arc<LspSessionManager>,
    pub lsp_proxy: Arc<LspProxy>,
    pub runtime_service: Arc<RuntimeService>,
    pub file_watcher: Arc<FileWatcher>,
    pub backup_service: Arc<BackupService>,
    pub tunnel_manager: Arc<TunnelManager>,
    pub update_service: Arc<UpdateService>,
    pub log_store: Arc<LogStore>,
    pub audit_logger: Arc<AuditLogger>,
}

pub struct AppConfig {
    pub media_allowlist: Arc<MediaAllowlist>,
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
            // Restrict DB file permissions to owner-only (0600)
            let db = establish_connection(&db_path)?;
            #[cfg(unix)]
            if let Ok(()) = std::fs::set_permissions(&db_path, PermissionsExt::from_mode(0o600)) {
            }
            let projects_repo = Arc::new(ProjectsRepo::new(db.clone()));
            let categories_repo = Arc::new(CategoriesRepo::new(db.clone()));
            let settings_service = Arc::new(SettingsService::new(db.clone()));
            settings_service.init_tables().ok();
            let git_service = Arc::new(GitService::new());
            let search_service = Arc::new(SearchService::new(handle.clone()));
            let lsp_session_manager = Arc::new(LspSessionManager::new());
            let lsp_proxy = Arc::new(LspProxy::new(lsp_session_manager.sessions()));
            let runtime_service = Arc::new(RuntimeService::new());
            let media_allowlist = Arc::new(MediaAllowlist::new());
            let log_store = Arc::new(LogStore::new());
            let (tunnel_manager_raw, mut tunnel_rx) = TunnelManager::new(handle.clone());
            let tunnel_manager = Arc::new(tunnel_manager_raw);
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

            let audit_log_path = app_base.join("audit.log");
            let audit_logger = Arc::new(AuditLogger::new(audit_log_path));

            app.manage(DbState {
                conn: db.clone(),
                projects_repo: Arc::clone(&projects_repo),
                categories_repo: Arc::clone(&categories_repo),
            });
            app.manage(GitState {
                git_service: Arc::clone(&git_service),
            });
            app.manage(ServicesState {
                settings_service: Arc::clone(&settings_service),
                search_service: Arc::clone(&search_service),
                lsp_session_manager: Arc::clone(&lsp_session_manager),
                lsp_proxy: Arc::clone(&lsp_proxy),
                runtime_service: Arc::clone(&runtime_service),
                file_watcher: Arc::clone(&file_watcher),
                backup_service: Arc::clone(&backup_service),
                tunnel_manager: Arc::clone(&tunnel_manager),
                update_service: Arc::clone(&update_service),
                log_store: Arc::clone(&log_store),
                audit_logger: Arc::clone(&audit_logger),
            });
            app.manage(AppConfig {
                media_allowlist: Arc::clone(&media_allowlist),
                dev_mode: is_dev,
                debug_mode: Arc::new(AtomicBool::new(false)),
            });

            let handle_clone = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(change) = file_watcher_rx.recv().await {
                    let _ = handle_clone.emit("file-changed", &change);
                }
            });

            if let Err(e) = tray::create_tray(&handle) {
                log::warn!("Failed to create tray: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::get_projects,
            commands::projects::add_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::categories::get_categories,
            commands::categories::add_category,
            commands::categories::update_category,
            commands::categories::delete_category,
            commands::process::start_process,
            commands::process::stop_process,
            commands::process::restart_process,
            commands::process::get_process_status,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::get_tunnel_logs,
            commands::tunnel::clear_tunnel_logs,
            commands::tunnel::export_tunnel_logs_project,
            commands::tunnel::export_tunnel_logs_all,
            commands::git::git_get_branches,
            commands::git::git_checkout_branch,
            commands::git::git_create_branch,
            commands::git::git_delete_branch,
            commands::git::git_get_status,
            commands::git::git_stage_file,
            commands::git::git_unstage_file,
            commands::git::git_stage_all,
            commands::git::git_commit,
            commands::git::git_push,
            commands::git::git_pull,
            commands::git::git_clone,
            commands::git::git_get_remotes,
            commands::git::git_add_remote,
            commands::git::git_get_diff_summary,
            commands::git::git_get_log,
            commands::git::git_stash,
            commands::git::git_stash_list,
            commands::git::git_stash_pop,
            commands::git::git_stash_drop,
            commands::git::git_get_tags,
            commands::git::git_diff_file,
            commands::git::git_get_commit_diff,
            commands::git::git_get_branch_ahead_behind,
            commands::git::git_get_current_branch,
            commands::git::git_stash_apply,
            commands::search::search_files,
            commands::runtime::get_installed_node_versions,
            commands::runtime::get_installed_python_versions,
            commands::runtime::install_node_version,
            commands::runtime::install_python_version,
            commands::runtime::uninstall_runtime_version,
            commands::runtime::get_available_node_versions,
            commands::runtime::get_available_python_versions,
            commands::settings::get_app_settings,
            commands::settings::set_app_setting,
            commands::settings::update_app_settings,
            commands::settings::get_project_settings,
            commands::settings::set_project_settings,
            commands::settings::delete_project_settings,
            commands::settings::get_settings,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            commands::update::check_for_updates,
            commands::update::download_and_install_update,
            commands::update::get_update_status,
            commands::update::get_current_version,
            commands::file_watcher::watch_directory,
            commands::file_watcher::unwatch_all,
            commands::file_watcher::is_watching,
            commands::lsp::start_lsp_server,
            commands::lsp::stop_lsp_server,
            commands::lsp::lsp_request,
            commands::lsp::start_lsp_proxy,
            commands::system::open_file,
            commands::system::open_folder,
            commands::system::open_external,
            commands::system::get_platform_info,
            commands::system::set_debug_mode,
            commands::system::get_debug_mode,
            commands::system::get_logs,
            commands::system::get_all_logs,
            commands::system::clear_logs,
            commands::system::export_console_logs_project,
            commands::system::export_console_logs_all,
            commands::system::close_window,
            commands::system::minimize_window,
            commands::system::toggle_maximize,
            commands::system::select_directory,
            commands::system::select_file,
            commands::system::join_path,
            commands::media::validate_media_path,
            commands::tray::refresh_tray_menu,
            commands::system::set_log_level,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|handle, event| {
            if let tauri::RunEvent::Exit = event {
                let services = handle.state::<ServicesState>();
                let db_state = handle.state::<DbState>();
                let tunnel_manager = services.tunnel_manager.clone();
                let log_store = services.log_store.clone();
                let db = db_state.conn.clone();
                tauri::async_runtime::block_on(async {
                    tunnel_manager.stop_all().await;
                    log_store.clear_all().await;
                    drop(db);
                });
            }
        });
}

// ===== PATH VALIDATION =====
pub(crate) fn validate_project_path(path: &str, db_state: &DbState) -> AppResult<std::path::PathBuf> {
    let candidate = std::path::Path::new(path);
    let resolved = dunce::canonicalize(candidate)
        .map_err(|_| AppError::PathSecurity(format!("path does not exist or cannot be resolved: {}", path)))?;
    let projects = db_state
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

// Helper trait for pipe
pub(crate) trait Pipe<T> {
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
