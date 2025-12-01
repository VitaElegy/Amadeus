use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 用户唯一标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 平台标识符 (e.g., "discord", "qq", "cli")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlatformId(pub String);

/// 平台特定的用户ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlatformUserId(pub String);

/// 权限定义
/// 格式通常为: "resource:action" 或 "scope:resource:action"
/// e.g. "memo:create", "system:admin"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(pub String);

impl Permission {
    pub fn new(p: impl Into<String>) -> Self {
        Self(p.into())
    }

    /// 检查权限是否匹配 (支持通配符 *)
    /// "memo:*" 匹配 "memo:create", "memo:read"
    pub fn matches(&self, required: &str) -> bool {
        if self.0 == "*" {
            return true;
        }
        let self_parts: Vec<&str> = self.0.split(':').collect();
        let req_parts: Vec<&str> = required.split(':').collect();

        for (i, part) in self_parts.iter().enumerate() {
            if *part == "*" {
                return true;
            }
            if i >= req_parts.len() || *part != req_parts[i] {
                return false;
            }
        }
        true
    }
}

/// 用户基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 全局唯一用户ID (Amadeus内部ID)
    pub id: UserId,
    /// 显示名称
    pub name: String,
    /// 来源平台
    pub platform: PlatformId,
    /// 平台侧的用户ID
    pub platform_user_id: PlatformUserId,
}

/// 用户上下文 - 注入到消息中
/// 包含用户身份和当次请求的权限快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// 用户基本信息
    pub user: UserInfo,
    /// 用户拥有的角色列表
    pub roles: Vec<String>,
    /// 用户拥有的具体权限集合
    pub permissions: HashSet<Permission>,
    /// 会话/Token过期时间 (Unix Timestamp, Optional)
    pub expires_at: Option<u64>,
}

impl UserContext {
    pub fn new(user: UserInfo) -> Self {
        Self {
            user,
            roles: Vec::new(),
            permissions: HashSet::new(),
            expires_at: None,
        }
    }

    pub fn with_permission(mut self, perm: impl Into<String>) -> Self {
        self.permissions.insert(Permission::new(perm));
        self
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 检查是否拥有指定权限
    pub fn has_permission(&self, required_perm: &str) -> bool {
        // 1. 检查 Admin 角色 (硬编码超级管理员)
        if self.roles.contains(&"admin".to_string()) || self.roles.contains(&"root".to_string()) {
            return true;
        }
        
        // 2. 检查具体权限
        self.permissions.iter().any(|p| p.matches(required_perm))
    }
}

