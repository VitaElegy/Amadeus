use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use crate::message_context::MessageContext;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
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

    /// 启动插件 - 在插件开始运行前调用
    fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("[{}] 插件启动", self.metadata().name);
        Ok(())
    }

    /// 运行插件 - 插件的主要逻辑
    fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("[{}] 插件运行中", self.metadata().name);
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

/// 支持消息的插件 trait
/// 
/// 这是一个可选的扩展 trait，插件可以实现它来获得消息订阅和发送的能力
/// 
/// 注意：实现此 trait 的插件必须同时实现 Plugin trait
/// 使用 tokio 的异步通道实现消息订阅和发送
pub trait MessagePlugin: Plugin {
    /// 设置消息订阅（在插件初始化时调用）
    /// 
    /// 插件可以在这里订阅感兴趣的消息类型
    /// 
    /// # 参数
    /// - `distribution_center`: 分发中心的引用，用于订阅消息
    /// - `message_tx`: 消息发送通道，用于发送消息
    /// 
    /// # 返回值
    /// 返回消息上下文，插件可以使用它来订阅和发送消息
    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: mpsc::Sender<Message>,
    ) -> impl Future<Output = anyhow::Result<Arc<MessageContext>>> + Send {
        // 默认实现：创建消息上下文但不订阅任何消息
        let plugin_name = self.metadata().name.clone();
        let distribution_center = Arc::new(distribution_center.clone());
        
        async move {
            let ctx = Arc::new(MessageContext::new(
                distribution_center,
                plugin_name,
                message_tx,
            ));
            Ok(ctx)
        }
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
    /// 
    /// # 示例
    /// ```no_run
    /// use amadeus::plugin::PluginRegistry;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let configs = PluginRegistry::load_config("plugins_config.json")?;
    /// for config in configs {
    ///     println!("加载插件配置: {}", config.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// 
    /// 只注册那些 enabled_by_default 为 true 的插件
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

    /// 运行所有插件
    pub fn run_all(&mut self) -> anyhow::Result<&mut Self> {
        tracing::info!("=== 运行所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.run()?;
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

    /// 执行完整的插件生命周期
    pub fn run_lifecycle(&mut self) -> anyhow::Result<()> {
        self.init_all()?
            .start_all()?
            .run_all()?
            .stop_all()?;
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

    /// 设置插件的消息订阅（如果插件实现了 MessagePlugin）
    /// 
    /// 注意：由于 Rust 的类型系统限制，此方法需要插件在注册时明确类型
    /// 或者使用类型擦除技术。这里提供一个基础框架。
    /// 
    /// 实际使用中，如果插件实现了 MessagePlugin，应该在插件的 init() 或 start() 方法中
    /// 通过其他方式（如全局注册表或依赖注入）获取分发中心并设置订阅
    pub fn setup_messaging(
        &mut self,
        _message_manager: &crate::message_manager::MessageManager,
    ) -> anyhow::Result<()> {
        tracing::info!("=== 设置插件消息订阅 ===");
        
        // 遍历所有插件，尝试设置消息订阅
        // 注意：由于 trait object 的限制，这里无法直接检查插件是否实现了 MessagePlugin
        // 实际使用中，需要使用 MessagePluginWrapper 或者类型擦除
        
        // 这里提供一个占位实现，实际使用时需要根据具体的设计模式来实现
        // 如果插件被包装在 MessagePluginWrapper 中，可以在这里设置
        // 或者插件在 init/start 时通过其他方式获取分发中心
        
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

