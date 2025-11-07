// Amadeus 插件系统核心库

pub mod app;
pub mod plugin;
pub mod plugins;

// 重新导出常用类型
pub use app::App;
pub use plugin::{Plugin, PluginMetadata, PluginRegistry};

