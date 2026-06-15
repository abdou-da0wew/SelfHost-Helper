use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub kind: String,
    pub paths: Vec<String>,
    pub timestamp: String,
}

pub struct FileWatcher {
    watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
    event_tx: mpsc::Sender<FileChangeEvent>,
    watched_dirs: Arc<RwLock<Vec<PathBuf>>>,
}

impl FileWatcher {
    pub fn new(event_tx: mpsc::Sender<FileChangeEvent>) -> Self {
        Self {
            watcher: Arc::new(RwLock::new(None)),
            event_tx,
            watched_dirs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn watch(&self, paths: Vec<String>) -> Result<(), String> {
        if self.watcher.read().await.is_some() {
            self.unwatch_all().await?;
        }

        let mut watcher_guard = self.watcher.write().await;

        let tx = self.event_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    let kind = match event.kind {
                        EventKind::Create(_) => "created",
                        EventKind::Modify(_) => "modified",
                        EventKind::Remove(_) => "removed",
                        _ => "other",
                    };
                    let paths: Vec<String> = event
                        .paths
                        .iter()
                        .filter_map(|p| p.to_str().map(|s| s.to_string()))
                        .collect();
                    if !paths.is_empty() {
                        let _ = tx.try_send(FileChangeEvent {
                            kind: kind.to_string(),
                            paths,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        });
                    }
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        for path_str in &paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                watcher
                    .watch(&path, RecursiveMode::Recursive)
                    .map_err(|e| format!("Failed to watch {}: {}", path_str, e))?;
                self.watched_dirs.write().await.push(path);
            }
        }

        *watcher_guard = Some(watcher);
        Ok(())
    }

    pub async fn unwatch_all(&self) -> Result<(), String> {
        let mut watcher_guard = self.watcher.write().await;
        if let Some(mut watcher) = watcher_guard.take() {
            for dir in self.watched_dirs.write().await.drain(..) {
                if let Err(e) = watcher.unwatch(&dir) {
                    eprintln!("[file_watcher] failed to unwatch {:?}: {}", dir, e);
                }
            }
        }
        Ok(())
    }

    pub async fn is_watching(&self) -> bool {
        self.watcher.read().await.is_some()
    }
}
