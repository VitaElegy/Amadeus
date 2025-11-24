// iceoryx2 消息传输数据结构
// 这些结构用于在 iceoryx2 服务之间传输消息

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

/// 用于 iceoryx2 传输的消息数据
///
/// 此结构实现了 ZeroCopySend，可以直接在 iceoryx2 中传输
/// 包含消息的序列化 JSON 数据，接收端可以反序列化为 Message 对象
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AmadeusMessageData {
    /// 消息类型（最大长度 64 字符）
    pub message_type: [u8; 64],
    /// 消息类型实际长度
    pub message_type_len: u8,
    /// 消息 JSON 数据（最大 4096 字节）
    pub json_data: [u8; 4096],
    /// JSON 数据实际长度
    pub json_data_len: u16,
    /// 优先级（0=Low, 1=Normal, 2=High, 3=Critical）
    pub priority: u8,
    /// 时间戳（Unix 时间戳，毫秒）
    pub timestamp: u64,
}

impl AmadeusMessageData {
    /// 创建新的消息数据
    pub fn new() -> Self {
        Self {
            message_type: [0; 64],
            message_type_len: 0,
            json_data: [0; 4096],
            json_data_len: 0,
            priority: 1, // Normal
            timestamp: 0,
        }
    }

    /// 从 JSON 字符串创建消息数据
    /// 
    /// # 参数
    /// - `message_type`: 消息类型字符串
    /// - `json`: 消息的 JSON 字符串
    /// - `priority`: 优先级
    /// - `timestamp`: 时间戳
    pub fn from_json(
        message_type: &str,
        json: &str,
        priority: u8,
        timestamp: u64,
    ) -> Result<Self, String> {
        let mut data = Self::new();
        
        // 设置消息类型
        let type_bytes = message_type.as_bytes();
        if type_bytes.len() > 64 {
            return Err(format!("消息类型过长: {} > 64", type_bytes.len()));
        }
        data.message_type[..type_bytes.len()].copy_from_slice(type_bytes);
        data.message_type_len = type_bytes.len() as u8;
        
        // 设置 JSON 数据
        let json_bytes = json.as_bytes();
        if json_bytes.len() > 4096 {
            return Err(format!("JSON 数据过长: {} > 4096", json_bytes.len()));
        }
        data.json_data[..json_bytes.len()].copy_from_slice(json_bytes);
        data.json_data_len = json_bytes.len() as u16;
        
        data.priority = priority;
        data.timestamp = timestamp;
        
        Ok(data)
    }

    /// 获取消息类型字符串
    pub fn message_type_str(&self) -> Result<String, String> {
        if self.message_type_len == 0 || self.message_type_len > 64 {
            return Err("无效的消息类型长度".to_string());
        }
        String::from_utf8(
            self.message_type[..self.message_type_len as usize].to_vec()
        ).map_err(|e| format!("无效的 UTF-8: {}", e))
    }

    /// 获取 JSON 字符串
    pub fn json_str(&self) -> Result<String, String> {
        if self.json_data_len == 0 || self.json_data_len > 4096 {
            return Err("无效的 JSON 数据长度".to_string());
        }
        String::from_utf8(
            self.json_data[..self.json_data_len as usize].to_vec()
        ).map_err(|e| format!("无效的 UTF-8: {}", e))
    }
}

impl Default for AmadeusMessageData {
    fn default() -> Self {
        Self::new()
    }
}

// 手动实现 ZeroCopySend trait
unsafe impl ZeroCopySend for AmadeusMessageData {
    unsafe fn type_name() -> &'static str {
        "AmadeusMessage"
    }
}


/// iceoryx2 服务名称常量
pub mod service_names {
    /// 默认的 amadeus 服务名称
    pub const AMADEUS_SERVICE: &str = "Amadeus/Message/Service";
    
    /// 根据消息类型生成服务名称
    pub fn for_message_type(message_type: &str) -> String {
        format!("Amadeus/Message/{}", message_type.replace(" ", "_"))
    }
}

