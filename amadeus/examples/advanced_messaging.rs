// 高级消息示例
// 演示 Public (广播) 和 Direct (定向) 消息

use amadeus::App;
use amadeus::core::messaging::{DistributionCenter, Message, MessageContext};
use amadeus::plugin::{Plugin, PluginMetadata};
use anyhow::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, Level};

// 1. 发送者插件：发送广播和定向消息
struct SenderPlugin {
    metadata: PluginMetadata,
}

impl SenderPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("sender", "Sends messages", "0.1.0"),
        }
    }
}

impl Plugin for SenderPlugin {
    fn id(&self) -> &str { "sender" }
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn setup_messaging(
        &mut self,
        _dc: &DistributionCenter,
        tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let plugin_uid = self.metadata.uid.clone();
        let dc_arc = Arc::new(_dc.clone());
        
        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(dc_arc, plugin_name, plugin_uid, tx));
            let ctx_clone = ctx.clone();

            // 启动发送任务
            tokio::spawn(async move {
                // 等待一下让接收者就绪
                tokio::time::sleep(Duration::from_millis(500)).await;

                // 1. 发送广播消息
                info!("[Sender] Broadcasting public message...");
                let pub_msg = Message::new("demo.public", serde_json::json!("Hello Everyone!"));
                if let Err(e) = ctx_clone.send(pub_msg).await {
                    tracing::error!("Send public failed: {}", e);
                }

                tokio::time::sleep(Duration::from_millis(500)).await;

                // 2. 发送定向消息给 Receiver
                info!("[Sender] Sending direct message to 'receiver'...");
                let direct_msg = Message::new_direct(
                    "receiver", // Target ID
                    "demo.direct",
                    serde_json::json!("Secret for you"),
                );
                if let Err(e) = ctx_clone.send(direct_msg).await {
                    tracing::error!("Send direct failed: {}", e);
                }
            });

            Ok(Some(ctx))
        })
    }
}

// 2. 接收者插件：接收消息
struct ReceiverPlugin {
    metadata: PluginMetadata,
}

impl ReceiverPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("receiver", "Receives messages", "0.1.0"),
        }
    }
}

impl Plugin for ReceiverPlugin {
    fn id(&self) -> &str { "receiver" }
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn setup_messaging(
        &mut self,
        dc: &DistributionCenter,
        tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let plugin_uid = self.metadata.uid.clone();
        let dc_arc = Arc::new(dc.clone());

        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(dc_arc, plugin_name, plugin_uid, tx));

            // 1. 订阅广播
            let mut public_rx = ctx.subscribe("demo.public").await;
            
            // 2. 开启定向接收
            let mut direct_rx = ctx.enable_direct_messaging().await;

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = public_rx.recv() => {
                            info!("[Receiver] Got PUBLIC message: {:?} from {:?}", msg.payload, msg.source);
                        }
                        Some(msg) = direct_rx.recv() => {
                            info!("[Receiver] Got DIRECT message: {:?} from {:?}", msg.payload, msg.source);
                        }
                    }
                }
            });

            Ok(Some(ctx))
        })
    }
}

// 3. 旁观者插件：只订阅广播，不应该收到定向消息
struct BystanderPlugin {
    metadata: PluginMetadata,
}

impl BystanderPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("bystander", "Should not see direct msg", "0.1.0"),
        }
    }
}

impl Plugin for BystanderPlugin {
    fn id(&self) -> &str { "bystander" }
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn setup_messaging(
        &mut self,
        dc: &DistributionCenter,
        tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let plugin_uid = self.metadata.uid.clone();
        let dc_arc = Arc::new(dc.clone());

        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(dc_arc, plugin_name, plugin_uid, tx));
            
            // 订阅同样的广播话题
            let mut public_rx = ctx.subscribe("demo.public").await;
            // 试图订阅定向话题（但这不起作用，因为定向是点对点的，除非它是广播）
            // 但我们可以订阅同名广播话题来看是否泄露
            let mut direct_leak_rx = ctx.subscribe("demo.direct").await;

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Ok(msg) = public_rx.recv() => {
                            info!("[Bystander] Got PUBLIC message: {:?}", msg.payload);
                        }
                        Ok(msg) = direct_leak_rx.recv() => {
                            // 如果收到这层，说明 Direct 消息被广播泄露了！
                            tracing::error!("[Bystander] ALARM! Saw DIRECT message: {:?}", msg.payload);
                        }
                    }
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

    info!("=== Advanced Messaging Demo ===");

    let app = App::with_plugins(vec![
        Box::new(ReceiverPlugin::new()),
        Box::new(BystanderPlugin::new()),
        Box::new(SenderPlugin::new()),
    ]).with_messaging();

    // 运行应用（这里会阻塞，按 Ctrl+C 退出）
    // 观察日志输出
    app.run_async().await?;

    Ok(())
}

