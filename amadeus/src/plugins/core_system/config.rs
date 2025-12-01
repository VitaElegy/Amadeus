use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreSystemConfig {
    pub memos: MemoConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoConfig {
    /// 不同优先级的配置
    pub priorities: HashMap<i32, PriorityConfig>,
    /// 默认过期策略 (单位: 天) - 备忘录过期多久后自动回收/删除
    pub expiration_days: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriorityConfig {
    pub name: String,
    pub color: String, // e.g. "#FF0000" or "red"
    pub default_reminder_message: String,
}

impl Default for CoreSystemConfig {
    fn default() -> Self {
        let mut priorities = HashMap::new();
        priorities.insert(0, PriorityConfig {
            name: "Low".to_string(),
            color: "gray".to_string(),
            default_reminder_message: "You have a low priority task pending: {content}".to_string(),
        });
        priorities.insert(1, PriorityConfig {
            name: "Normal".to_string(),
            color: "blue".to_string(),
            default_reminder_message: "Reminder: {content}".to_string(),
        });
        priorities.insert(2, PriorityConfig {
            name: "High".to_string(),
            color: "orange".to_string(),
            default_reminder_message: "Important! Don't forget: {content}".to_string(),
        });
        priorities.insert(3, PriorityConfig {
            name: "Critical".to_string(),
            color: "red".to_string(),
            default_reminder_message: "URGENT: {content} is due!".to_string(),
        });

        Self {
            memos: MemoConfig {
                priorities,
                expiration_days: 30, // Default retain for 30 days after expiration
            },
        }
    }
}

