// 分发器订阅功能示例
// 运行: cargo run --example dispatcher_subscription

use amadeus::dispatcher::iceoryx2::Iceoryx2Dispatcher;
use amadeus::message::Message;
use amadeus::message_manager::MessageManager;
use amadeus::Dispatcher; // Add this import
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("=== 分发器订阅功能示例 ===\n");

    let mut msg_mgr = MessageManager::new();

    // 创建三个分发器，订阅不同的消息类型
    tracing::info!("1. 创建分发器并设置订阅：\n");

    // 分发器1：订阅所有消息类型（默认）
    let dispatcher1 = Iceoryx2Dispatcher::new("service1")
        .with_name("全量分发器");
    tracing::info!("  - {}: 订阅所有消息类型", dispatcher1.name());
    msg_mgr.register_dispatcher(dispatcher1).await;

    // 分发器2：只订阅 "notification" 类型
    let dispatcher2 = Iceoryx2Dispatcher::new("service2")
        .with_name("通知分发器")
        .subscribe_to(&["notification"]);
    tracing::info!("  - {}: 只订阅 'notification' 类型", dispatcher2.name());
    msg_mgr.register_dispatcher(dispatcher2).await;

    // 分发器3：订阅多个消息类型
    let dispatcher3 = Iceoryx2Dispatcher::new("service3")
        .with_name("多类型分发器")
        .subscribe_to(&["alert", "warning"]);
    tracing::info!("  - {}: 订阅 'alert' 和 'warning' 类型", dispatcher3.name());
    msg_mgr.register_dispatcher(dispatcher3).await;

    // 启动分发器
    msg_mgr.start_dispatchers().await?;
    msg_mgr.start_message_loop();

    // 模拟插件发送不同类型的消息
    tracing::info!("\n2. 模拟插件发送消息：\n");

    // 消息1：notification 类型
    let msg1 = Message::from_plugin(
        "notification",
        json!({"content": "这是一条通知"}),
        "test_plugin",
    );
    tracing::info!("  发送消息: 类型='notification'");
    msg_mgr.handle_plugin_message(msg1).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    tracing::info!("    → 应该被接收的分发器: 全量分发器, 通知分发器\n");

    // 消息2：alert 类型
    let msg2 = Message::from_plugin(
        "alert",
        json!({"content": "这是一条警报"}),
        "test_plugin",
    );
    tracing::info!("  发送消息: 类型='alert'");
    msg_mgr.handle_plugin_message(msg2).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    tracing::info!("    → 应该被接收的分发器: 全量分发器, 多类型分发器\n");

    // 消息3：warning 类型
    let msg3 = Message::from_plugin(
        "warning",
        json!({"content": "这是一条警告"}),
        "test_plugin",
    );
    tracing::info!("  发送消息: 类型='warning'");
    msg_mgr.handle_plugin_message(msg3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    tracing::info!("    → 应该被接收的分发器: 全量分发器, 多类型分发器\n");

    // 消息4：其他类型
    let msg4 = Message::from_plugin(
        "other",
        json!({"content": "这是其他类型的消息"}),
        "test_plugin",
    );
    tracing::info!("  发送消息: 类型='other'");
    msg_mgr.handle_plugin_message(msg4).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    tracing::info!("    → 应该被接收的分发器: 全量分发器（只有它订阅所有类型）\n");

    // 停止
    msg_mgr.stop_message_loop().await;
    msg_mgr.stop_dispatchers().await?;

    tracing::info!("3. 总结：");
    tracing::info!("  - 全量分发器：接收所有消息（订阅所有类型）");
    tracing::info!("  - 通知分发器：只接收 'notification' 类型");
    tracing::info!("  - 多类型分发器：只接收 'alert' 和 'warning' 类型");

    Ok(())
}

