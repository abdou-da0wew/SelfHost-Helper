use std::collections::HashSet;
use std::path::Path;
use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::utils::ignore_patterns::{is_path_ignored, ripgrep_exclude_globs};
use crate::utils::sidecar::resolve_sidecar;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub text: String,
    pub match_start: u32,
    pub match_end: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchStats {
    pub total_matches: usize,
    pub files_with_matches: usize,
    pub duration_ms: u64,
}

pub struct SearchService {
    app_handle: tauri::AppHandle,
}

impl SearchService {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn search(
        &self,
        directory: &str,
        query: &str,
        case_sensitive: bool,
        whole_word: bool,
        regex_mode: bool,
        max_results: usize,
    ) -> AppResult<(Vec<SearchResult>, SearchStats)> {
        let start = std::time::Instant::now();

        let dir_path = std::path::Path::new(directory);
        if !dir_path.is_dir() {
            return Err(AppError::Search("Invalid search directory".into()));
        }
        if query.is_empty() {
            return Err(AppError::Search("Search query cannot be empty".into()));
        }

        if regex_mode {
            let pat = query.to_string();
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::task::spawn_blocking(move || {
                    regex::RegexBuilder::new(&pat)
                        .size_limit(1_048_576)
                        .build()
                }),
            )
            .await
            .map_err(|_| AppError::Search("Regex pattern too complex".into()))?
            .map_err(|_| AppError::Search("Regex compilation failed".into()))?
            .map_err(|e| AppError::Search(format!("Invalid regex: {}", e)))?;
        }

        let mut args = vec![
            "--json".to_string(),
            "--line-number".to_string(),
            "--column-number".to_string(),
            "--max-count".to_string(),
            (max_results * 2).to_string(),
            "--hidden".to_string(),
        ];

        if !case_sensitive {
            args.push("--ignore-case".to_string());
        }
        if whole_word {
            args.push("--word-regexp".to_string());
        }
        if !regex_mode {
            args.push("--fixed-strings".to_string());
        }

        for glob in ripgrep_exclude_globs() {
            args.push("--glob".to_string());
            args.push(glob);
        }
        args.push(query.to_string());
        args.push(directory.to_string());

        let rg_path = resolve_sidecar(&self.app_handle, "rg")?;

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            Command::new(rg_path).args(&args).output(),
        )
        .await
        .map_err(|_| AppError::Search("Search timed out after 120s".into()))?
        .map_err(|e| {
            AppError::Search(format!(
                "Failed to run rg: {}. Ensure ripgrep is installed.",
                e
            ))
        })?;

        if !output.status.success() && output.stdout.is_empty() {
            return Ok((
                Vec::new(),
                SearchStats {
                    total_matches: 0,
                    files_with_matches: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        let mut files_set = HashSet::new();

        for line in stdout.lines() {
            if line.is_empty() {
                continue;
            }
            if results.len() >= max_results {
                break;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(submatches) = json.get("submatches").and_then(|s| s.as_array()) {
                    if let Some(first_match) = submatches.first() {
                        let match_text = first_match
                            .get("text")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let match_start = first_match
                            .get("start")
                            .and_then(|s| s.as_u64())
                            .unwrap_or(0) as u32;
                        let match_end = first_match
                            .get("end")
                            .and_then(|s| s.as_u64())
                            .unwrap_or(0) as u32;
                        let path = json
                            .get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let line_number = json
                            .get("line_number")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(0) as u32;
                        files_set.insert(path.to_string());
                        results.push(SearchResult {
                            file: path.to_string(),
                            line: line_number,
                            column: match_start + 1,
                            text: match_text.to_string(),
                            match_start,
                            match_end,
                        });
                    }
                } else if let Some(data) = json.get("data").and_then(|d| d.as_str()) {
                    let path = json
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or("");
                    let line_number = json
                        .get("line_number")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(0) as u32;
                    files_set.insert(path.to_string());
                    results.push(SearchResult {
                        file: path.to_string(),
                        line: line_number,
                        column: 1,
                        text: data.to_string(),
                        match_start: 0,
                        match_end: data.len() as u32,
                    });
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let filtered: Vec<SearchResult> = results
            .into_iter()
            .filter(|r| !is_path_ignored(Path::new(&r.file)))
            .collect();
        let total = filtered.len();
        Ok((
            filtered,
            SearchStats {
                total_matches: total,
                files_with_matches: files_set.len(),
                duration_ms: duration,
            },
        ))
    }
}
