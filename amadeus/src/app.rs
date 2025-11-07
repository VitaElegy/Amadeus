use crate::plugin::{Plugin, PluginRegistry};
use anyhow::Result;

/// Amadeus 应用构建器
/// 
/// 提供更优雅的方式来配置和运行插件系统
pub struct App {
    registry: PluginRegistry,
    show_metadata: bool,
    show_startup_message: bool,
}

impl App {
    /// 创建新的应用实例，自动加载所有启用的插件
    pub fn new() -> Self {
        let registry = PluginRegistry::with_enabled_plugins(
            crate::plugins::get_all_plugins()
        );
        
        Self {
            registry,
            show_metadata: false,
            show_startup_message: true,
        }
    }

    /// 使用自定义插件列表创建应用
    pub fn with_plugins(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let registry = PluginRegistry::with_enabled_plugins(plugins);
        
        Self {
            registry,
            show_metadata: false,
            show_startup_message: true,
        }
    }

    /// 加载所有插件（无论是否启用）
    pub fn with_all_plugins() -> Self {
        let registry = PluginRegistry::with_all_plugins(
            crate::plugins::get_all_plugins()
        );
        
        Self {
            registry,
            show_metadata: false,
            show_startup_message: true,
        }
    }

    /// 设置是否显示元数据
    pub fn show_metadata(mut self, show: bool) -> Self {
        self.show_metadata = show;
        self
    }

    /// 设置是否显示启动消息
    pub fn show_startup_message(mut self, show: bool) -> Self {
        self.show_startup_message = show;
        self
    }

    /// 获取插件注册表的可变引用
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }

    /// 运行应用
    pub fn run(mut self) -> Result<()> {
        if self.show_startup_message {
            println!("=== Amadeus 插件系统启动 ===\n");
        }

        // 列出插件
        self.registry.list_plugins();

        // 可选：显示元数据
        if self.show_metadata {
            if let Ok(json) = self.registry.export_metadata() {
                println!("\n=== 插件元数据 (JSON) ===");
                println!("{}", json);
            }
        }

        // 执行插件生命周期
        self.registry.run_lifecycle()?;

        if self.show_startup_message {
            println!("\n=== Amadeus 插件系统已关闭 ===");
        }
        
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

