use rusqlite::{params, Connection};
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::error::AppResult;

pub struct ProjectsRepo {
    db: Arc<Mutex<Connection>>,
}

impl ProjectsRepo {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> AppResult<Vec<Value>> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name, p.path, p.category_id, p.tags, p.icon, p.created_at, p.updated_at,
                    c.name as category_name, c.color as category_color
             FROM projects p
             LEFT JOIN categories c ON p.category_id = c.id
             ORDER BY p.name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let path: String = row.get(2)?;
            let category_id: Option<i64> = row.get(3)?;
            let tags: Option<String> = row.get(4)?;
            let icon: Option<String> = row.get(5)?;
            let created_at: Option<String> = row.get(6)?;
            let updated_at: Option<String> = row.get(7)?;
            let category_name: Option<String> = row.get(8)?;
            let category_color: Option<String> = row.get(9)?;
            Ok(serde_json::json!({
                "id": id,
                "name": name,
                "path": path,
                "category_id": category_id,
                "tags": tags,
                "icon": icon,
                "created_at": created_at,
                "updated_at": updated_at,
                "category_name": category_name,
                "category_color": category_color,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_by_id(&self, id: i64) -> AppResult<Option<Value>> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, path, category_id, tags, icon, created_at, updated_at
             FROM projects WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "path": row.get::<_, String>(2)?,
                "category_id": row.get::<_, Option<i64>>(3)?,
                "tags": row.get::<_, Option<String>>(4)?,
                "icon": row.get::<_, Option<String>>(5)?,
                "created_at": row.get::<_, Option<String>>(6)?,
                "updated_at": row.get::<_, Option<String>>(7)?,
            }))
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn create(
        &self,
        name: &str,
        path: &str,
        category_id: Option<i64>,
        tags: Option<&str>,
    ) -> AppResult<i64> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT INTO projects (name, path, category_id, tags) VALUES (?1, ?2, ?3, ?4)",
            params![name, path, category_id, tags],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(
        &self,
        id: i64,
        name: Option<&str>,
        path: Option<&str>,
        category_id: Option<i64>,
        tags: Option<&str>,
        icon: Option<&str>,
    ) -> AppResult<()> {
        let current = match self.get_by_id(id)? {
            Some(c) => c,
            None => {
                return Err(crate::error::AppError::NotFound(format!(
                    "Project {} not found",
                    id
                )))
            }
        };
        let new_name = name.unwrap_or(current["name"].as_str().unwrap_or(""));
        let new_path = path.unwrap_or(current["path"].as_str().unwrap_or(""));
        let new_tags = tags.or_else(|| current["tags"].as_str());
        let new_icon = icon.or_else(|| current["icon"].as_str());
        let new_category_id = if category_id.is_some() {
            category_id
        } else {
            current["category_id"].as_i64()
        };
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "UPDATE projects SET name = ?1, path = ?2, category_id = ?3, tags = ?4, icon = ?5, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?6",
            params![new_name, new_path, new_category_id, new_tags, new_icon, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn count(&self) -> AppResult<i64> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn search(&self, query: &str) -> AppResult<Vec<Value>> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, path, category_id, tags, icon
             FROM projects
             WHERE name LIKE ?1 OR path LIKE ?1 OR tags LIKE ?1
             ORDER BY name ASC",
        )?;
        let pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "path": row.get::<_, String>(2)?,
                "category_id": row.get::<_, Option<i64>>(3)?,
                "tags": row.get::<_, Option<String>>(4)?,
                "icon": row.get::<_, Option<String>>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

use crate::error::AppError;
