use rusqlite::{params, Connection};
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::error::{AppError, AppResult};

pub struct CategoriesRepo {
    db: Arc<Mutex<Connection>>,
}

impl CategoriesRepo {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> AppResult<Vec<Value>> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM categories
             ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "color": row.get::<_, Option<String>>(2)?,
                "created_at": row.get::<_, Option<String>>(3)?,
                "updated_at": row.get::<_, Option<String>>(4)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_by_id(&self, id: i64) -> AppResult<Option<Value>> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM categories WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "color": row.get::<_, Option<String>>(2)?,
                "created_at": row.get::<_, Option<String>>(3)?,
                "updated_at": row.get::<_, Option<String>>(4)?,
            }))
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn create(&self, name: &str, color: Option<&str>) -> AppResult<i64> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT INTO categories (name, color) VALUES (?1, ?2)",
            params![name, color],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(&self, id: i64, name: Option<&str>, color: Option<&str>) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let changed = conn.execute(
            "UPDATE categories SET
                name = COALESCE(?1, name),
                color = COALESCE(?2, color),
                updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![name, color, id],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("Category {} not found", id)));
        }
        Ok(())
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn count(&self) -> AppResult<i64> {
        let conn = self.db.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        Ok(count)
    }
}

