use anyhow::Result;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoQueryParams {
    pub user_id: Option<String>,
    pub status: Option<String>, // "pending", "completed", "expired", "deleted", "all"
    pub tags: Option<Vec<String>>, // tags OR logic (contain any)
    pub min_priority: Option<i32>,
    pub from_date: Option<i64>,
    pub to_date: Option<i64>,
    pub keyword: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MemoRecord {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
    pub remind_at: Option<i64>,
    pub cron_pattern: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    pub todo_date: Option<i64>,
    pub priority: i32,
    pub user_id: Option<String>,
}

impl From<SqliteRow> for MemoRecord {
    fn from(row: SqliteRow) -> Self {
        let tags_str: Option<String> = row.get("tags");
        let tags = tags_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self {
            id: row.get("id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            remind_at: row.get("remind_at"),
            cron_pattern: row.get("cron_pattern"),
            status: row.get("status"),
            tags,
            todo_date: row.get("todo_date"),
            priority: row.get("priority"),
            user_id: row.get("user_id"),
        }
    }
}

