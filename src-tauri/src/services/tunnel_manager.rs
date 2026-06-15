use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, RwLock};

use crate::error::{AppError, AppResult};
use crate::utils::sidecar::resolve_sidecar;

struct TunnelInstance {
    #[allow(dead_code)]
    project_id: String,
    port: u16,
    #[allow(dead_code)]
    mode: String,
    url: Option<String>,
    child: Option<Child>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TunnelLogEntry {
    pub project_id: String,
    pub message: String,
    pub r#type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TunnelStatus {
    pub project_id: String,
    pub status: String,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub error: Option<String>,
}

pub struct TunnelManager {
    running: Arc<RwLock<HashMap<String, TunnelInstance>>>,
    logs: Arc<RwLock<HashMap<String, Vec<TunnelLogEntry>>>>,
    event_tx: mpsc::Sender<TunnelStatus>,
    app_handle: tauri::AppHandle,
}

impl TunnelManager {
    pub fn new(app_handle: tauri::AppHandle) -> (Self, mpsc::Receiver<TunnelStatus>) {
        let (tx, rx) = mpsc::channel(64);
        (
            Self {
                running: Arc::new(RwLock::new(HashMap::new())),
                logs: Arc::new(RwLock::new(HashMap::new())),
                event_tx: tx,
                app_handle,
            },
            rx,
        )
    }

    pub async fn start_tunnel(
        &self,
        project_id: String,
        mode: String,
        port: u16,
        token: Option<String>,
        config: Option<HashMap<String, String>>,
    ) -> AppResult<()> {
        if port == 0 {
            return Err(AppError::Validation("Invalid port".into()));
        }
        if project_id.is_empty() || project_id.len() > 256 {
            return Err(AppError::Validation("Invalid project ID".into()));
        }

        {
            let running = self.running.read().await;
            if running.contains_key(&project_id) {
                return Err(AppError::Validation("Tunnel already running".into()));
            }
        }

        self.send_status(&project_id, "connecting", None, None, None)
            .await;

        let cloudflared_path = resolve_sidecar(&self.app_handle, "cloudflared")?;

        let mut args: Vec<String> = Vec::new();
        if mode == "authenticated" {
            let tok = token.ok_or_else(|| AppError::Validation("Token required".into()))?;
            if tok.is_empty() || tok.len() > 2048 {
                return Err(AppError::Validation("Invalid token".into()));
            }
            args.push("tunnel".to_string());
            args.push("run".to_string());
            args.push("--url".to_string());
            args.push(format!("http://localhost:{}", port));
            args.push("--token".to_string());
            args.push(tok);

            if let Some(cfg) = &config {
                if let Some(proto) = cfg.get("protocol") {
                    if !proto.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return Err(AppError::Validation(
                            "Invalid protocol value".into(),
                        ));
                    }
                    args.push("--protocol".to_string());
                    args.push(proto.clone());
                }
                if let Some(true_str) = cfg.get("noTLSVerify") {
                    if true_str == "true" {
                        args.push("--no-tls-verify".to_string());
                    }
                }
                if let Some(host) = cfg.get("httpHostHeader") {
                    if host.is_empty() || host.len() > 253 {
                        return Err(AppError::Validation(
                            "Invalid HTTP host header".into(),
                        ));
                    }
                    args.push("--http-host-header".to_string());
                    args.push(host.clone());
                }
            }
        } else {
            args.push("tunnel".to_string());
            args.push("--url".to_string());
            args.push(format!("http://localhost:{}", port));
        }

        let child = Command::new(cloudflared_path)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AppError::Process(format!("Failed to spawn cloudflared: {}", e)))?;

        let inst = TunnelInstance {
            project_id: project_id.clone(),
            port,
            mode,
            url: None,
            child: Some(child),
        };

        {
            let mut running = self.running.write().await;
            running.insert(project_id.clone(), inst);
        }

        let running_ref = self.running.clone();
        let event_tx = self.event_tx.clone();
        let pid_clone = project_id.clone();

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let mut child_died = false;
            if let Some(inst) = running_ref.write().await.get_mut(&pid_clone) {
                if let Some(ref mut child) = inst.child {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let _ = event_tx
                                .send(TunnelStatus {
                                    project_id: pid_clone.clone(),
                                    status: "error".into(),
                                    url: None,
                                    port: None,
                                    error: Some(format!("cloudflared exited with {}", status)),
                                })
                                .await;
                            running_ref.write().await.remove(&pid_clone);
                            child_died = true;
                        }
                        _ => {}
                    }
                }
            }

            if !child_died {
                let _ = event_tx
                    .send(TunnelStatus {
                        project_id: pid_clone.clone(),
                        status: "running".into(),
                        url: Some(format!("https://{}.trycloudflare.com", pid_clone)),
                        port: Some(port),
                        error: None,
                    })
                    .await;
            }
        });

        Ok(())
    }

    pub async fn stop_tunnel(&self, project_id: &str) -> AppResult<()> {
        let (child_to_kill, exists) = {
            let mut running = self.running.write().await;
            match running.get_mut(project_id) {
                Some(inst) => {
                    let child = inst.child.take();
                    running.remove(project_id);
                    (child, true)
                }
                None => (None, false),
            }
        };

        if !exists {
            return Err(AppError::NotFound("No tunnel running".into()));
        }

        if let Some(mut child) = child_to_kill {
            child
                .kill()
                .await
                .map_err(|e| AppError::Process(e.to_string()))?;
            let _ = child.wait().await;
        }

        self.send_status(project_id, "stopped", None, None, None)
            .await;
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut running = self.running.write().await;
        for (_, mut inst) in running.drain() {
            if let Some(ref mut child) = inst.child {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }
    }

    pub async fn get_status(&self, project_id: &str) -> Option<TunnelStatus> {
        let running = self.running.read().await;
        running.get(project_id).map(|inst| TunnelStatus {
            project_id: project_id.to_string(),
            status: if inst.url.is_some() {
                "running".into()
            } else {
                "connecting".into()
            },
            url: inst.url.clone(),
            port: Some(inst.port),
            error: None,
        })
    }

    pub async fn get_logs(&self, project_id: &str) -> Vec<TunnelLogEntry> {
        self.logs
            .read()
            .await
            .get(project_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn get_all_logs(&self) -> HashMap<String, Vec<TunnelLogEntry>> {
        self.logs.read().await.clone()
    }

    pub async fn clear_logs(&self, project_id: &str) {
        self.logs.write().await.remove(project_id);
    }

    async fn send_status(
        &self,
        project_id: &str,
        status: &str,
        url: Option<String>,
        port: Option<u16>,
        error: Option<String>,
    ) {
        let _ = self
            .event_tx
            .send(TunnelStatus {
                project_id: project_id.to_string(),
                status: status.into(),
                url,
                port,
                error,
            })
            .await;
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        if let Ok(mut running) = self.running.try_write() {
            for (_key, mut inst) in running.drain() {
                if let Some(ref mut child) = inst.child {
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        if let Some(pid) = child.id() {
                            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                        }
                    }
                    #[cfg(windows)]
                    {
                        if let Some(pid) = child.id() {
                            let _ = std::process::Command::new("taskkill")
                                .args(&["/F", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
            }
        }
    }
}
