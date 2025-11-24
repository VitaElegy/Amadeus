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
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                remind_at INTEGER,
                status TEXT NOT NULL DEFAULT 'pending',
                metadata TEXT -- JSON string for extra fields
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    // Basic CRUD operations could go here, or in a separate repository struct
    
    pub async fn add_memo(&self, content: &str, remind_at: Option<i64>) -> Result<i64> {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
            
        let id = sqlx::query(
            r#"
            INSERT INTO memos (content, created_at, remind_at, status)
            VALUES (?, ?, ?, 'pending')
            RETURNING id
            "#
        )
        .bind(content)
        .bind(created_at)
        .bind(remind_at)
        .fetch_one(&self.pool)
        .await?
        .get(0);
        
        Ok(id)
    }
}
