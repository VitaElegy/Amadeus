use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row, QueryBuilder};
use std::path::Path;
use crate::core::user::{UserId, PlatformId, PlatformUserId, UserInfo, UserContext};
use std::collections::HashSet;

pub mod types;
use self::types::{MemoQueryParams, MemoRecord};

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
        // 创建表 - 重构 Memos 表以支持更高级的查询
        // 注意：SQLite 的 ALTER TABLE 功能有限，对于复杂的结构变更，
        // 生产环境通常需要创建新表 -> 迁移数据 -> 删除旧表。
        // 为了演示和简化，这里我们假设是新部署或者兼容性修改。
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                remind_at INTEGER,
                cron_pattern TEXT,
                status TEXT NOT NULL DEFAULT 'pending', -- pending, completed, deleted, expired
                metadata TEXT,
                tags TEXT, -- JSON array of tags: ["tag1", "tag2"]
                todo_date INTEGER, -- 截止日期/执行日期
                priority INTEGER DEFAULT 1, -- 重要程度: 0=Low, 1=Normal, 2=High, 3=Critical
                user_id TEXT -- 所有者ID
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // 尝试添加新字段以支持迁移
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN cron_pattern TEXT").execute(&self.pool).await;
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN tags TEXT").execute(&self.pool).await;
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN todo_date INTEGER").execute(&self.pool).await;
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN priority INTEGER DEFAULT 1").execute(&self.pool).await;
        let _ = sqlx::query("ALTER TABLE memos ADD COLUMN user_id TEXT").execute(&self.pool).await;

        // 创建索引以加速查询
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_memos_user_status ON memos(user_id, status)").execute(&self.pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_memos_todo_date ON memos(todo_date)").execute(&self.pool).await;

        // --- 用户系统表 ---
        
        // Users Table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                platform TEXT NOT NULL,
                platform_user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_users_platform ON users(platform, platform_user_id);
            "#
        )
        .execute(&self.pool)
        .await?;

        // User Roles Table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_roles (
                user_id TEXT NOT NULL,
                role TEXT NOT NULL,
                PRIMARY KEY (user_id, role),
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // Role Permissions Table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS role_permissions (
                role TEXT NOT NULL,
                permission TEXT NOT NULL,
                PRIMARY KEY (role, permission)
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
    
    // --- 备忘录核心功能 ---

    pub async fn add_memo(
        &self, 
        content: &str, 
        remind_at: Option<i64>, 
        cron_pattern: Option<&str>, 
        tags: Option<&str>, 
        todo_date: Option<i64>,
        priority: Option<i32>,
        user_id: Option<&str>
    ) -> Result<i64> {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        
        let priority_val = priority.unwrap_or(1); // Default Normal
            
        let id = sqlx::query(
            r#"
            INSERT INTO memos (content, created_at, remind_at, cron_pattern, status, tags, todo_date, priority, user_id)
            VALUES (?, ?, ?, ?, 'pending', ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(content)
        .bind(created_at)
        .bind(remind_at)
        .bind(cron_pattern)
        .bind(tags)
        .bind(todo_date)
        .bind(priority_val)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?
        .get(0);
        
        Ok(id)
    }

    /// 高级查询接口
    /// 使用 sqlx::QueryBuilder 安全地构建动态 SQL，防止注入
    pub async fn query_memos(&self, params: MemoQueryParams) -> Result<Vec<MemoRecord>> {
        let mut qb = QueryBuilder::new("SELECT * FROM memos WHERE 1=1 ");

        // User Filter
        if let Some(uid) = params.user_id {
            qb.push(" AND user_id = ");
            qb.push_bind(uid);
        }

        // Status Filter
        if let Some(status) = params.status {
            if status != "all" {
                qb.push(" AND status = ");
                qb.push_bind(status);
            }
        } else {
            // Default to not showing deleted
            qb.push(" AND status != 'deleted' ");
        }

        // Priority Filter
        if let Some(min_p) = params.min_priority {
            qb.push(" AND priority >= ");
            qb.push_bind(min_p);
        }

        // Date Range Filter (todo_date)
        if let Some(from) = params.from_date {
            qb.push(" AND todo_date >= ");
            qb.push_bind(from);
        }
        if let Some(to) = params.to_date {
            qb.push(" AND todo_date <= ");
            qb.push_bind(to);
        }

        // Keyword Search (Content)
        if let Some(keyword) = params.keyword {
            qb.push(" AND content LIKE ");
            qb.push_bind(format!("%{}%", keyword));
        }

        // Tag Filter (Complex: SQLite doesn't have array types, using LIKE workaround)
        // For strict correctness, we should iterate tags.
        // Or if we normalized tags, we'd use JOIN.
        // Here we use OR logic: (tags LIKE '%"tag1"%' OR tags LIKE '%"tag2"%')
        if let Some(tags) = params.tags {
            if !tags.is_empty() {
                qb.push(" AND (");
                let mut separated = qb.separated(" OR ");
                for tag in tags {
                    separated.push("tags LIKE ");
                    separated.push_bind_unseparated(format!("%\"{}\"%", tag));
                }
                qb.push(")");
            }
        }

        // Ordering
        qb.push(" ORDER BY todo_date ASC, priority DESC, created_at DESC ");

        // Pagination
        if let Some(limit) = params.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit);
        }
        if let Some(offset) = params.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset);
        }

        let query = qb.build();
        let rows = query.fetch_all(&self.pool).await?;

        let records = rows.into_iter().map(MemoRecord::from).collect();
        Ok(records)
    }

    /// 标记过期的备忘录（自动回收）
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

    /// 回收（删除）过期的备忘录
    /// days: 备忘录过期多少天后被删除
    pub async fn recycle_expired_memos(&self, days: u64) -> Result<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
            
        let cutoff_time = now - (days * 24 * 3600) as i64;

        let result = sqlx::query(
            r#"
            DELETE FROM memos 
            WHERE status = 'expired' 
              AND todo_date IS NOT NULL 
              AND todo_date < ?
            "#
        )
        .bind(cutoff_time)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取所有活跃的提醒（状态为 pending 且有 remind_at 或 cron_pattern）
    /// 这个主要用于系统启动时加载调度器，不需要复杂的过滤
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

    /// 更新备忘录完整信息
    pub async fn update_memo(
        &self, 
        id: i64, 
        content: Option<&str>, 
        remind_at: Option<i64>, 
        cron_pattern: Option<&str>, 
        tags: Option<&str>, 
        todo_date: Option<i64>,
        priority: Option<i32>
    ) -> Result<()> {
        let mut qb = QueryBuilder::new("UPDATE memos SET ");
        let mut separated = qb.separated(", ");
        
        if let Some(c) = content {
            separated.push("content = ");
            separated.push_bind_unseparated(c);
        }
        
        if let Some(r) = remind_at {
            separated.push("remind_at = ");
            separated.push_bind_unseparated(r);
        }

        if let Some(c) = cron_pattern {
            separated.push("cron_pattern = ");
            separated.push_bind_unseparated(c);
        }
        
        if let Some(t) = tags {
            separated.push("tags = ");
            separated.push_bind_unseparated(t);
        }
        
        if let Some(td) = todo_date {
            separated.push("todo_date = ");
            separated.push_bind_unseparated(td);
        }
        
        if let Some(p) = priority {
            separated.push("priority = ");
            separated.push_bind_unseparated(p);
        }
        
        qb.push(" WHERE id = ");
        qb.push_bind(id);
        
        qb.build().execute(&self.pool).await?;
        Ok(())
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

    // --- 用户系统方法 ---

    pub async fn get_user_by_platform(&self, platform: &str, platform_user_id: &str) -> Result<Option<UserInfo>> {
        let row = sqlx::query(
            "SELECT id, name, platform, platform_user_id FROM users WHERE platform = ? AND platform_user_id = ?"
        )
        .bind(platform)
        .bind(platform_user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserInfo {
            id: UserId::new(r.get::<String, _>("id")),
            name: r.get("name"),
            platform: PlatformId(r.get("platform")),
            platform_user_id: PlatformUserId(r.get("platform_user_id")),
        }))
    }

    pub async fn create_user(&self, name: &str, platform: &str, platform_user_id: &str) -> Result<UserInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO users (id, name, platform, platform_user_id, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(name)
        .bind(platform)
        .bind(platform_user_id)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(UserInfo {
            id: UserId::new(id),
            name: name.to_string(),
            platform: PlatformId(platform.to_string()),
            platform_user_id: PlatformUserId(platform_user_id.to_string()),
        })
    }

    pub async fn get_user_context(&self, user_id: &str) -> Result<Option<UserContext>> {
        // 1. Get User Info
        let user_row = sqlx::query(
            "SELECT id, name, platform, platform_user_id FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let user_info = match user_row {
            Some(r) => UserInfo {
                id: UserId::new(r.get::<String, _>("id")),
                name: r.get("name"),
                platform: PlatformId(r.get("platform")),
                platform_user_id: PlatformUserId(r.get("platform_user_id")),
            },
            None => return Ok(None),
        };

        // 2. Get Roles
        let role_rows = sqlx::query(
            "SELECT role FROM user_roles WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let roles: Vec<String> = role_rows.iter().map(|r| r.get("role")).collect();

        // 3. Get Permissions (from all roles)
        let mut permissions = HashSet::new();
        for role in &roles {
            let perm_rows = sqlx::query(
                "SELECT permission FROM role_permissions WHERE role = ?"
            )
            .bind(role)
            .fetch_all(&self.pool)
            .await?;

            for r in perm_rows {
                permissions.insert(crate::core::user::Permission::new(r.get::<String, _>("permission")));
            }
        }

        let mut ctx = UserContext::new(user_info);
        ctx.roles = roles;
        ctx.permissions = permissions;

        Ok(Some(ctx))
    }

    pub async fn add_role_to_user(&self, user_id: &str, role: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role) VALUES (?, ?)")
            .bind(user_id)
            .bind(role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_permission_to_role(&self, role: &str, permission: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO role_permissions (role, permission) VALUES (?, ?)")
            .bind(role)
            .bind(permission)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
