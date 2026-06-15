use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::AppResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectSettings {
    pub project_id: String,
    pub runtime: Option<String>,
    pub node_version: Option<String>,
    pub python_version: Option<String>,
    pub install_date: Option<String>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub clear_logs_before_start: Option<bool>,
    pub start_maximized: Option<bool>,
    pub dev_mode: Option<bool>,
    pub external_media_allowed_dirs: Option<String>,
    pub default_project_path: Option<String>,
    pub editor_font_size: Option<i32>,
    pub editor_tab_size: Option<i32>,
    pub editor_theme: Option<String>,
}

pub struct SettingsService {
    db: Arc<Mutex<Connection>>,
}

impl SettingsService {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    pub fn init_tables(&self) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS project_settings (
                project_id   TEXT PRIMARY KEY NOT NULL,
                runtime      TEXT,
                node_version TEXT,
                python_version TEXT,
                install_date TEXT,
                last_used    TEXT
            );",
        )?;
        Ok(())
    }

    pub fn get_app_settings(&self) -> AppResult<AppSettings> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = AppSettings::default();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "clearLogsBeforeStart" => settings.clear_logs_before_start = value.parse().ok(),
                "startMaximized" => settings.start_maximized = value.parse().ok(),
                "devMode" => settings.dev_mode = value.parse().ok(),
                "externalMediaAllowedDirs" => settings.external_media_allowed_dirs = Some(value),
                "defaultProjectPath" => settings.default_project_path = Some(value),
                "editorFontSize" => settings.editor_font_size = value.parse().ok(),
                "editorTabSize" => settings.editor_tab_size = value.parse().ok(),
                "editorTheme" => settings.editor_theme = Some(value),
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn update_app_settings(&self, updates: &HashMap<String, String>) -> AppResult<()> {
        for (key, value) in updates {
            self.set_app_setting(key, value)?;
        }
        Ok(())
    }

    pub fn get_project_settings(&self, project_id: &str) -> AppResult<Option<ProjectSettings>> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT project_id, runtime, node_version, python_version, install_date, last_used
             FROM project_settings WHERE project_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![project_id], |row| {
            Ok(ProjectSettings {
                project_id: row.get(0)?,
                runtime: row.get(1)?,
                node_version: row.get(2)?,
                python_version: row.get(3)?,
                install_date: row.get(4)?,
                last_used: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn set_project_settings(&self, settings: &ProjectSettings) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO project_settings (project_id, runtime, node_version, python_version, install_date, last_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                settings.project_id,
                settings.runtime,
                settings.node_version,
                settings.python_version,
                settings.install_date,
                settings.last_used,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project_settings(&self, project_id: &str) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        conn.execute(
            "DELETE FROM project_settings WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }
}
