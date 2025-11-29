use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

/// 消息管理器 (插件中心核心组件)
/// 
/// 负责协调插件间的消息路由和分发
/// 支持广播消息（Public）和定向消息（Direct）
pub struct MessageManager {
    distribution_center: Arc<DistributionCenter>,
    /// 消息接收通道（接收来自插件的消息）
    message_rx: Option<mpsc::Receiver<Message>>,
    message_tx: mpsc::Sender<Message>,
    /// 消息处理任务句柄
    message_task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MessageManager {
    /// 创建新的消息管理器
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        
        Self {
            distribution_center: Arc::new(DistributionCenter::new()),
            message_rx: Some(rx),
            message_tx: tx,
            message_task_handle: None,
        }
    }

    /// 获取分发中心的引用
    pub fn distribution_center(&self) -> &Arc<DistributionCenter> {
        &self.distribution_center
    }

    /// 获取消息发送通道（用于插件发送消息）
    pub fn message_tx(&self) -> mpsc::Sender<Message> {
        self.message_tx.clone()
    }

    /// 启动消息处理任务
    pub fn start_message_loop(&mut self) {
        let distribution_center = Arc::clone(&self.distribution_center);
        let mut message_rx = self.message_rx.take().expect("消息接收器已被使用");

        let handle = tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                // 检查是否为定向消息
                if let Some(recipient) = &message.recipient {
                    // 定向消息：发送给指定插件
                    if let Err(e) = distribution_center.send_direct(recipient, message.clone()).await {
                        tracing::warn!("[消息管理器] 发送定向消息失败 (目标: {}): {}", recipient, e);
                    }
                } else {
                    // 广播消息：分发给所有订阅者
                    distribution_center.distribute(&message).await;
                }
            }
        });

        self.message_task_handle = Some(handle);
    }

    /// 停止消息处理任务
    pub async fn stop_message_loop(&mut self) {
        if let Some(handle) = self.message_task_handle.take() {
            handle.abort(); 
            let _ = handle.await;
        }
    }
}

impl Default for MessageManager {
    fn default() -> Self {
        Self::new()
    }
}
