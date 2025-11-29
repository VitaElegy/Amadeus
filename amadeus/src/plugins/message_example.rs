// amadeus/src/plugins/message_example.rs

use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use crate::message_context::MessageContext;
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::pin::Pin;

/// 消息示例插件
/// 
/// 演示如何订阅和发送消息
pub struct MessageExamplePlugin {
    metadata: PluginMetadata,
}

impl MessageExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "message-example",
                "演示消息订阅和发送功能",
                "0.1.0",
            ),
        }
    }
}

impl Plugin for MessageExamplePlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let dc = Arc::new(distribution_center.clone());
        
        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(
                dc,
                plugin_name,
                message_tx,
            ));
            
            // 订阅所有消息（通配符）
            let mut rx = ctx.subscribe("test.message").await;
            
            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    tracing::info!("[MessageExample] 收到消息: {:?}", msg);
                }
            });

            Ok(Some(ctx))
        })
    }
}
