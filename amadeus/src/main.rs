mod plugin;
mod plugins;

use plugin::PluginRegistry;
use plugins::code4rena::Code4renaPlugin;
use plugins::example_plugin::ExamplePlugin;

fn main() -> anyhow::Result<()> {
    println!("=== Amadeus 插件系统启动 ===\n");

    // 创建插件注册表
    let mut registry = PluginRegistry::new();

    // 注册插件
    registry.register(Code4renaPlugin::new());
    registry.register(ExamplePlugin::new());
    // 可以继续注册更多插件
    // registry.register(AnotherPlugin::new());

    // 列出所有已注册的插件
    registry.list_plugins();

    // 导出插件元数据到 JSON（演示用）
    match registry.export_metadata() {
        Ok(json) => {
            println!("\n=== 插件元数据 (JSON) ===");
            println!("{}", json);
        }
        Err(e) => eprintln!("导出元数据失败: {}", e),
    }

    // 执行插件生命周期
    registry.init_all()?;
    registry.start_all()?;
    registry.run_all()?;
    registry.stop_all()?;

    println!("\n=== Amadeus 插件系统已关闭 ===");
    Ok(())
}
