use amadeus::core::messaging::message::Message;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_user_system_flow() -> anyhow::Result<()> {
    // 0. 初始化日志
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // 1. 初始化环境
    let _ = std::fs::remove_file("amadeus_user_test.db");
    
    use amadeus::core::messaging::message_manager::MessageManager;
    use amadeus::plugin::PluginRegistry;
    
    // 配置使用 amadeus_user_test.db
    // 注意：这里我们通过设置环境变量或者修改代码来指定DB稍微有点麻烦，
    // 因为 CoreSystemPlugin 目前硬编码了 db_url 如果是在 PluginRegistry::with_enabled_plugins 里创建的话。
    // 但我们可以手动实例化 CoreSystemPlugin。
    
    let mut message_manager = MessageManager::new();
    let mut registry = PluginRegistry::new();
    
    // 手动添加 CoreSystemPlugin，指定测试DB
    let core_plugin = amadeus::plugins::core_system::CoreSystemPlugin::new("sqlite:amadeus_user_test.db");
    registry.register(core_plugin);
    
    // 还需要添加一个 MessageExample 插件作为“外部适配器”的角色，或者我们就用 registry.setup_messaging 返回的 client 吗？
    // PluginRegistry 没有暴露直接发送消息的接口给外部测试代码，除了通过 MessageManager。
    // 但是我们需要接收回复。
    // 我们可以创建一个 MockPlugin 来接收回复。
    
    // 使用 amadeus::plugins::message_example::MessageExamplePlugin;
    // registry.register(Box::new(MessageExamplePlugin::default()));

    registry.setup_messaging(&message_manager).await?;
    message_manager.start_message_loop();
    registry.init_all()?;
    registry.start_all()?;
    
    // 2. 模拟外部适配器发送 resolve 请求
    // 我们需要一个途径来接收定向回复。
    // 在测试中，我们可以订阅所有消息，虽然我们无法直接作为 "recipient" 接收定向消息（除非我们冒充某个插件）。
    // 这是一个架构上的限制：定向消息只能发给注册过的插件通道。
    
    // 解决方案：我们创建一个临时的 "MockAdapter" 插件并在测试中注册它。
    // 或者，为了简化，我们修改 system.user.resolved 为广播消息（仅用于调试/测试），或者让 CoreSystem 直接返回。
    // 但为了严谨，我们应该遵循协议。
    
    // 让我们用 MessageManager 的 channel 发送消息，但是接收回复比较困难，因为定向消息不会流过广播通道。
    // 等等，DistributionCenter 的 subscribe_all 只能收到公共消息。
    // 定向消息走的是 register_direct_channel 注册的 mpsc。
    
    // 既然如此，测试策略改为：验证数据库副作用。
    
    let tx = message_manager.message_tx();
    
    // A. 请求解析/创建用户
    let platform_id = "discord";
    let platform_user_id = "123456789";
    let user_name = "Test User";
    
    let msg = Message::new(
        "system.user.resolve",
        serde_json::json!({
            "platform": platform_id,
            "platform_user_id": platform_user_id,
            "name": user_name
        })
    );
    // .with_recipient("CoreSystem"); // 不要设为定向，因为 CoreSystem 目前还没开启定向接收
    
    // 注意：Message::new 默认是广播。
    println!("Sending resolve message...");
    tx.send(msg).await?;
    println!("Message sent.");
    
    sleep(Duration::from_millis(2000)).await;
    
    // 3. 验证数据库
    println!("Checking database...");
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:amadeus_user_test.db")
        .await?;
        
    let row: (String, String) = sqlx::query_as("SELECT name, platform FROM users WHERE platform_user_id = ?")
        .bind(platform_user_id)
        .fetch_one(&pool)
        .await?;
        
    assert_eq!(row.0, user_name);
    assert_eq!(row.1, platform_id);
    println!("Verified: User created in database!");
    
    // 4. 清理
    registry.stop_all()?;
    message_manager.stop_message_loop().await;
    pool.close().await;
    let _ = std::fs::remove_file("amadeus_user_test.db");
    
    Ok(())
}

