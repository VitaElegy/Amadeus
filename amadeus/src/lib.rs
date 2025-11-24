// Amadeus 插件系统核心库

pub mod app;
pub mod dispatcher;
pub mod distribution_center;
pub mod message;
pub mod message_context;
pub mod message_manager;
pub mod plugin;
pub mod plugins;

// 重新导出常用类型
pub use app::App;
pub use dispatcher::{Dispatcher, DispatcherRegistry};
pub use distribution_center::DistributionCenter;
pub use message::{Message, MessageHandleResult, MessagePriority, MessageSource, MessageType};
pub use message_context::MessageContext;
pub use message_manager::MessageManager;
pub use plugin::{MessagePlugin, Plugin, PluginMetadata, PluginRegistry};

