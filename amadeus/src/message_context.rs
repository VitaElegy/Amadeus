use crate::distribution_center::DistributionCenter;
use crate::message::{Message, MessageType};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 消息上下文
/// 
/// 为插件提供消息订阅和发送的便捷接口
/// 使用 tokio 的异步通道
pub struct MessageContext {
    distribution_center: Arc<DistributionCenter>,
    plugin_name: String,
    /// 消息发送通道（用于发送消息到分发中心）
    message_tx: tokio::sync::mpsc::Sender<Message>,
}

impl MessageContext {
    /// 创建新的消息上下文
    pub fn new(
        distribution_center: Arc<DistributionCenter>,
        plugin_name: impl Into<String>,
        message_tx: tokio::sync::mpsc::Sender<Message>,
    ) -> Self {
        Self {
            distribution_center,
            plugin_name: plugin_name.into(),
            message_tx,
        }
    }

    /// 订阅消息类型
    /// 
    /// # 参数
    /// - `message_type`: 要订阅的消息类型
    /// 
    /// # 返回值
    /// - 返回一个广播接收器，用于接收该类型的公共消息
    pub async fn subscribe(&self, message_type: impl Into<MessageType>) -> broadcast::Receiver<Message> {
        self.distribution_center
            .subscribe(message_type, &self.plugin_name)
            .await
    }

    /// 订阅所有公共消息
    /// 
    /// # 返回值
    /// - 返回一个广播接收器，用于接收所有公共消息
    pub async fn subscribe_all(&self) -> broadcast::Receiver<Message> {
        self.distribution_center
            .subscribe_all(&self.plugin_name)
            .await
    }

    /// 启用定向消息接收
    /// 
    /// 注册当前插件的定向消息通道，允许其他插件向此插件发送私密消息
    /// 
    /// # 返回值
    /// - 返回一个 mpsc 接收器，用于接收定向给此插件的消息
    pub async fn enable_direct_messaging(&self) -> tokio::sync::mpsc::Receiver<Message> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.distribution_center.register_direct_channel(&self.plugin_name, tx).await;
        rx
    }

    /// 发送消息
    /// 
    /// 消息会被分发中心路由给所有订阅了该消息类型的插件和分发器
    pub async fn send(&self, mut message: Message) -> Result<()> {
        // 确保消息来源设置为当前插件
        message.source = crate::message::MessageSource::Plugin(self.plugin_name.clone());
        
        // 通过通道发送消息
        self.message_tx.send(message).await
            .map_err(|e| anyhow::anyhow!("发送消息失败: {}", e))?;
        
        Ok(())
    }

    /// 获取分发中心的引用
    pub fn distribution_center(&self) -> &Arc<DistributionCenter> {
        &self.distribution_center
    }

    /// 获取插件名称
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }
}

impl Clone for MessageContext {
    fn clone(&self) -> Self {
        Self {
            distribution_center: Arc::clone(&self.distribution_center),
            plugin_name: self.plugin_name.clone(),
            message_tx: self.message_tx.clone(),
        }
    }
}
