// 示例插件 - 展示多文件插件结构
mod config;
mod handler;

use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;
pub use config::PluginConfig;
pub use handler::DataHandler;

/// 示例插件 - 展示如何构建复杂的多文件插件
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    config: PluginConfig,
    handler: DataHandler,
    is_initialized: bool,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "example_plugin",
            "一个示例插件，展示多文件插件结构",
            "0.1.0",
        )
        .enabled_by_default(false)  // 示例插件默认禁用
        .author("Amadeus Team")
        .with_property("category", "example")
        .with_property("complexity", "multi-file");

        Self {
            metadata,
            config: PluginConfig::default(),
            handler: DataHandler::new(),
            is_initialized: false,
        }
    }
}

impl Plugin for ExamplePlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        println!("[ExamplePlugin] 初始化中...");
        
        // 加载配置
        self.config.load()?;
        println!("[ExamplePlugin] 配置已加载: {:?}", self.config);
        
        // 初始化处理器
        self.handler.init()?;
        
        self.is_initialized = true;
        println!("[ExamplePlugin] 初始化完成!");
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        if !self.is_initialized {
            anyhow::bail!("插件未初始化");
        }

        println!("[ExamplePlugin] 启动中...");
        self.handler.start()?;
        
        // 模拟运行主要逻辑
        println!("[ExamplePlugin] 运行主要逻辑...");
        // 使用处理器处理数据
        let data = vec!["数据1", "数据2", "数据3"];
        // Clone handler if needed or ensure thread safety, but here it's simple
        // For sync logic we can keep it here, or spawn if slow.
        self.handler.process(&data)?;
        println!("[ExamplePlugin] 任务完成!");

        println!("[ExamplePlugin] 已启动!");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!("[ExamplePlugin] 停止中...");
        self.handler.cleanup()?;
        self.is_initialized = false;
        println!("[ExamplePlugin] 已停止!");
        Ok(())
    }
}

