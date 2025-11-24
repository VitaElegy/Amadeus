// 这个文件展示了 Amadeus 插件系统的各种使用方式
// 运行: cargo run --example usage

use amadeus::app::App;
use amadeus::plugin::PluginRegistry;
use amadeus::plugins;

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // ============================================
    // 方式 1: 最简洁的方式 - 只需一行代码！
    // ============================================
    tracing::info!("=== 方式 1: 最简洁 ===");
    App::new().show_metadata(false).run()?;

    // ============================================
    // 方式 2: 手动控制注册表
    // ============================================
    tracing::info!("\n=== 方式 2: 手动控制 ===");
    {
        let mut registry = PluginRegistry::with_enabled_plugins(
            plugins::get_all_plugins()
        );
        
        registry.list_plugins();
        registry.run_lifecycle()?;
    }

    // ============================================
    // 方式 3: 按名称过滤插件
    // ============================================
    tracing::info!("\n=== 方式 3: 按名称过滤 ===");
    {
        let mut registry = PluginRegistry::new();
        registry.register_by_names(
            plugins::get_all_plugins(),
            &["code4rena"]  // 只加载这个插件
        );
        
        registry.list_plugins();
        registry.run_lifecycle()?;
    }

    // ============================================
    // 方式 4: 使用自定义过滤器
    // ============================================
    tracing::info!("\n=== 方式 4: 自定义过滤器 ===");
    {
        let mut registry = PluginRegistry::new();
        
        // 只加载 category 为 "security" 的插件
        registry.register_filtered(plugins::get_all_plugins(), |meta| {
            meta.properties.get("category") == Some(&"security".to_string())
        });
        
        registry.list_plugins();
        registry.run_lifecycle()?;
    }

    // ============================================
    // 方式 5: 链式调用生命周期
    // ============================================
    tracing::info!("\n=== 方式 5: 链式调用 ===");
    {
        let mut registry = PluginRegistry::with_enabled_plugins(
            plugins::get_all_plugins()
        );
        
        registry
            .init_all()?
            .start_all()?
            .run_all()?
            .stop_all()?;
    }

    Ok(())
}
