use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
        println!("[{}] 插件初始化", self.metadata().name);
        Ok(())
    }

    /// 启动插件 - 在插件开始运行前调用
    fn start(&mut self) -> anyhow::Result<()> {
        println!("[{}] 插件启动", self.metadata().name);
        Ok(())
    }

    /// 运行插件 - 插件的主要逻辑
    fn run(&mut self) -> anyhow::Result<()> {
        println!("[{}] 插件运行中", self.metadata().name);
        Ok(())
    }

    /// 停止插件 - 在插件停止时调用，用于清理资源
    fn stop(&mut self) -> anyhow::Result<()> {
        println!("[{}] 插件停止", self.metadata().name);
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
    /// 
    /// # 示例
    /// ```no_run
    /// let configs = PluginRegistry::load_config("plugins_config.json")?;
    /// for config in configs {
    ///     println!("加载插件配置: {}", config.name);
    /// }
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
        println!("注册插件: {}", plugin.metadata().name);
        self.plugins.push(Box::new(plugin));
    }

    /// 批量注册插件列表
    pub fn register_all(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        for plugin in plugins {
            println!("注册插件: {}", plugin.metadata().name);
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
                println!("✓ 注册插件: {} [启用]", name);
                self.plugins.push(plugin);
            } else {
                println!("✗ 跳过插件: {} [禁用]", name);
            }
        }
    }

    /// 注册匹配名称的插件
    pub fn register_by_names(&mut self, plugins: Vec<Box<dyn Plugin>>, names: &[&str]) {
        for plugin in plugins {
            let name = &plugin.metadata().name;
            if names.contains(&name.as_str()) {
                println!("✓ 注册插件: {}", name);
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
                println!("✓ 注册插件: {}", meta.name);
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
        println!("\n=== 初始化所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.init()?;
        }
        Ok(self)
    }

    /// 启动所有插件
    pub fn start_all(&mut self) -> anyhow::Result<&mut Self> {
        println!("\n=== 启动所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.start()?;
        }
        Ok(self)
    }

    /// 运行所有插件
    pub fn run_all(&mut self) -> anyhow::Result<&mut Self> {
        println!("\n=== 运行所有插件 ===");
        for plugin in self.plugins.iter_mut() {
            plugin.run()?;
        }
        Ok(self)
    }

    /// 停止所有插件（按相反顺序）
    pub fn stop_all(&mut self) -> anyhow::Result<&mut Self> {
        println!("\n=== 停止所有插件 ===");
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
        println!("\n=== 已注册的插件 ===");
        for (idx, plugin) in self.plugins.iter().enumerate() {
            let meta = plugin.metadata();
            println!(
                "{}. {} v{} - {} [{}]",
                idx + 1,
                meta.name,
                meta.version,
                meta.description,
                if meta.enabled_by_default { "启用" } else { "禁用" }
            );
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

