use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

pub(crate) struct LspSession {
    child: Child,
    stdin_tx: mpsc::Sender<Vec<u8>>,
    pending: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    next_id: Arc<RwLock<u64>>,
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

pub struct LspSessionManager {
    sessions: Arc<RwLock<HashMap<String, LspSession>>>,
}

impl LspSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) fn sessions(&self) -> Arc<RwLock<HashMap<String, LspSession>>> {
        self.sessions.clone()
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

        // Kill existing session for this project path if any
        if let Some(mut old) = self.sessions.write().await.remove(project_path) {
            let _ = old.child.kill().await;
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

        let pending_for_read = pending.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                // Read LSP headers (Content-Length: N)
                let mut content_length: Option<usize> = None;
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(0) => return,
                        Ok(_) => {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                break;
                            }
                            if let Some(val) = trimmed.strip_prefix("Content-Length: ") {
                                content_length = val.trim().parse().ok();
                            }
                        }
                        Err(_) => return,
                    }
                }

                // Read and parse the JSON body
                if let Some(len) = content_length {
                    let mut body = vec![0u8; len];
                    if reader.read_exact(&mut body).await.is_err() {
                        return;
                    }
                    if let Ok(response) =
                        serde_json::from_slice::<serde_json::Value>(&body)
                    {
                        if let Some(id) = response.get("id").and_then(|i| i.as_u64()) {
                            let mut pending = pending_for_read.write().await;
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(response);
                            }
                        }
                    }
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
        project_path: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(project_path)
            .ok_or_else(|| format!("No active LSP session for {}", project_path))?;
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

pub struct LspProxy {
    sessions: Arc<RwLock<HashMap<String, LspSession>>>,
    proxy_port: Arc<RwLock<Option<u16>>>,
}

impl LspProxy {
    pub(crate) fn new(sessions: Arc<RwLock<HashMap<String, LspSession>>>) -> Self {
        Self {
            sessions,
            proxy_port: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<u16, String> {
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
                                if let Some((_, session)) = sessions.iter().next() {
                                    let result =
                                        session.request(&req.method, req.params).await;
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
}
