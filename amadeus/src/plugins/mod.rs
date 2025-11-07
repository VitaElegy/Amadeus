pub mod code4rena;
pub mod example_plugin;

use crate::plugin::Plugin;
use code4rena::Code4renaPlugin;
use example_plugin::ExamplePlugin;

/// 获取所有可用的插件实例
/// 
/// 这个函数会自动创建所有插件的实例并返回
/// 新增插件时，只需要在这里添加即可
pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(Code4renaPlugin::new()),
        Box::new(ExamplePlugin::new()),
        // 在这里添加更多插件
        // Box::new(YourPlugin::new()),
    ]
}
