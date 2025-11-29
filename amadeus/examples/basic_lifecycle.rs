// 基础生命周期示例
// 演示插件的初始化、启动和停止流程

use amadeus::App;
use amadeus::plugin::{Plugin, PluginMetadata};
use anyhow::Result;
use tracing::{info, Level};

// 定义一个简单的自定义插件
pub struct LifecycleDemoPlugin {
    metadata: PluginMetadata,
}

impl LifecycleDemoPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "lifecycle_demo",
                "Demonstrates plugin lifecycle",
                "0.1.0",
            ),
        }
    }
}

impl Plugin for LifecycleDemoPlugin {
    fn id(&self) -> &str {
        "lifecycle_demo"
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        info!("Step 1: Init - Loaded configuration");
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        info!("Step 2: Start - Services are running");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Step 3: Stop - Cleanup complete");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("=== Basic Lifecycle Demo ===");

    // 使用 with_plugins 只加载我们要演示的插件，避免其他杂音
    let app = App::with_plugins(vec![
        Box::new(LifecycleDemoPlugin::new())
    ]);

    // 运行应用
    // App::run_async 会执行 init -> start，然后挂起直到收到 Ctrl+C
    // 为了演示目的，我们不会让它永远阻塞，而是手动控制（但在真实应用中通常是阻塞的）
    
    // 由于 App::run_async 设计为阻塞直到停止信号，我们在这里使用一个稍微不同的方式
    // 或者我们接受它需要手动停止。
    // 为了自动化演示，我们通常不使用 App::run_async，而是像测试那样手动控制 Registry。
    // 但作为示例，我们展示标准用法。
    
    // 提示用户如何退出
    info!("App running... Press Ctrl+C to stop and trigger shutdown sequence.");
    app.run_async().await?;

    Ok(())
}

