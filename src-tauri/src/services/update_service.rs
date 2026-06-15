use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::{RwLock, Semaphore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub status: String,
    pub update: Option<UpdateInfo>,
    pub error: Option<String>,
}

pub struct UpdateService {
    app_handle: tauri::AppHandle,
    status: Arc<RwLock<UpdateStatus>>,
    check_interval_secs: u64,
    check_guard: Arc<Semaphore>,
    cancel_token: Arc<tokio::sync::watch::Sender<bool>>,
}

impl UpdateService {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let (cancel_tx, _) = tokio::sync::watch::channel(false);
        Self {
            app_handle,
            status: Arc::new(RwLock::new(UpdateStatus {
                status: "idle".into(),
                update: None,
                error: None,
            })),
            check_interval_secs: 86400,
            check_guard: Arc::new(Semaphore::new(1)),
            cancel_token: Arc::new(cancel_tx),
        }
    }

    pub async fn check_for_updates(&self) -> AppResult<Option<UpdateInfo>> {
        let _permit = self
            .check_guard
            .acquire()
            .await
            .map_err(|_| AppError::Internal("Update check already in progress".into()))?;

        *self.status.write().await = UpdateStatus {
            status: "checking".into(),
            update: None,
            error: None,
        };

        let updater = self
            .app_handle
            .updater()
            .map_err(|e| AppError::Internal(format!("Updater init failed: {}", e)))?;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            updater.check(),
        )
        .await
        .map_err(|_| AppError::Internal("Update check timed out".into()))?;

        match result {
            Ok(Some(update)) => {
                let info = UpdateInfo {
                    version: update.version.clone(),
                    date: update.date.map(|d| d.to_string()),
                    notes: update.body.clone(),
                };
                *self.status.write().await = UpdateStatus {
                    status: "available".into(),
                    update: Some(info.clone()),
                    error: None,
                };
                Ok(Some(info))
            }
            Ok(None) => {
                *self.status.write().await = UpdateStatus {
                    status: "up_to_date".into(),
                    update: None,
                    error: None,
                };
                Ok(None)
            }
            Err(e) => {
                let err_msg = format!("Failed to check for updates: {}", e);
                *self.status.write().await = UpdateStatus {
                    status: "error".into(),
                    update: None,
                    error: Some(err_msg.clone()),
                };
                Err(AppError::Internal(err_msg))
            }
        }
    }

    pub async fn download_and_install(&self) -> AppResult<()> {
        let current_update = self.status.read().await.update.clone();
        *self.status.write().await = UpdateStatus {
            status: "downloading".into(),
            update: current_update,
            error: None,
        };

        let updater = self
            .app_handle
            .updater()
            .map_err(|e| AppError::Internal(format!("Update check failed: {}", e)))?;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            updater.check(),
        )
        .await
        .map_err(|_| AppError::Internal("Update check timed out".into()))?;

        match result {
            Ok(Some(update)) => tokio::time::timeout(
                std::time::Duration::from_secs(600),
                update.download_and_install(|_, _| {}, || {}),
            )
            .await
            .map_err(|_| AppError::Internal("Download timed out".into()))?
            .map_err(|e| AppError::Internal(format!("Update install failed: {}", e))),
            Ok(None) => Err(AppError::Validation("No update available".into())),
            Err(e) => Err(AppError::Internal(format!(
                "Update check failed: {}",
                e
            ))),
        }
    }

    pub async fn get_status(&self) -> UpdateStatus {
        self.status.read().await.clone()
    }

    pub async fn get_current_version(&self) -> String {
        self.app_handle.package_info().version.to_string()
    }

    pub fn start_periodic_check(&self) {
        let status = self.status.clone();
        let handle = self.app_handle.clone();
        let interval = self.check_interval_secs;
        let mut cancel_rx = self.cancel_token.subscribe();
        tokio::spawn(async move {
            let mut interval_timer =
                tokio::time::interval(tokio::time::Duration::from_secs(interval));
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {}
                    _ = cancel_rx.changed() => {
                        break;
                    }
                }
                if let Ok(updater) = handle.updater() {
                    let result = tokio::time::timeout(
                        std::time::Duration::from_secs(60),
                        updater.check(),
                    )
                    .await;
                    if let Ok(Ok(Some(update))) = result {
                        let info = UpdateInfo {
                            version: update.version.clone(),
                            date: update.date.map(|d| d.to_string()),
                            notes: update.body.clone(),
                        };
                        *status.write().await = UpdateStatus {
                            status: "available".into(),
                            update: Some(info),
                            error: None,
                        };
                        let _ = handle.emit("update-available", &status.read().await.clone());
                    }
                }
            }
        });
    }
}

impl Drop for UpdateService {
    fn drop(&mut self) {
        let _ = self.cancel_token.send(true);
    }
}
