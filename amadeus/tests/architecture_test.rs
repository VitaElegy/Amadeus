use amadeus::App;
use amadeus::core::messaging::message::Message;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_core_system_architecture() -> anyhow::Result<()> {
    // 1. 初始化应用 (启用消息系统)
    // CoreSystemPlugin 会被自动加载，并连接到 amadeus.db (如果不存在则创建)
    
    // 清理旧的测试DB
    let _ = std::fs::remove_file("amadeus.db");

    // 让我们手动组装一下测试环境，模拟 App 的行为，但拥有更多控制权。
    
    use amadeus::core::messaging::message_manager::MessageManager;
    use amadeus::plugin::PluginRegistry;
    
    // 1. 创建 MessageManager
    let mut message_manager = MessageManager::new();
    
    // 2. 创建 Registry 并加载插件
    let mut registry = PluginRegistry::with_enabled_plugins(
        amadeus::plugins::get_all_plugins()
    );
    
    // 3. Setup Messaging (这一步会初始化 CoreSystemPlugin 的 Storage 和 Scheduler)
    registry.setup_messaging(&message_manager).await?;
    
    // 4. 启动消息循环
    message_manager.start_message_loop();
    
    // 5. 模拟 App 启动流程
    registry.init_all()?;
    registry.start_all()?;
    
    // 6. 发送测试消息：创建 Memo
    let tx = message_manager.message_tx();
    let memo_content = "Test Memo content";
    let msg = Message::new(
        "system.memo.create",
        serde_json::json!({
            "content": memo_content
        })
    );
    
    tx.send(msg).await?;
    
    // 给一点时间让异步消息处理完成
    sleep(Duration::from_millis(500)).await;
    
    // 7. 验证：我们需要查询数据库看是否写入成功
    // 由于我们无法直接访问 CoreSystemPlugin 内部的 Storage，
    // 我们通过直接连接数据库来验证。
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:amadeus.db")
        .await?;
        
    let row: (String,) = sqlx::query_as("SELECT content FROM memos WHERE content = ?")
        .bind(memo_content)
        .fetch_one(&pool)
        .await?;
        
    assert_eq!(row.0, memo_content);
    println!("Verified: Memo found in database!");

    // 8. 清理
    registry.stop_all()?;
    message_manager.stop_message_loop().await;
    pool.close().await;

    Ok(())
}

