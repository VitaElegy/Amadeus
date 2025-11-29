use super::message::{Message, MessageType};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// 分发中心 - 负责消息的路由和分发
/// 
/// 使用 tokio::sync::broadcast 实现发布-订阅模式（进程内通信）：
/// - 插件可以订阅特定类型的消息
/// - 分发器可以将消息发送到分发中心
/// - 分发中心将消息广播给所有订阅了该消息类型的插件
/// 
/// 注意：此组件用于进程内通信（插件之间）。
/// 进程间通信由 Dispatcher（如 Iceoryx2Dispatcher）处理。
pub struct DistributionCenter {
    /// 消息类型到广播发送器的映射
    /// 使用 broadcast channel 实现一对多的消息分发
    channels: std::sync::Arc<tokio::sync::RwLock<HashMap<MessageType, broadcast::Sender<Message>>>>,
    /// 插件ID到定向消息发送器的映射
    direct_channels: std::sync::Arc<tokio::sync::RwLock<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>,
    /// 全局订阅者（接收所有广播消息）
    global_subscribers: std::sync::Arc<tokio::sync::RwLock<Vec<tokio::sync::broadcast::Sender<Message>>>>,
    /// 插件名称到其订阅的消息类型的映射（用于取消订阅）
    plugin_subscriptions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Vec<MessageType>>>>,
    /// 广播通道的容量（默认 1024）
    channel_capacity: usize,
}

impl DistributionCenter {
    /// 创建新的分发中心
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// 使用指定容量创建分发中心
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            direct_channels: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            global_subscribers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            plugin_subscriptions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            channel_capacity: capacity,
        }
    }

    /// 订阅所有消息（全局订阅）
    pub async fn subscribe_all(&self, _plugin_name: impl Into<String>) -> tokio::sync::broadcast::Receiver<Message> {
        let mut globals = self.global_subscribers.write().await;
        let (tx, rx) = tokio::sync::broadcast::channel(self.channel_capacity);
        globals.push(tx);
        rx
    }

    /// 注册定向消息通道
    pub async fn register_direct_channel(&self, plugin_id: impl Into<String>, sender: tokio::sync::mpsc::Sender<Message>) {
        let mut channels = self.direct_channels.write().await;
        channels.insert(plugin_id.into(), sender);
    }

    /// 发送定向消息
    pub async fn send_direct(&self, plugin_id: &str, message: Message) -> anyhow::Result<()> {
        let channels = self.direct_channels.read().await;
        if let Some(sender) = channels.get(plugin_id) {
            sender.send(message).await.map_err(|e| anyhow::anyhow!("发送定向消息失败: {}", e))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("找不到目标插件: {}", plugin_id))
        }
    }

    /// 订阅消息类型
    /// 
    /// # 参数
    /// - `message_type`: 要订阅的消息类型
    /// - `plugin_name`: 插件名称
    /// 
    /// # 返回值
    /// 返回一个接收器，用于接收该类型的消息
    pub async fn subscribe(
        &self,
        message_type: impl Into<MessageType>,
        plugin_name: impl Into<String>,
    ) -> broadcast::Receiver<Message> {
        let message_type = message_type.into();
        let plugin_name = plugin_name.into();

        // 获取或创建该消息类型的广播通道
        let mut channels = self.channels.write().await;
        let sender = channels
            .entry(message_type.clone())
            .or_insert_with(|| {
                broadcast::channel(self.channel_capacity).0
            })
            .clone();

        // 记录插件的订阅
        let mut plugin_subs = self.plugin_subscriptions.write().await;
        plugin_subs
            .entry(plugin_name)
            .or_insert_with(Vec::new)
            .push(message_type);

        // 返回接收器
        sender.subscribe()
    }

    /// 取消订阅消息类型
    pub async fn unsubscribe(&self, plugin_name: &str, message_type: &MessageType) {
        let mut plugin_subs = self.plugin_subscriptions.write().await;
        
        if let Some(types) = plugin_subs.get_mut(plugin_name) {
            types.retain(|t| t != message_type);
        }
    }

    /// 取消插件的所有订阅
    pub async fn unsubscribe_all(&self, plugin_name: &str) {
        let mut plugin_subs = self.plugin_subscriptions.write().await;
        plugin_subs.remove(plugin_name);
    }

    /// 分发消息给所有订阅者
    /// 
    /// # 返回值
    /// 返回发送的消息数量（订阅者数量）
    pub async fn distribute(&self, message: &Message) -> usize {
        let mut count = 0;
        
        // 1. 发送给特定类型的订阅者
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&message.message_type) {
            count += sender.receiver_count();
            let _ = sender.send(message.clone());
        }

        // 2. 发送给全局订阅者
        let globals = self.global_subscribers.read().await;
        for sender in globals.iter() {
            count += sender.receiver_count();
            let _ = sender.send(message.clone());
        }

        count
    }

    /// 获取订阅统计信息
    pub async fn get_subscription_stats(&self) -> HashMap<String, usize> {
        let channels = self.channels.read().await;
        let mut stats = HashMap::new();
        
        for (message_type, sender) in channels.iter() {
            stats.insert(message_type.as_str().to_string(), sender.receiver_count());
        }
        
        stats
    }

    /// 获取插件订阅的消息类型列表
    pub async fn get_plugin_subscriptions(&self, plugin_name: &str) -> Vec<MessageType> {
        let plugin_subs = self.plugin_subscriptions.read().await;
        plugin_subs
            .get(plugin_name)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for DistributionCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DistributionCenter {
    fn clone(&self) -> Self {
        Self {
            channels: std::sync::Arc::clone(&self.channels),
            direct_channels: std::sync::Arc::clone(&self.direct_channels),
            global_subscribers: std::sync::Arc::clone(&self.global_subscribers),
            plugin_subscriptions: std::sync::Arc::clone(&self.plugin_subscriptions),
            channel_capacity: self.channel_capacity,
        }
    }
}
