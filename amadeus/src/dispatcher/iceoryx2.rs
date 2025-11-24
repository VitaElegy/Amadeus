use crate::dispatcher::Dispatcher;
use crate::message::{Message, MessageType};
use anyhow::Result;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Iceoryx2 分发器实现
/// 
/// 使用 iceoryx2 进行零拷贝进程间通信
pub struct Iceoryx2Dispatcher {
    name: String,
    service_name: String,
    running: Arc<AtomicBool>,
    /// 订阅的消息类型集合（空集合表示订阅所有消息类型）
    subscribed_types: HashSet<MessageType>,
    // 这里可以添加 iceoryx2 的具体实现
    // 由于 iceoryx2 的 API 可能比较复杂，这里提供一个基础框架
}

impl Iceoryx2Dispatcher {
    /// 创建新的 Iceoryx2 分发器
    /// 
    /// 默认订阅所有消息类型
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            name: "Iceoryx2Dispatcher".to_string(),
            service_name: service_name.into(),
            running: Arc::new(AtomicBool::new(false)),
            subscribed_types: HashSet::new(), // 空集合表示订阅所有消息类型
        }
    }

    /// 设置分发器名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 订阅指定的消息类型
    /// 
    /// # 参数
    /// - `message_types`: 要订阅的消息类型列表
    /// 
    /// # 示例
    /// ```rust
    /// let dispatcher = Iceoryx2Dispatcher::new("service")
    ///     .subscribe_to(&["notification", "alert"]);
    /// ```
    pub fn subscribe_to(mut self, message_types: &[&str]) -> Self {
        self.subscribed_types = message_types
            .iter()
            .map(|s| MessageType::from(*s))
            .collect();
        self
    }

    /// 订阅单个消息类型
    pub fn subscribe_to_one(mut self, message_type: impl Into<MessageType>) -> Self {
        self.subscribed_types.insert(message_type.into());
        self
    }
}

impl Dispatcher for Iceoryx2Dispatcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // TODO: 初始化 iceoryx2 连接
        // 这里需要根据 iceoryx2 的实际 API 来实现
        println!("[{}] 初始化 Iceoryx2 服务: {}", self.name, self.service_name);
        
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // TODO: 清理 iceoryx2 资源
        println!("[{}] 停止 Iceoryx2 服务", self.name);
        
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn send_message(&self, message: &Message) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("分发器未运行"));
        }

        // TODO: 通过 iceoryx2 发送消息
        // 这里需要将 Message 序列化并通过 iceoryx2 发送
        let json = message.to_json()?;
        println!("[{}] 通过 Iceoryx2 发送消息: {}", self.name, json);
        
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn subscribed_message_types(&self) -> HashSet<MessageType> {
        self.subscribed_types.clone()
    }
}

