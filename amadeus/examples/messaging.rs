// 消息系统使用示例（使用 tokio）
// 运行: cargo run --example messaging

use amadeus::App;
use amadeus::plugins::iceoryx2_dispatcher::Iceoryx2DispatcherPlugin;
use amadeus::plugins::message_example::MessageExamplePlugin;
use amadeus::plugin::Plugin;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("=== Amadeus 消息系统示例（使用 tokio）===");

    // 创建应用并启用消息系统
    // 我们手动添加插件以展示配置过程
    let app = App::with_plugins(vec![
        Box::new(Iceoryx2DispatcherPlugin::new("amadeus_demo")),
        Box::new(MessageExamplePlugin::new()),
    ])
    .with_messaging()
    .show_metadata(true);

    // 运行应用
    app.run_async().await?;

    Ok(())
}
