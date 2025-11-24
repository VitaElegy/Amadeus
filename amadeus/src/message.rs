use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

/// 消息类型标识符
/// 插件通过消息类型来订阅感兴趣的消息
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageType(pub String);

impl MessageType {
    /// 创建新的消息类型
    pub fn new(ty: impl Into<String>) -> Self {
        Self(ty.into())
    }

    /// 获取消息类型字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MessageType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 消息来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    /// 来自外部（通过分发器）
    External(String),
    /// 来自插件
    Plugin(String),
    /// 来自系统内部
    System,
}

/// 统一的消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息类型
    pub message_type: MessageType,
    /// 消息内容（JSON格式）
    pub payload: serde_json::Value,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 消息来源
    pub source: MessageSource,
    /// 消息时间戳（Unix时间戳，毫秒）
    pub timestamp: u64,
    /// 消息ID（可选，用于追踪）
    pub message_id: Option<String>,
    /// 元数据（可选）
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl Message {
    /// 创建新消息
    pub fn new(
        message_type: impl Into<MessageType>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            message_type: message_type.into(),
            payload,
            priority: MessagePriority::default(),
            source: MessageSource::System,
            timestamp: Self::current_timestamp(),
            message_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 从外部创建消息
    pub fn from_external(
        message_type: impl Into<MessageType>,
        payload: serde_json::Value,
        source: impl Into<String>,
    ) -> Self {
        Self {
            message_type: message_type.into(),
            payload,
            priority: MessagePriority::default(),
            source: MessageSource::External(source.into()),
            timestamp: Self::current_timestamp(),
            message_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 从插件创建消息
    pub fn from_plugin(
        message_type: impl Into<MessageType>,
        payload: serde_json::Value,
        plugin_name: impl Into<String>,
    ) -> Self {
        Self {
            message_type: message_type.into(),
            payload,
            priority: MessagePriority::default(),
            source: MessageSource::Plugin(plugin_name.into()),
            timestamp: Self::current_timestamp(),
            message_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置消息ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = Some(id.into());
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// 将消息序列化为JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// 从JSON反序列化消息
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub enum MessageHandleResult {
    /// 消息已成功处理
    Handled,
    /// 消息被忽略（插件不感兴趣）
    Ignored,
    /// 处理失败
    Failed(String),
}

