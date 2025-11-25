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
use tracing::{info, error};

pub struct CoreSystemPlugin {
    metadata: PluginMetadata,
    db_url: String,
    // Note: We don't store Storage/Scheduler here anymore because init is handled in setup_messaging closure
    // If we needed to access them from other plugin methods, we'd need to use Arc<Mutex<Option<...>>> or similar
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
            
            let ctx = Arc::new(MessageContext::new(
                dc,
                plugin_name,
                tx,
            ));

            // Subscribe to relevant messages
            let mut rx_memo = ctx.subscribe("system.memo.create").await;
            let mut rx_sched = ctx.subscribe("system.schedule.add").await;

            let storage_clone = storage.clone();
            let scheduler_clone = scheduler.clone();

            // Spawn message handler
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = rx_memo.recv() => {
                            handle_memo_message(&msg, &storage_clone).await;
                        }
                        Ok(msg) = rx_sched.recv() => {
                            handle_schedule_message(&msg, &scheduler_clone).await;
                        }
                        else => {
                            // All channels closed, exit gracefully
                            tracing::info!("All message channels closed, stopping handler");
                            break;
                        }
                    }
                }
            });
            
            Ok(Some(ctx))
        })
    }
}

async fn handle_memo_message(msg: &Message, storage: &Storage) {
    if msg.message_type.as_str() == "system.memo.create" {
        if let Some(content) = msg.payload.get("content").and_then(|v| v.as_str()) {
            info!("Creating memo: {}", content);
            match storage.add_memo(content, None).await {
                Ok(id) => info!("Memo created with ID: {}", id),
                Err(e) => error!("Failed to create memo: {}", e),
            }
        }
    }
}

async fn handle_schedule_message(msg: &Message, scheduler: &Scheduler) {
    if msg.message_type.as_str() == "system.schedule.add" {
        if let Some(cron) = msg.payload.get("cron").and_then(|v| v.as_str()) {
            info!("Scheduling job: {}", cron);
            // The payload should contain the message to be sent
            if let Some(trigger_msg_val) = msg.payload.get("message") {
                 if let Ok(trigger_msg) = serde_json::from_value::<Message>(trigger_msg_val.clone()) {
                     match scheduler.add_cron_job(cron, trigger_msg).await {
                         Ok(uuid) => info!("Job scheduled: {}", uuid),
                         Err(e) => error!("Failed to schedule job: {}", e),
                     }
                 }
            }
        }
    }
}
