use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use crate::message_context::MessageContext;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::mpsc;

/// 插件元数据 - 可以被序列化到 JSON 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 是否默认启用
    pub enabled_by_default: bool,
    /// 插件作者
    pub author: Option<String>,
    /// 其他自定义属性
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

impl PluginMetadata {
    /// 创建一个新的插件元数据
    pub fn new(name: &str, description: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            enabled_by_default: true,
            author: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// 设置是否默认启用
    pub fn enabled_by_default(mut self, enabled: bool) -> Self {
        self.enabled_by_default = enabled;
        self
    }

    /// 设置作者
    pub fn author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }

    /// 添加自定义属性
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }
}

/// 插件生命周期 trait
/// 所有插件必须实现此 trait
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> &PluginMetadata;

    /// 初始化插件 - 在插件加载时调用
    fn init(&mut self) -> anyhow::Result<()> {
        tracing::info!("[{}] 插件初始化", self.metadata().name);
        Ok(())
    }

    /// 设置消息订阅
    /// 
    /// 插件可以在这里订阅感兴趣的消息类型
    /// 返回一个 Future，因为订阅可能涉及异步操作
    fn setup_messaging(
        &mut self,
        _distribution_center: &DistributionCenter,
        _message_tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Arc<MessageContext>>>> + Send>> {
        // 默认实现：不订阅任何消息
        Box::pin(async { Ok(None) })
    }

    /// 启动插件 - 在插件开始运行前调用
    fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("[{}] 插件启动", self.metadata().name);
        Ok(())
    }

    /// 停止插件 - 在插件停止时调用，用于清理资源
    fn stop(&mut self) -> anyhow::Result<()> {
        tracing::info!("[{}] 插件停止", self.metadata().name);
        Ok(())
    }

    /// 获取插件是否启用
    fn is_enabled(&self) -> bool {
        self.metadata().enabled_by_default
    }
}

/// 插件注册表 - 管理所有插件
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 创建插件注册表并自动加载所有启用的插件
    pub fn with_enabled_plugins(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let mut registry = Self::new();
        registry.register_enabled(plugins);
        registry
    }

    /// 创建插件注册表并加载所有插件（无论是否启用）
    pub fn with_all_plugins(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let mut registry = Self::new();
        registry.register_all(plugins);
        registry
    }

    /// 从配置文件加载插件元数据
    pub fn load_config(path: &str) -> anyhow::Result<Vec<PluginMetadata>> {
        let content = std::fs::read_to_string(path)?;
        let metadata: Vec<PluginMetadata> = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// 保存插件元数据到配置文件
    pub fn save_config(path: &str, metadata: &[PluginMetadata]) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(metadata)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 注册一个插件
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        tracing::info!("注册插件: {}", plugin.metadata().name);
        self.plugins.push(Box::new(plugin));
    }

    /// 批量注册插件列表
    pub fn register_all(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        for plugin in plugins {
            tracing::info!("注册插件: {}", plugin.metadata().name);
            self.plugins.push(plugin);
        }
    }

    /// 根据配置有选择地注册插件
    pub fn register_enabled(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        for plugin in plugins {
            let enabled = plugin.metadata().enabled_by_default;
            let name = plugin.metadata().name.clone();
            
            if enabled {
                tracing::info!("✓ 注册插件: {} [启用]", name);
                self.plugins.push(plugin);
            } else {
                tracing::info!("✗ 跳过插件: {} [禁用]", name);
            }
        }
    }

    /// 注册匹配名称的插件
    pub fn register_by_names(&mut self, plugins: Vec<Box<dyn Plugin>>, names: &[&str]) {
        for plugin in plugins {
            let name = &plugin.metadata().name;
            if names.contains(&name.as_str()) {
                tracing::info!("✓ 注册插件: {}", name);
                self.plugins.push(plugin);
            }
        }
    }

    /// 注册匹配过滤器的插件
    pub fn register_filtered<F>(&mut self, plugins: Vec<Box<dyn Plugin>>, filter: F)
    where
        F: Fn(&PluginMetadata) -> bool,
    {
        for plugin in plugins {
            let meta = plugin.metadata();
            if filter(meta) {
                tracing::info!("✓ 注册插件: {}", meta.name);
                self.plugins.push(plugin);
            }
        }
    }

    /// 获取所有插件
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// 获取可变引用的所有插件
    pub fn plugins_mut(&mut self) -> &mut [Box<dyn Plugin>] {
        &mut self.plugins
    }

    /// 初始化所有插件
    pub fn init_all(&mut self) -> anyhow::Result<&mut Self> {
        tracing::info!("=== 初始化所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.init()?;
        }
        Ok(self)
    }

    /// 启动所有插件
    pub fn start_all(&mut self) -> anyhow::Result<&mut Self> {
        tracing::info!("=== 启动所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.start()?;
        }
        Ok(self)
    }

    /// 停止所有插件（按相反顺序）
    pub fn stop_all(&mut self) -> anyhow::Result<&mut Self> {
        tracing::info!("=== 停止所有插件 ===");
        for plugin in self.plugins.iter_mut().rev() {
            plugin.stop()?;
        }
        Ok(self)
    }

    /// 执行插件启动流程 (init -> start)
    pub fn startup(&mut self) -> anyhow::Result<()> {
        self.init_all()?
            .start_all()?;
        Ok(())
    }

    /// 执行插件停止流程 (stop)
    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        self.stop_all()?;
        Ok(())
    }

    /// 导出所有插件的元数据为 JSON
    pub fn export_metadata(&self) -> anyhow::Result<String> {
        let metadata: Vec<&PluginMetadata> = self
            .plugins
            .iter()
            .map(|p| p.metadata())
            .collect();
        Ok(serde_json::to_string_pretty(&metadata)?)
    }

    /// 列出所有插件
    pub fn list_plugins(&self) {
        tracing::info!("=== 已注册的插件 ===");
        for (idx, plugin) in self.plugins.iter().enumerate() {
            let meta = plugin.metadata();
            tracing::info!(
                "{}. {} v{} - {} [{}]",
                idx + 1,
                meta.name,
                meta.version,
                meta.description,
                if meta.enabled_by_default { "启用" } else { "禁用" }
            );
        }
    }

    /// 设置插件的消息订阅
    pub async fn setup_messaging(
        &mut self,
        message_manager: &crate::message_manager::MessageManager,
    ) -> anyhow::Result<()> {
        tracing::info!("=== 设置插件消息订阅 ===");
        
        let dc = message_manager.distribution_center();
        let tx = message_manager.message_tx();

        for plugin in self.plugins.iter_mut() {
            // 调用每个插件的 setup_messaging
            // 因为我们现在统一了接口，所以可以直接调用
            match plugin.setup_messaging(dc, tx.clone()).await {
                Ok(Some(ctx)) => {
                    tracing::info!("✓ 插件 {} 消息订阅已配置", plugin.metadata().name);
                    // 这里可能需要保存 ctx，但目前 MessageContext 主要用于订阅时的生命周期管理
                    // 如果插件自己管理了 handle，那么这里不需要做额外的事情
                    // 通常插件会在 setup_messaging 里 spawn 任务
                    
                    // 为了保持 context 存活（如果需要），我们可能需要将其 attach 到插件
                    // 但由于 dyn Plugin 不知道具体的结构，我们假设 setup_messaging 内部处理好了所有事情
                    // (例如 CoreSystemPlugin 将 ctx 存入了自己的结构体)
                    drop(ctx); 
                }
                Ok(None) => {
                    // 插件不需要消息功能
                }
                Err(e) => {
                    tracing::error!("✗ 插件 {} 消息订阅配置失败: {}", plugin.metadata().name, e);
                }
            }
        }
        
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
