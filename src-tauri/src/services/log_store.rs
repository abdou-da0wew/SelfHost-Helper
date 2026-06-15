use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_HISTORY_PER_KEY: usize = 100;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub key: String,
    pub message: String,
    pub r#type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LogBatch {
    pub key: String,
    pub entries: Vec<LogEntry>,
}

pub struct LogStore {
    history: Arc<RwLock<HashMap<String, Vec<LogEntry>>>>,
    batch_queue: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn push(&self, key: String, entry: LogEntry) -> Vec<LogBatch> {
        {
            let mut history = self.history.write().await;
            let hist = history.entry(key).or_default();
            hist.push(entry.clone());
            Self::trim_history(hist);
        }

        let mut queue = self.batch_queue.write().await;
        queue.push(entry);

        let mut batches = Vec::new();
        if queue.len() >= 50 {
            let taken: Vec<LogEntry> = queue.drain(..).collect();
            let mut by_key: HashMap<String, Vec<LogEntry>> = HashMap::new();
            for e in taken {
                by_key.entry(e.key.clone()).or_default().push(e);
            }
            for (k, entries) in by_key {
                batches.push(LogBatch { key: k, entries });
            }
        }
        batches
    }

    pub async fn drain_batch(&self) -> Vec<LogBatch> {
        let mut queue = self.batch_queue.write().await;
        if queue.is_empty() {
            return Vec::new();
        }
        let taken: Vec<LogEntry> = queue.drain(..).collect();
        let mut by_key: HashMap<String, Vec<LogEntry>> = HashMap::new();
        for e in taken {
            by_key.entry(e.key.clone()).or_default().push(e);
        }
        by_key
            .into_iter()
            .map(|(k, entries)| LogBatch { key: k, entries })
            .collect()
    }

    pub async fn get_history(&self, key: &str) -> Vec<LogEntry> {
        self.history
            .read()
            .await
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn clear_history(&self, key: &str) {
        self.history.write().await.remove(key);
    }

    pub async fn clear_all(&self) {
        self.history.write().await.clear();
    }

    fn trim_history(history: &mut Vec<LogEntry>) {
        if history.len() > MAX_HISTORY_PER_KEY {
            history.drain(..history.len() - MAX_HISTORY_PER_KEY);
        }
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}
