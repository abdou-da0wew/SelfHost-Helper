use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

pub struct LspBridge {
    sessions: Arc<RwLock<HashMap<String, LspSession>>>,
    #[allow(dead_code)]
    proxy_port: Arc<RwLock<Option<u16>>>,
}

struct LspSession {
    #[allow(dead_code)]
    child: Child,
    stdin_tx: mpsc::Sender<Vec<u8>>,
    #[allow(dead_code)]
    pending: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    next_id: Arc<RwLock<u64>>,
}

impl LspBridge {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            proxy_port: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_proxy(&self) -> Result<u16, String> {
        let mut port_guard = self.proxy_port.write().await;
        if let Some(port) = *port_guard {
            return Ok(port);
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("Failed to bind proxy: {}", e))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get addr: {}", e))?
            .port();

        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            loop {
                let (socket, _) = match listener.accept().await {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let sessions = sessions.clone();
                tokio::spawn(async move {
                    let ws_stream = match tokio_tungstenite::accept_async(socket).await {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let (mut ws_sink, mut ws_stream) = ws_stream.split();
                    while let Some(msg) = ws_stream.next().await {
                        let msg = match msg {
                            Ok(m) => m,
                            Err(_) => break,
                        };
                        if msg.is_text() {
                            let text = msg.to_text().unwrap_or("");
                            if let Ok(req) = serde_json::from_str::<LspRequest>(text) {
                                let sessions = sessions.read().await;
                                let mut found_session = None;
                                for (_, session) in sessions.iter() {
                                    found_session = Some(session);
                                    break;
                                }
                                if let Some(session) = found_session {
                                    let result = session.request(&req.method, req.params).await;
                                    let response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": req.id,
                                        "result": result,
                                    });
                                    let _ = ws_sink
                                        .send(tokio_tungstenite::tungstenite::Message::Text(
                                            response.to_string(),
                                        ))
                                        .await;
                                }
                            }
                        }
                    }
                });
            }
        });

        *port_guard = Some(port);
        Ok(port)
    }

    pub async fn start_server(
        &self,
        project_path: &str,
        server_command: &str,
        args: Vec<String>,
    ) -> Result<(), String> {
        if server_command.is_empty() {
            return Err("Server command cannot be empty".into());
        }
        let path = std::path::Path::new(project_path);
        if !path.is_dir() {
            return Err("Invalid project path".into());
        }
        let mut child = Command::new(server_command)
            .args(&args)
            .current_dir(project_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start LSP server: {}", e))?;

        let stdin = child.stdin.take().ok_or("No stdin")?;
        let stdout = child.stdout.take().ok_or("No stdout")?;

        let (stdin_tx, mut stdin_rx) = mpsc::channel::<Vec<u8>>(32);
        let pending: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let next_id = Arc::new(RwLock::new(1u64));

        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(data) = stdin_rx.recv().await {
                let _ = stdin.write_all(&data).await;
            }
        });

        let _pending_for_read = pending.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut _content_length: Option<usize> = None;
            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    _content_length = None;
                } else if let Some(val) = line.strip_prefix("Content-Length: ") {
                    _content_length = val.trim().parse().ok();
                }
            }
        });

        let session = LspSession {
            child,
            stdin_tx,
            pending,
            next_id,
        };

        self.sessions
            .write()
            .await
            .insert(project_path.to_string(), session);
        Ok(())
    }

    pub async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .values()
            .next()
            .ok_or("No active LSP session")?;
        session.request(method, params).await
    }

    pub async fn stop_server(&self, project_path: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(project_path) {
            let _ = session.child.kill().await;
        }
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut sessions = self.sessions.write().await;
        for (_, mut session) in sessions.drain() {
            let _ = session.child.kill().await;
        }
    }
}

impl LspSession {
    async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut id_guard = self.next_id.write().await;
        let id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let (tx, rx) = oneshot::channel();
        self.pending.write().await.insert(id, tx);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body = request.to_string();
        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        self.stdin_tx
            .send(message.into_bytes())
            .await
            .map_err(|e| format!("Failed to send: {}", e))?;

        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(_)) => Err("Channel closed".into()),
            Err(_) => {
                self.pending.write().await.remove(&id);
                Err("Request timed out".into())
            }
        }
    }
}
