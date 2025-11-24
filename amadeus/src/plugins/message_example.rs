use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use crate::message_context::MessageContext;
use crate::plugin::{MessagePlugin, Plugin, PluginMetadata};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

/// 消息示例插件
/// 
/// 演示如何使用 tokio 通道订阅和发送消息
pub struct MessageExamplePlugin {
    metadata: PluginMetadata,
    message_context: Option<Arc<MessageContext>>,
    received_count: usize,
    /// 消息接收任务句柄
    message_task_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl MessageExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "message_example",
                "消息系统示例插件",
                "1.0.0",
            ),
            message_context: None,
            received_count: 0,
            message_task_handles: Vec::new(),
        }
    }

    /// 发送测试消息
    pub async fn send_test_message(&self) -> Result<()> {
        if let Some(ctx) = &self.message_context {
            let message = Message::from_plugin(
                "test.message",
                json!({
                    "content": "这是一条测试消息",
                    "from": "message_example",
                }),
                &self.metadata.name,
            );
            ctx.send(message).await?;
            println!("[{}] 已发送测试消息", self.metadata.name);
        } else {
            eprintln!("[{}] 消息上下文未设置", self.metadata.name);
        }
        Ok(())
    }
}

impl Plugin for MessageExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        println!("[{}] 插件初始化", self.metadata.name);
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        println!("[{}] 插件启动", self.metadata.name);
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        println!("[{}] 插件运行中", self.metadata.name);
        
        // 发送一条测试消息（需要在异步上下文中调用）
        // 这里简化处理，实际使用时应该在异步任务中调用
        if let Some(ctx) = &self.message_context {
            let ctx = Arc::clone(ctx);
            let plugin_name = self.metadata.name.clone();
            tokio::spawn(async move {
                let message = Message::from_plugin(
                    "test.message",
                    json!({
                        "content": "这是一条测试消息",
                        "from": plugin_name,
                    }),
                    &plugin_name,
                );
                if let Err(e) = ctx.send(message).await {
                    eprintln!("发送消息失败: {}", e);
                }
            });
        }
        
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!(
            "[{}] 插件停止，共接收 {} 条消息",
            self.metadata.name, self.received_count
        );
        
        // 取消所有消息接收任务
        for handle in self.message_task_handles.drain(..) {
            handle.abort();
        }
        
        Ok(())
    }
}

impl MessagePlugin for MessageExamplePlugin {
    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: mpsc::Sender<Message>,
    ) -> impl std::future::Future<Output = Result<Arc<MessageContext>>> + Send {
        let plugin_name = self.metadata.name.clone();
        let distribution_center = Arc::new(distribution_center.clone());
        
        async move {
            // 创建消息上下文
            let ctx = Arc::new(MessageContext::new(
                Arc::clone(&distribution_center),
                &plugin_name,
                message_tx,
            ));
            
            // 订阅 "command" 类型的消息
            let mut command_rx = ctx.subscribe("command").await;
            let plugin_name_clone = plugin_name.clone();
            let ctx_clone = Arc::clone(&ctx);
            
            let handle1 = tokio::spawn(async move {
                while let Ok(message) = command_rx.recv().await {
                    println!(
                        "[{}] 收到命令消息: {}",
                        plugin_name_clone,
                        message.payload
                    );
                }
            });
            
            // 订阅 "notification" 类型的消息
            let mut notification_rx = ctx.subscribe("notification").await;
            let plugin_name_clone2 = plugin_name.clone();
            
            let handle2 = tokio::spawn(async move {
                while let Ok(message) = notification_rx.recv().await {
                    println!(
                        "[{}] 收到通知消息: {}",
                        plugin_name_clone2,
                        message.payload
                    );
                }
            });
            
            // 保存任务句柄（注意：这里无法直接保存到 self，因为是在 async 块中）
            // 实际使用时，可以通过其他方式管理这些任务
            
            Ok(ctx)
        }
    }
}

impl Default for MessageExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}
