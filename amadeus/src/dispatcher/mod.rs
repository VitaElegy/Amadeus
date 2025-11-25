pub mod iceoryx2;

use crate::message::{Message, MessageType};
use anyhow::Result;
use std::collections::HashSet;
use tracing::{info, warn};

/// 分发器 trait
/// 
/// 分发器负责与外界交互（如前端、QQ bot等），
/// 接收外部消息并转发给分发中心，同时接收分发中心的消息并发送给外界
pub trait Dispatcher: Send + Sync {
    /// 分发器名称
    fn name(&self) -> &str;

    /// 启动分发器
    fn start(&mut self) -> Result<()>;

    /// 停止分发器
    fn stop(&mut self) -> Result<()>;

    /// 发送消息到外部
    /// 
    /// # 参数
    /// - `message`: 要发送的消息
    fn send_message(&self, message: &Message) -> Result<()>;

    /// 检查分发器是否正在运行
    fn is_running(&self) -> bool;

    /// 获取分发器订阅的消息类型列表
    /// 
    /// 返回空集合表示订阅所有消息类型
    /// 返回非空集合表示只订阅指定的消息类型
    fn subscribed_message_types(&self) -> HashSet<MessageType> {
        // 默认实现：订阅所有消息类型
        HashSet::new()
    }

    /// 检查分发器是否订阅了指定的消息类型
    /// 
    /// # 参数
    /// - `message_type`: 消息类型
    /// 
    /// # 返回值
    /// - `true`: 如果订阅了该消息类型或订阅所有消息类型
    /// - `false`: 如果没有订阅该消息类型
    fn is_subscribed_to(&self, message_type: &MessageType) -> bool {
        let subscribed = self.subscribed_message_types();
        // 空集合表示订阅所有消息类型
        subscribed.is_empty() || subscribed.contains(message_type)
    }
}

/// 分发器注册表
pub struct DispatcherRegistry {
    dispatchers: Vec<Box<dyn Dispatcher>>,
}

impl DispatcherRegistry {
    /// 创建新的分发器注册表
    pub fn new() -> Self {
        Self {
            dispatchers: Vec::new(),
        }
    }

    /// 注册分发器
    pub fn register<D: Dispatcher + 'static>(&mut self, dispatcher: D) {
        info!("注册分发器: {}", dispatcher.name());
        self.dispatchers.push(Box::new(dispatcher));
    }

    /// 获取所有分发器
    pub fn dispatchers(&self) -> &[Box<dyn Dispatcher>] {
        &self.dispatchers
    }

    /// 获取所有分发器的可变引用
    pub fn dispatchers_mut(&mut self) -> &mut [Box<dyn Dispatcher>] {
        &mut self.dispatchers
    }

    /// 启动所有分发器
    pub fn start_all(&mut self) -> Result<()> {
        info!("=== 启动所有分发器 ===");
        for dispatcher in self.dispatchers.iter_mut() {
            dispatcher.start()?;
            info!("✓ 分发器 {} 已启动", dispatcher.name());
        }
        Ok(())
    }

    /// 停止所有分发器
    pub fn stop_all(&mut self) -> Result<()> {
        info!("=== 停止所有分发器 ===");
        for dispatcher in self.dispatchers.iter_mut().rev() {
            if let Err(e) = dispatcher.stop() {
                warn!("! 分发器 {} 停止时出错: {}", dispatcher.name(), e);
            } else {
                info!("✓ 分发器 {} 已停止", dispatcher.name());
            }
        }
        Ok(())
    }
}

impl Default for DispatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

