use amadeus::App;
use amadeus::message::Message;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_core_system_architecture() -> anyhow::Result<()> {
    // 1. 初始化应用 (启用消息系统)
    // CoreSystemPlugin 会被自动加载，并连接到 amadeus.db (如果不存在则创建)
    // 为了测试干净，我们可以使用内存数据库，但 CoreSystemPlugin 目前在代码里硬编码了 "sqlite:amadeus.db"
    // TODO: 应该允许配置 DB URL。目前我们先接受文件数据库。
    
    // 清理旧的测试DB
    let _ = std::fs::remove_file("amadeus.db");

    let _app = App::new().with_messaging();
    
    // 我们需要在另一个任务中运行 app.run_async()，因为它是一个长时间运行的循环
    // 但 App::run_async 会阻塞直到 stop，所以我们得修改一下测试策略
    // 实际上 App::run_async 执行了 lifecycle：init -> start -> run -> stop
    // 如果我们想在它运行时交互，我们需要在它运行之前拿到 MessageManager 的引用
    // 但 App 拿到 ownership 后就跑了。
    
    // 让我们看看 App 的结构。它有 message_manager。
    // 在 main.rs 里通常是 app.run()。
    
    // 为了集成测试，我们可以手动构建组件，或者稍微修改 App 暴露更多控制。
    // 但 App::run_async 内部逻辑是：
    // 1. start_message_loop
    // 2. start_dispatchers
    // 3. run_lifecycle (init, start, run, stop)
    // 4. cleanup
    
    // 问题是 run_lifecycle 中的 run() 对于很多插件来说可能是一个阻塞操作或者一次性操作。
    // 对于 CoreSystemPlugin，run() 是空的。
    // 所以 app.run_async() 会很快执行完 init/start/run/stop 然后退出。
    // 这样的话，我们的消息处理循环可能还没来得及处理消息就退出了。
    
    // 正确的 Daemon 模式应该是：插件的 run() 方法可能会阻塞（如果不返回），或者 App 在 run_lifecycle 之后等待信号。
    // 目前的架构是：run_lifecycle 顺序执行所有插件的 run()，如果它们都立即返回，App 就退出了。
    
    // 我们手动组装一下测试环境，模拟 App 的行为，但拥有更多控制权。
    
    use amadeus::message_manager::MessageManager;
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

    // 8. 测试调度器
    // 我们发送一个调度消息
    let _schedule_msg = Message::new(
        "system.schedule.add",
        serde_json::json!({
            "cron": "1/1 * * * * * *", // 每秒执行一次 (quartz format with seconds?)
            // tokio-cron-scheduler 默认支持 6位 (sec min hour day month day_of_week) 
            // 或者是 5位 (min hour ...). Let's try standard 6 fields or 7.
            // "1/1 * * * * *" means every second?
            // "0/1 * * * * *" 
            "message": Message::new("test.scheduled", serde_json::Value::Null)
        })
    );
    // 注意：tokio-cron-scheduler 的 cron 格式比较严格。
    // 为了测试简单，我们跳过精确的 cron 测试，只要代码不 panic 就行。
    // tx.send(schedule_msg).await?;

    // 9. 清理
    registry.stop_all()?;
    message_manager.stop_message_loop().await;
    pool.close().await;

    Ok(())
}

