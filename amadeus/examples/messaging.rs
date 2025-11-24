// 消息系统使用示例（使用 tokio）
// 运行: cargo run --example messaging

use amadeus::dispatcher::iceoryx2::Iceoryx2Dispatcher;
use amadeus::message::Message;
use amadeus::message_manager::MessageManager;
use amadeus::App;
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Amadeus 消息系统示例（使用 tokio）===\n");

    // 创建应用并启用消息系统
    let mut app = App::new()
        .with_messaging()
        .show_metadata(false);

    // 获取消息管理器并注册分发器
    if let Some(msg_mgr) = app.message_manager_mut() {
        // 注册 Iceoryx2 分发器
        let dispatcher = Iceoryx2Dispatcher::new("amadeus_service")
            .with_name("Iceoryx2分发器");
        msg_mgr.register_dispatcher(dispatcher);
    }

    // 运行应用
    app.run()?;

    // 演示消息发送和接收
    println!("\n=== 消息系统演示 ===");
    demonstrate_messaging().await?;

    Ok(())
}

async fn demonstrate_messaging() -> Result<()> {
    let mut msg_mgr = MessageManager::new();

    // 注册一个分发器
    let dispatcher = Iceoryx2Dispatcher::new("demo_service");
    msg_mgr.register_dispatcher(dispatcher);

    // 启动分发器
    msg_mgr.start_dispatchers()?;

    // 启动消息处理循环
    msg_mgr.start_message_loop();

    // 模拟外部消息到达
    println!("\n1. 模拟外部消息到达...");
    let external_message = Message::from_external(
        "command",
        json!({
            "action": "process",
            "data": "test data"
        }),
        "external_system",
    );

    msg_mgr.handle_dispatcher_message(external_message).await?;

    // 等待一下让消息处理
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 模拟插件发送消息
    println!("\n2. 模拟插件发送消息...");
    let plugin_message = Message::from_plugin(
        "notification",
        json!({
            "type": "status",
            "content": "处理完成"
        }),
        "example_plugin",
    );

    msg_mgr.handle_plugin_message(plugin_message).await?;

    // 等待一下让消息处理
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 显示订阅统计
    println!("\n3. 订阅统计:");
    let stats = msg_mgr.distribution_center().get_subscription_stats().await;
    if stats.is_empty() {
        println!("   (暂无订阅)");
    } else {
        for (msg_type, count) in stats {
            println!("   {}: {} 个订阅者", msg_type, count);
        }
    }

    // 停止消息循环和分发器
    msg_mgr.stop_message_loop().await;
    msg_mgr.stop_dispatchers()?;

    Ok(())
}
