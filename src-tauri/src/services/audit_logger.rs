use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
    pub target: Option<String>,
    pub details: serde_json::Value,
    pub outcome: String,
    pub duration_ms: u64,
}

pub struct AuditLogger {
    log_path: PathBuf,
    buffer: Arc<std::sync::Mutex<Vec<AuditEntry>>>,
    flush_notify: Arc<tokio::sync::Notify>,
}

impl AuditLogger {
    pub fn new(log_path: PathBuf) -> Self {
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let buffer = Arc::new(std::sync::Mutex::new(Vec::new()));
        let flush_notify = Arc::new(tokio::sync::Notify::new());

        let logger = Self {
            log_path: log_path.clone(),
            buffer: buffer.clone(),
            flush_notify: flush_notify.clone(),
        };

        let buf = buffer.clone();
        let path = logger.log_path.clone();
        let notify = flush_notify.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify.notified() => {}
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {}
                }
                let entries = {
                    let mut guard = match buf.lock() {
                        Ok(g) => g,
                        Err(_) => continue,
                    };
                    if guard.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *guard)
                };
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    for entry in &entries {
                        if let Ok(line) = serde_json::to_string(entry) {
                            let _ = writeln!(file, "{}", line);
                        }
                    }
                }
            }
        });

        logger
    }

    pub fn log(&self, entry: AuditEntry) {
        let should_flush = {
            let mut guard = self.buffer.lock().unwrap();
            guard.push(entry);
            guard.len() >= 50
        };
        if should_flush {
            self.flush_notify.notify_one();
        }
    }

    pub fn flush(&self) {
        self.flush_notify.notify_one();
    }

    pub fn timed<T>(
        &self,
        operation: &'static str,
        f: impl FnOnce() -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let start = Instant::now();
        let result = f();
        self.log(AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: operation.to_string(),
            actor: "system".to_string(),
            target: None,
            details: serde_json::json!({}),
            outcome: if result.is_ok() {
                "success".to_string()
            } else {
                "error".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        });
        result
    }

    pub async fn timed_async<T>(
        &self,
        operation: &'static str,
        target: Option<String>,
        details: serde_json::Value,
        future: impl std::future::Future<Output = Result<T, AppError>>,
    ) -> Result<T, AppError> {
        let start = Instant::now();
        let result = future.await;
        self.log(AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: operation.to_string(),
            actor: "system".to_string(),
            target,
            details,
            outcome: if result.is_ok() {
                "success".to_string()
            } else {
                "error".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        });
        result
    }
}
