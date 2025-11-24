pub mod code4rena;
pub mod example_plugin;
pub mod core_system;
pub mod message_example;

use crate::plugin::Plugin;
use code4rena::Code4renaPlugin;
use example_plugin::ExamplePlugin;
use core_system::CoreSystemPlugin;
use message_example::MessageExamplePlugin;

/// 获取所有可用的插件实例
///
/// 这个函数会自动创建所有插件的实例并返回
/// 新增插件时，只需要在这里添加即可
pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        // Core System Plugin - always active
        Box::new(CoreSystemPlugin::new("sqlite:amadeus.db")),
        Box::new(Code4renaPlugin::new()),
        Box::new(ExamplePlugin::new()),
        Box::new(MessageExamplePlugin::new()),
        // 在这里添加更多插件
        // Box::new(YourPlugin::new()),
    ]
}
