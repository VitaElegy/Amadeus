use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    /// Initialize storage with a database URL (e.g., "sqlite:amadeus.db")
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create the database file if it doesn't exist (if using sqlite:)
        if database_url.starts_with("sqlite:") {
             let path_str = database_url.trim_start_matches("sqlite:");
             if path_str != ":memory:" {
                 let path = Path::new(path_str);
                 if let Some(parent) = path.parent() {
                     tokio::fs::create_dir_all(parent).await?;
                 }
                 if !path.exists() {
                     std::fs::File::create(path)?;
                 }
             }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let storage = Self { pool };
        storage.init_schema().await?;
        
        Ok(storage)
    }

    async fn init_schema(&self) -> Result<()> {
        // 创建表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                remind_at INTEGER,
                cron_pattern TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                metadata TEXT,
                tags TEXT -- JSON array of tags: ["tag1", "tag2"]
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // 尝试添加 cron_pattern 列（如果不存在），以支持迁移
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN cron_pattern TEXT")
            .execute(&self.pool)
            .await;
            
        // 尝试添加 tags 列（如果不存在）
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN tags TEXT")
            .execute(&self.pool)
            .await;

        // 尝试添加 todo_date 列（如果不存在）
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN todo_date INTEGER")
            .execute(&self.pool)
            .await;

        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    // Basic CRUD operations could go here, or in a separate repository struct
    
    pub async fn add_memo(&self, content: &str, remind_at: Option<i64>, cron_pattern: Option<&str>, tags: Option<&str>, todo_date: Option<i64>) -> Result<i64> {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
            
        let id = sqlx::query(
            r#"
            INSERT INTO memos (content, created_at, remind_at, cron_pattern, status, tags, todo_date)
            VALUES (?, ?, ?, ?, 'pending', ?, ?)
            RETURNING id
            "#
        )
        .bind(content)
        .bind(created_at)
        .bind(remind_at)
        .bind(cron_pattern)
        .bind(tags)
        .bind(todo_date)
        .fetch_one(&self.pool)
        .await?
        .get(0);
        
        Ok(id)
    }

    /// 标记过期的备忘录
    pub async fn mark_expired_memos(&self) -> Result<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let result = sqlx::query(
            r#"
            UPDATE memos 
            SET status = 'expired' 
            WHERE status = 'pending' 
              AND todo_date IS NOT NULL 
              AND todo_date < ?
            "#
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取所有活跃的提醒（状态为 pending 且有 remind_at 或 cron_pattern）
    pub async fn get_active_reminders(&self) -> Result<Vec<(i64, String, Option<i64>, Option<String>, Option<String>, Option<String>)>> {
        let rows = sqlx::query(
            r#"
            SELECT id, content, remind_at, cron_pattern, metadata, tags
            FROM memos 
            WHERE status = 'pending' 
              AND (remind_at IS NOT NULL OR cron_pattern IS NOT NULL)
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let reminders = rows.iter().map(|row| {
            (
                row.get("id"),
                row.get("content"),
                row.get("remind_at"),
                row.get("cron_pattern"),
                row.get("metadata"),
                row.get("tags"),
            )
        }).collect();

        Ok(reminders)
    }

    /// 更新备忘录状态
    pub async fn update_memo_status(&self, id: i64, status: &str) -> Result<()> {
        sqlx::query("UPDATE memos SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新备忘录元数据
    pub async fn update_memo_metadata(&self, id: i64, metadata: &str) -> Result<()> {
        sqlx::query("UPDATE memos SET metadata = ? WHERE id = ?")
            .bind(metadata)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取备忘录元数据
    pub async fn get_memo_metadata(&self, id: i64) -> Result<Option<String>> {
        let row = sqlx::query("SELECT metadata FROM memos WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|r| r.get("metadata")))
    }

    /// 获取指定 Tag 的所有 pending Memo
    pub async fn get_memos_by_tag(&self, tag: &str) -> Result<Vec<(i64, String)>> {
        // SQLite doesn't have native JSON array support in older versions, using LIKE hack for simplicity
        // ideally use json_each if enabled or normalize tags table.
        // For '["tag1", "tag2"]', LIKE '%"tag1"%' works reasonably well.
        let pattern = format!("%\"{}\"%", tag);
        
        let rows = sqlx::query(
            r#"
            SELECT id, content
            FROM memos 
            WHERE status = 'pending' 
              AND tags LIKE ?
            "#
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;

        let memos = rows.iter().map(|row| {
            (
                row.get("id"),
                row.get("content"),
            )
        }).collect();

        Ok(memos)
    }
}
