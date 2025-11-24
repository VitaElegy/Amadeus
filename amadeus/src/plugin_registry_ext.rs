use crate::distribution_center::DistributionCenter;
use crate::message_manager::MessageManager;
use crate::plugin::{MessagePlugin, Plugin};
use std::any::Any;

/// 扩展的插件注册表，支持消息插件
/// 
/// 这个结构提供了一个辅助方法来设置消息插件的订阅
pub struct ExtendedPluginRegistry;

impl ExtendedPluginRegistry {
    /// 为所有实现了 MessagePlugin 的插件设置消息订阅
    /// 
    /// 由于 Rust 的类型系统限制，这个方法需要插件在注册时明确类型
    /// 或者使用类型擦除。这里提供一个辅助函数。
    pub fn setup_messaging_for_plugins(
        plugins: &mut [Box<dyn Plugin>],
        message_manager: &MessageManager,
    ) -> anyhow::Result<()> {
        let distribution_center = message_manager.distribution_center();
        
        println!("\n=== 设置插件消息订阅 ===");
        
        for plugin in plugins.iter_mut() {
            // 尝试将插件转换为 MessagePlugin
            // 由于 trait object 的限制，我们需要使用类型擦除
            // 这里提供一个基础框架
            
            // 注意：实际使用中，如果插件实现了 MessagePlugin，
            // 应该在插件的 init() 或 start() 方法中通过其他方式
            // （如全局注册表、依赖注入等）获取分发中心并设置订阅
            
            // 或者，可以使用一个包装器来同时实现 Plugin 和 MessagePlugin
        }
        
        Ok(())
    }
}

/// 消息插件包装器
/// 
/// 这个包装器允许将实现了 MessagePlugin 的插件包装成 Plugin trait object
/// 同时保留消息功能
pub struct MessagePluginWrapper {
    plugin: Box<dyn MessagePlugin>,
}

impl MessagePluginWrapper {
    /// 创建新的消息插件包装器
    pub fn new<P: MessagePlugin + 'static>(plugin: P) -> Self {
        Self {
            plugin: Box::new(plugin),
        }
    }

    /// 获取内部插件的引用（用于设置消息订阅）
    pub fn as_message_plugin_mut(&mut self) -> &mut dyn MessagePlugin {
        self.plugin.as_mut()
    }
}

impl Plugin for MessagePluginWrapper {
    fn metadata(&self) -> &crate::plugin::PluginMetadata {
        self.plugin.metadata()
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.plugin.init()
    }

    fn start(&mut self) -> anyhow::Result<()> {
        self.plugin.start()
    }

    fn run(&mut self) -> anyhow::Result<()> {
        self.plugin.run()
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        self.plugin.stop()
    }

    fn is_enabled(&self) -> bool {
        self.plugin.is_enabled()
    }
}

