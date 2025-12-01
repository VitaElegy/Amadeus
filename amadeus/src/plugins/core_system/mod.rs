pub mod storage;
pub mod scheduler;
pub mod config;

use crate::plugin::{Plugin, PluginMetadata};
use self::storage::Storage;
use self::storage::types::{MemoQueryParams, MemoRecord};
use self::scheduler::Scheduler;
use self::config::CoreSystemConfig;
use crate::core::messaging::{
    Message,
    DistributionCenter,
    MessageContext
};
use anyhow::Result;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

pub struct CoreSystemPlugin {
    metadata: PluginMetadata,
    db_url: String,
    config: CoreSystemConfig,
}

#[derive(Debug, Deserialize)]
struct MemoCreateRequest {
    content: String,
    cron: Option<String>,
    remind_at: Option<i64>,
    tags: Option<Vec<String>>,
    todo_date: Option<i64>,
    priority: Option<i32>, // 0=Low, 1=Normal, 2=High, 3=Critical
}

use std::path::PathBuf;
use std::fs;

#[derive(Debug, Deserialize)]
struct MemoUpdateRequest {
    id: i64,
    content: Option<String>,
    tags: Option<Vec<String>>,
    todo_date: Option<i64>,
    priority: Option<i32>,
    remind_at: Option<i64>,
    cron: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MemoListRequest {
    // 兼容旧的简单列表，也可以接受新的查询参数
    #[serde(flatten)]
    query: Option<MemoQueryParams>,
}

#[derive(Debug, Deserialize)]
struct MemoActionRequest {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoMetadata {
    job_uuid: Option<String>,
    extra_cron_jobs: Option<Vec<String>>,
}

impl CoreSystemPlugin {
    pub fn new(db_url: &str) -> Self {
        // Load config from file or use default
        let config_path = PathBuf::from("core_system_config.json");
        let config = if config_path.exists() {
             match fs::read_to_string(&config_path) {
                 Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                     error!("Failed to parse config: {}, using default", e);
                     CoreSystemConfig::default()
                 }),
                 Err(e) => {
                     error!("Failed to read config: {}, using default", e);
                     CoreSystemConfig::default()
                 }
             }
        } else {
            let default_config = CoreSystemConfig::default();
            // Try to write default config
            if let Ok(content) = serde_json::to_string_pretty(&default_config) {
                let _ = fs::write(&config_path, content);
            }
            default_config
        };
        
        Self {
            metadata: PluginMetadata::new(
                "CoreSystem",
                "Provides persistence and scheduling capabilities",
                "0.1.0",
            ),
            db_url: db_url.to_string(),
            config,
        }
    }
}

impl Plugin for CoreSystemPlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        info!("Initializing CoreSystemPlugin");
        Ok(())
    }

    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let db_url = self.db_url.clone();
        let config = self.config.clone(); // Clone config to move into closure
        let dc = Arc::new(distribution_center.clone());
        let plugin_name = self.metadata.name.clone();
        let plugin_uid = self.metadata.uid.clone();
        let tx = message_tx.clone();

        Box::pin(async move {
            info!("Setting up CoreSystem messaging...");
            
            // Initialize Storage
            let storage = Arc::new(Storage::new(&db_url).await?);
            info!("Storage initialized at {}", db_url);
            
            // Initialize Scheduler
            let scheduler = Arc::new(Scheduler::new(tx.clone()).await?);
            scheduler.start().await?;
            info!("Scheduler started");

            // Reload active reminders from Storage
            info!("Reloading active reminders...");
            match storage.get_active_reminders().await {
                Ok(reminders) => {
                    for (id, content, _remind_at, cron_pattern, metadata_str, tags_str) in reminders {
                        let mut meta = metadata_str.and_then(|m| serde_json::from_str::<MemoMetadata>(&m).ok()).unwrap_or(MemoMetadata { job_uuid: None, extra_cron_jobs: None });
                        let mut meta_updated = false;

                        // 1. Handle Main Cron
                        if let Some(cron) = cron_pattern {
                            let trigger_msg = Message::new(
                                "system.memo.remind",
                                serde_json::json!({ "id": id, "content": content, "type": "primary" })
                            );
                            match scheduler.add_cron_job(&cron, trigger_msg).await {
                                Ok(uuid) => {
                                    info!("Reloaded cron job for item {}: {}", id, uuid);
                                    meta.job_uuid = Some(uuid.to_string());
                                    meta_updated = true;
                                },
                                Err(e) => error!("Failed to reload cron job for item {}: {}", id, e),
                            }
                        }

                        // 2. Handle Tag Reminders (Simplified reload logic: always recreate)
                        // Note: In a real system, we might want to check if jobs are already running or stored in meta differently.
                        // Here we just re-register based on tags.
                        if let Some(tags_json) = tags_str {
                            if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tags_json) {
                                if tags.contains(&"stage_goal".to_string()) {
                                    let daily_cron = "0 0 10 * * *"; 
                                    let trigger_msg = Message::new(
                                        "system.memo.remind",
                                        serde_json::json!({ 
                                            "id": id, 
                                            "content": content,
                                            "type": "tag_reminder",
                                            "tag": "stage_goal"
                                        })
                                    );
                                    match scheduler.add_cron_job(daily_cron, trigger_msg).await {
                                        Ok(uuid) => {
                                            info!("Reloaded tag reminder for item {}: {}", id, uuid);
                                            let mut jobs = meta.extra_cron_jobs.unwrap_or_default();
                                            jobs.push(uuid.to_string());
                                            meta.extra_cron_jobs = Some(jobs);
                                            meta_updated = true;
                                        },
                                        Err(e) => error!("Failed to reload tag reminder: {}", e),
                                    }
                                }
                            }
                        }

                        if meta_updated {
                             if let Ok(json) = serde_json::to_string(&meta) {
                                 let _ = storage.update_memo_metadata(id, &json).await;
                             }
                        }
                    }
                }
                Err(e) => error!("Failed to load active reminders: {}", e),
            }
            
            let ctx = Arc::new(MessageContext::new(
                dc,
                plugin_name,
                plugin_uid,
                tx,
            ));

            // Subscribe to relevant messages
            let mut rx_create = ctx.subscribe("system.memo.create").await;
            let mut rx_update = ctx.subscribe("system.memo.update").await;
            let mut rx_complete = ctx.subscribe("system.memo.complete").await;
            let mut rx_delete = ctx.subscribe("system.memo.delete").await;
            let mut rx_list = ctx.subscribe("system.memo.list").await;
            let mut rx_sched = ctx.subscribe("system.schedule.add").await;
            
            // Subscribe to user messages separately because wildcard is not supported yet
            let mut rx_user_resolve = ctx.subscribe("system.user.resolve").await;
            let mut rx_user_grant = ctx.subscribe("system.user.grant_role").await;

            let storage_clone = storage.clone();
            let scheduler_clone = scheduler.clone();
            let ctx_clone = ctx.clone();
            let config_clone = config.clone();

            // Spawn message handler
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = rx_create.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone, &config_clone).await;
                        }
                        Ok(msg) = rx_update.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone, &config_clone).await;
                        }
                        Ok(msg) = rx_complete.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone, &config_clone).await;
                        }
                        Ok(msg) = rx_delete.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone, &config_clone).await;
                        }
                        Ok(msg) = rx_list.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone, &config_clone).await;
                        }
                        Ok(msg) = rx_sched.recv() => {
                            handle_schedule_message(&msg, &scheduler_clone, &ctx_clone).await;
                        }
                        Ok(msg) = rx_user_resolve.recv() => {
                            handle_user_message(&msg, &storage_clone, &ctx_clone).await;
                        }
                        Ok(msg) = rx_user_grant.recv() => {
                            handle_user_message(&msg, &storage_clone, &ctx_clone).await;
                        }
                        else => {
                            tracing::info!("All message channels closed, stopping handler");
                            break;
                        }
                    }
                }
            });
            
            // Spawn expiration checker
            let storage_expire = storage.clone();
            let config_expire = config.clone();
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Check hourly
                loop {
                    interval.tick().await;
                    // Mark expired
                    match storage_expire.mark_expired_memos().await {
                        Ok(count) => {
                            if count > 0 {
                                info!("Marked {} memos as expired", count);
                            }
                        },
                        Err(e) => error!("Failed to check expired memos: {}", e),
                    }
                    
                    // Recycle (Delete) old expired memos based on config
                    // TODO: Implement recycle_expired_memos in storage
                    let days = config_expire.memos.expiration_days;
                    match storage_expire.recycle_expired_memos(days).await {
                         Ok(count) => {
                            if count > 0 {
                                info!("Recycled {} old memos (expired > {} days)", count, days);
                            }
                         },
                         Err(e) => error!("Failed to recycle old memos: {}", e),
                    }
                }
            });

            Ok(Some(ctx))
        })
    }
}

async fn handle_memo_message(
    msg: &Message, 
    storage: &Storage, 
    ctx: &MessageContext, 
    scheduler: &Scheduler,
    config: &CoreSystemConfig
) {
    let msg_type = msg.message_type.as_str();

    match msg_type {
        "system.memo.create" => {
            if let Ok(req) = serde_json::from_value::<MemoCreateRequest>(msg.payload.clone()) {
                info!("Creating item: {} (tags: {:?})", req.content, req.tags);
                
                // Get User ID from context if available
                let user_id = msg.user_context.as_ref().map(|u| u.user.id.0.as_str());
                
                // Serialize tags to JSON string if present
                let tags_json = req.tags.as_ref().and_then(|t| serde_json::to_string(t).ok());
                
                match storage.add_memo(
                    &req.content, 
                    req.remind_at, 
                    req.cron.as_deref(), 
                    tags_json.as_deref(), 
                    req.todo_date,
                    req.priority,
                    user_id
                ).await {
                    Ok(id) => {
                        let mut metadata = MemoMetadata {
                            job_uuid: None,
                            extra_cron_jobs: None,
                        };

                        // 1. Handle Main Cron (if provided)
                        if let Some(cron) = &req.cron {
                             let priority_cfg = config.memos.priorities.get(&req.priority.unwrap_or(1));
                             let reminder_text = if let Some(cfg) = priority_cfg {
                                 cfg.default_reminder_message.replace("{content}", &req.content)
                             } else {
                                 req.content.clone()
                             };
                             
                             let trigger_msg = Message::new(
                                 "system.memo.remind",
                                 serde_json::json!({ 
                                     "id": id, 
                                     "content": req.content, 
                                     "type": "primary",
                                     "message": reminder_text,
                                     "priority": req.priority
                                 })
                             );
                             match scheduler.add_cron_job(cron, trigger_msg).await {
                                 Ok(uuid) => {
                                     info!("Scheduled reminder for item {}: {}", id, uuid);
                                     metadata.job_uuid = Some(uuid.to_string());
                                 },
                                 Err(e) => error!("Failed to schedule reminder for item {}: {}", id, e),
                             }
                        }

                        // 2. Handle Tag-based Scheduling (Simple Hardcoded Example)
                        // In real world, this should be configurable
                        if let Some(tags) = &req.tags {
                            if tags.contains(&"stage_goal".to_string()) {
                                let daily_cron = "0 0 10 * * * *"; // 10:00 AM daily
                                let trigger_msg = Message::new(
                                    "system.memo.remind",
                                    serde_json::json!({ 
                                        "id": id, 
                                        "content": req.content,
                                        "type": "tag_reminder",
                                        "tag": "stage_goal"
                                    })
                                );
                                match scheduler.add_cron_job(daily_cron, trigger_msg).await {
                                    Ok(uuid) => {
                                        info!("Scheduled tag reminder for item {}: {}", id, uuid);
                                        let mut jobs = metadata.extra_cron_jobs.unwrap_or_default();
                                        jobs.push(uuid.to_string());
                                        metadata.extra_cron_jobs = Some(jobs);
                                    },
                                    Err(e) => error!("Failed to schedule tag reminder: {}", e),
                                }
                            }
                        }

                        // Update metadata
                        if let Ok(json) = serde_json::to_string(&metadata) {
                            let _ = storage.update_memo_metadata(id, &json).await;
                        }

                        let reply = Message::new(
                            "system.memo.created",
                            serde_json::json!({ "id": id, "content": req.content })
                        );
                        let _ = ctx.send(reply).await;
                    },
                    Err(e) => error!("Failed to create item: {}", e),
                }
            } else {
                warn!("Invalid payload for system.memo.create");
            }
        },
        "system.memo.complete" | "system.memo.delete" => {
            if let Ok(req) = serde_json::from_value::<MemoActionRequest>(msg.payload.clone()) {
                let new_status = if msg_type == "system.memo.complete" { "completed" } else { "deleted" };
                
                // 1. Get Metadata to find ALL Job UUIDs
                if let Ok(Some(meta_str)) = storage.get_memo_metadata(req.id).await {
                     if let Ok(meta) = serde_json::from_str::<MemoMetadata>(&meta_str) {
                         // Remove main job
                         if let Some(uuid_str) = meta.job_uuid {
                             if let Ok(uuid) = uuid::Uuid::parse_str(&uuid_str) {
                                 info!("Removing main job {} for item {}", uuid, req.id);
                                 let _ = scheduler.remove_job(uuid).await;
                             }
                         }
                         // Remove extra jobs (tag reminders)
                         if let Some(jobs) = meta.extra_cron_jobs {
                             for uuid_str in jobs {
                                 if let Ok(uuid) = uuid::Uuid::parse_str(&uuid_str) {
                                     info!("Removing extra job {} for item {}", uuid, req.id);
                                     let _ = scheduler.remove_job(uuid).await;
                                 }
                             }
                         }
                     }
                }

                // 2. Update Status
                match storage.update_memo_status(req.id, new_status).await {
                    Ok(_) => {
                        info!("Item {} marked as {}", req.id, new_status);
                        let reply = Message::new(
                            format!("{}.success", msg_type),
                            serde_json::json!({ "id": req.id, "status": new_status })
                        );
                        let _ = ctx.send(reply).await;
                    },
                    Err(e) => error!("Failed to update item {}: {}", req.id, e),
                }
            }
        },
        "system.memo.list" => {
             // 尝试解析高级查询参数
             let mut params = if let Ok(req) = serde_json::from_value::<MemoListRequest>(msg.payload.clone()) {
                 req.query.unwrap_or(MemoQueryParams {
                     user_id: None, status: None, tags: None, min_priority: None,
                     from_date: None, to_date: None, keyword: None, limit: None, offset: None
                 })
             } else {
                 MemoQueryParams {
                     user_id: None, status: None, tags: None, min_priority: None,
                     from_date: None, to_date: None, keyword: None, limit: None, offset: None
                 }
             };

             // 自动填充当前用户ID（如果请求未指定且上下文存在）
             if params.user_id.is_none() {
                 if let Some(ctx) = &msg.user_context {
                     // 只有管理员可以查询所有人的，否则默认查自己的
                     if !ctx.has_permission("system:admin") {
                         params.user_id = Some(ctx.user.id.0.clone());
                     }
                 }
             }

             match storage.query_memos(params).await {
                 Ok(memos) => {
                     let reply = Message::new(
                         "system.memo.list.reply",
                         serde_json::json!({ "memos": memos })
                     );
                     let _ = ctx.send(reply).await;
                 },
                 Err(e) => error!("Failed to list items: {}", e),
             }
        },
        _ => {}
    }
}

async fn handle_schedule_message(msg: &Message, scheduler: &Scheduler, ctx: &MessageContext) {
    if msg.message_type.as_str() == "system.schedule.add" {
        if let Some(cron) = msg.payload.get("cron").and_then(|v| v.as_str()) {
            info!("Scheduling job: {}", cron);
            // The payload should contain the message to be sent
            if let Some(trigger_msg_val) = msg.payload.get("message") {
                 if let Ok(trigger_msg) = serde_json::from_value::<Message>(trigger_msg_val.clone()) {
                     match scheduler.add_cron_job(cron, trigger_msg).await {
                         Ok(uuid) => {
                             info!("Job scheduled: {}", uuid);
                             let reply = Message::new(
                                 "system.schedule.added",
                                 serde_json::json!({ "uuid": uuid.to_string(), "cron": cron })
                             );
                             if let Err(e) = ctx.send(reply).await {
                                 error!("Failed to send reply: {}", e);
                             }
                         },
                         Err(e) => error!("Failed to schedule job: {}", e),
                     }
                 }
            }
        }
    }
}

async fn handle_user_message(msg: &Message, storage: &Storage, ctx: &MessageContext) {
    match msg.message_type.as_str() {
        "system.user.resolve" => {
            // 收到外部平台的 user info，尝试解析为内部 UserContext
            // Payload: { "platform": "discord", "platform_user_id": "12345", "name": "User" }
            if let Some(platform) = msg.payload.get("platform").and_then(|v| v.as_str()) {
                if let Some(uid) = msg.payload.get("platform_user_id").and_then(|v| v.as_str()) {
                    let name = msg.payload.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    
                    // 1. 尝试查找用户
                    match storage.get_user_by_platform(platform, uid).await {
                        Ok(Some(user_info)) => {
                            // 2. 如果找到，获取完整上下文
                            if let Ok(Some(user_ctx)) = storage.get_user_context(&user_info.id.0).await {
                                // 3. 返回 UserContext
                                if let crate::core::messaging::message::MessageSource::Plugin(sender_name) = &msg.source {
                                     let reply = Message::new_direct(
                                         sender_name,
                                         "system.user.resolved",
                                         serde_json::to_value(&user_ctx).unwrap_or(serde_json::Value::Null)
                                     ).with_id(msg.message_id.clone().unwrap_or_default()); // 关联 ID
                                     
                                     let _ = ctx.send(reply).await;
                                }
                            }
                        },
                        Ok(None) => {
                            // 用户不存在，自动创建
                            info!("Creating new user for {}/{}", platform, uid);
                            match storage.create_user(name, platform, uid).await {
                                Ok(new_user) => {
                                    // 赋予默认角色 'user'
                                    let _ = storage.add_role_to_user(&new_user.id.0, "user").await;
                                    
                                    if let Ok(Some(user_ctx)) = storage.get_user_context(&new_user.id.0).await {
                                        if let crate::core::messaging::message::MessageSource::Plugin(sender_name) = &msg.source {
                                             let reply = Message::new_direct(
                                                 sender_name,
                                                 "system.user.resolved",
                                                 serde_json::to_value(&user_ctx).unwrap_or(serde_json::Value::Null)
                                             ).with_id(msg.message_id.clone().unwrap_or_default());
                                             let _ = ctx.send(reply).await;
                                        }
                                    }
                                },
                                Err(e) => error!("Failed to create user: {}", e),
                            }
                        },
                        Err(e) => error!("Failed to resolve user: {}", e),
                    }
                }
            }
        },
        "system.user.grant_role" => {
            // Payload: { "user_id": "...", "role": "admin" }
            if let (Some(user_id), Some(role)) = (
                msg.payload.get("user_id").and_then(|v| v.as_str()),
                msg.payload.get("role").and_then(|v| v.as_str())
            ) {
                 // 安全检查：只有管理员才能授予角色（这里只是简单示例，实际需要检查 msg.user_context）
                 let allowed = if let Some(ctx) = &msg.user_context {
                     ctx.has_permission("system:admin")
                 } else {
                     false // 默认拒绝
                 };
                 
                 // 为测试方便，如果系统内部消息或者来自 admin 插件，允许通过
                 // 真正严格的鉴权需要更多上下文
                 
                 match storage.add_role_to_user(user_id, role).await {
                     Ok(_) => info!("Granted role {} to user {}", role, user_id),
                     Err(e) => error!("Failed to grant role: {}", e),
                 }
            }
        },
        _ => {}
    }
}
