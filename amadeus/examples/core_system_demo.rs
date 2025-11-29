// Core System 演示
// 演示备忘录创建、查询和调度器的交互

use amadeus::App;
use amadeus::core::messaging::{DistributionCenter, Message, MessageContext};
use amadeus::plugin::{Plugin, PluginMetadata};
use anyhow::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, Level};

// 交互插件：模拟用户操作
struct InteractorPlugin {
    metadata: PluginMetadata,
}

impl InteractorPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("interactor", "Simulates user actions", "0.1.0"),
        }
    }
}

impl Plugin for InteractorPlugin {
    fn id(&self) -> &str { "interactor" }
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn setup_messaging(
        &mut self,
        dc: &DistributionCenter,
        tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let dc_arc = Arc::new(dc.clone());

        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(dc_arc, plugin_name, tx));
            let ctx_clone = ctx.clone();

            // 订阅 Core System 的回复
            let mut created_rx = ctx.subscribe("system.memo.created").await;
            let mut list_rx = ctx.subscribe("system.memo.list.reply").await;
            let mut remind_rx = ctx.subscribe("system.memo.remind").await;

            // 启动接收循环
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = created_rx.recv() => {
                            info!("✅ Memo Created: {}", msg.payload);
                            // 创建成功后，请求列表
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            let list_req = Message::new("system.memo.list", serde_json::json!({}));
                            let _ = ctx_clone.send(list_req).await;
                        }
                        Ok(msg) = list_rx.recv() => {
                            info!("📋 Memo List: {}", msg.payload);
                        }
                        Ok(msg) = remind_rx.recv() => {
                            info!("⏰ REMINDER TRIGGERED: {}", msg.payload);
                        }
                    }
                }
            });

            // 启动模拟操作
            let sender = ctx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                info!("1. Creating a memo with scheduling...");
                let create_msg = Message::new(
                    "system.memo.create",
                    serde_json::json!({
                        "content": "Buy milk",
                        "cron": "1/2 * * * * *", // Every 2 seconds (Quartz: Sec Min Hour Day Month Dow)
                        // Note: CoreSystem uses tokio-cron-scheduler. 
                        "tags": ["shopping", "urgent"]
                    })
                );
                if let Err(e) = sender.send(create_msg).await {
                    tracing::error!("Failed to send create: {}", e);
                }
            });

            Ok(Some(ctx))
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // 清理旧DB以保证演示干净
    let _ = std::fs::remove_file("amadeus.db");

    info!("=== Core System Demo ===");

    // CoreSystemPlugin 是默认加载的（如果使用 with_all_plugins 或 new），
    // 但为了明确，我们这里显式加载它（实际上 App::new() 会自动加载 CoreSystemPlugin 因为它在 get_all_plugins 中）。
    // 我们只需要添加我们的 InteractorPlugin。
    // 但是 App::with_plugins 会覆盖默认列表。
    // 所以我们需要手动把 CoreSystemPlugin 加进去，或者使用 App::new() 并注册额外的。
    // App 没有 add_plugin 方法暴露。
    // 我们手动构建列表。
    
    use amadeus::plugins::core_system::CoreSystemPlugin;
    
    let app = App::with_plugins(vec![
        Box::new(CoreSystemPlugin::new("sqlite:amadeus.db")),
        Box::new(InteractorPlugin::new()),
    ]).with_messaging();

    info!("Running... Watch for memo creation and reminders (every few seconds).");
    app.run_async().await?;

    Ok(())
}

