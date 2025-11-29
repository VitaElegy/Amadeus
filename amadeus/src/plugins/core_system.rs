use crate::plugin::{Plugin, PluginMetadata};
use crate::storage::Storage;
use crate::scheduler::Scheduler;
use crate::message::Message;
use crate::distribution_center::DistributionCenter;
use crate::message_context::MessageContext;
use anyhow::Result;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

pub struct CoreSystemPlugin {
    metadata: PluginMetadata,
    db_url: String,
}

#[derive(Debug, Deserialize)]
struct MemoCreateRequest {
    content: String,
    cron: Option<String>,
    remind_at: Option<i64>,
    tags: Option<Vec<String>>,
    todo_date: Option<i64>,
    priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MemoActionRequest {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoMetadata {
    job_uuid: Option<String>,
    todo_date: Option<i64>,
    priority: Option<i32>,
    extra_cron_jobs: Option<Vec<String>>,
}

impl CoreSystemPlugin {
    pub fn new(db_url: &str) -> Self {
        Self {
            metadata: PluginMetadata::new(
                "CoreSystem",
                "Provides persistence and scheduling capabilities",
                "0.1.0",
            ),
            db_url: db_url.to_string(),
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
        let dc = Arc::new(distribution_center.clone());
        let plugin_name = self.metadata.name.clone();
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
                        let mut meta = metadata_str.and_then(|m| serde_json::from_str::<MemoMetadata>(&m).ok()).unwrap_or(MemoMetadata { job_uuid: None, todo_date: None, priority: None, extra_cron_jobs: None });
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
                                    let daily_cron = "0 0 10 * * * *"; 
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
                tx,
            ));

            // Subscribe to relevant messages
            let mut rx_create = ctx.subscribe("system.memo.create").await;
            let mut rx_complete = ctx.subscribe("system.memo.complete").await;
            let mut rx_delete = ctx.subscribe("system.memo.delete").await;
            let mut rx_list = ctx.subscribe("system.memo.list").await;
            let mut rx_sched = ctx.subscribe("system.schedule.add").await;

            let storage_clone = storage.clone();
            let scheduler_clone = scheduler.clone();
            let ctx_clone = ctx.clone();

            // Spawn message handler
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = rx_create.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone).await;
                        }
                        Ok(msg) = rx_complete.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone).await;
                        }
                        Ok(msg) = rx_delete.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone).await;
                        }
                        Ok(msg) = rx_list.recv() => {
                            handle_memo_message(&msg, &storage_clone, &ctx_clone, &scheduler_clone).await;
                        }
                        Ok(msg) = rx_sched.recv() => {
                            handle_schedule_message(&msg, &scheduler_clone, &ctx_clone).await;
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
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    match storage_expire.mark_expired_memos().await {
                        Ok(count) => {
                            if count > 0 {
                                info!("Marked {} memos as expired", count);
                            }
                        },
                        Err(e) => error!("Failed to check expired memos: {}", e),
                    }
                }
            });

            Ok(Some(ctx))
        })
    }
}

async fn handle_memo_message(msg: &Message, storage: &Storage, ctx: &MessageContext, scheduler: &Scheduler) {
    let msg_type = msg.message_type.as_str();

    match msg_type {
        "system.memo.create" => {
            if let Ok(req) = serde_json::from_value::<MemoCreateRequest>(msg.payload.clone()) {
                info!("Creating item: {} (tags: {:?})", req.content, req.tags);
                
                // Serialize tags to JSON string if present
                let tags_json = req.tags.as_ref().and_then(|t| serde_json::to_string(t).ok());
                
                match storage.add_memo(&req.content, req.remind_at, req.cron.as_deref(), tags_json.as_deref(), req.todo_date).await {
                    Ok(id) => {
                        let mut metadata = MemoMetadata {
                            job_uuid: None,
                            todo_date: req.todo_date,
                            priority: req.priority,
                            extra_cron_jobs: None,
                        };

                        // 1. Handle Main Cron (if provided)
                        if let Some(cron) = &req.cron {
                             let trigger_msg = Message::new(
                                 "system.memo.remind",
                                 serde_json::json!({ "id": id, "content": req.content, "type": "primary" })
                             );
                             match scheduler.add_cron_job(cron, trigger_msg).await {
                                 Ok(uuid) => {
                                     info!("Scheduled reminder for item {}: {}", id, uuid);
                                     metadata.job_uuid = Some(uuid.to_string());
                                 },
                                 Err(e) => error!("Failed to schedule reminder for item {}: {}", id, e),
                             }
                        }

                        // 2. Handle Tag-based Scheduling
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
             match storage.get_active_reminders().await {
                 Ok(reminders) => {
                     let list: Vec<serde_json::Value> = reminders.into_iter().map(|(id, content, remind_at, cron, metadata_str, tags_str)| {
                         // Parse metadata to get extra fields if needed
                         let mut todo_date = None;
                         let mut priority = None;
                         if let Some(meta) = metadata_str.and_then(|m| serde_json::from_str::<MemoMetadata>(&m).ok()) {
                             todo_date = meta.todo_date;
                             priority = meta.priority;
                         }
                         
                         // Parse tags
                         let tags: Option<Vec<String>> = tags_str.and_then(|t| serde_json::from_str(&t).ok());

                         serde_json::json!({
                             "id": id,
                             "content": content,
                             "remind_at": remind_at,
                             "cron": cron,
                             "tags": tags,
                             "todo_date": todo_date,
                             "priority": priority
                         })
                     }).collect();
                     
                     let reply = Message::new(
                         "system.memo.list.reply",
                         serde_json::json!({ "memos": list })
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
