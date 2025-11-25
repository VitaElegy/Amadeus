# Amadeus 开发者指南

本文档是 Amadeus 插件系统的完整开发者指南，包含快速入门、插件开发、消息系统使用和高级功能。

## 目录

- [快速开始](#快速开始)
- [插件开发基础](#插件开发基础)
- [消息系统](#消息系统)
- [高级功能](#高级功能)

---

## 快速开始

### 5 分钟创建一个插件

#### 步骤 1: 创建插件文件

在 `src/plugins/` 下创建 `my_plugin.rs`：

```rust
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "my_plugin",
            "我的第一个插件",
            "0.1.0",
        )
        .enabled_by_default(true)
        .author("你的名字");

        Self { metadata }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        tracing::info!("🚀 [MyPlugin] 插件初始化成功！");
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        tracing::info!("✨ [MyPlugin] 插件启动...");
        // 如果有长运行任务，请在这里 spawn
        tokio::spawn(async move {
            // 长时间运行的任务逻辑
        });
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        tracing::info!("👋 [MyPlugin] 插件已停止");
        Ok(())
    }
}
```

#### 步骤 2: 导出插件模块

编辑 `src/plugins/mod.rs`，添加：

```rust
pub mod my_plugin;
```

#### 步骤 3: 注册插件

编辑 `src/main.rs` 或使用 `App` 构建器：

```rust
use amadeus::App;

fn main() -> anyhow::Result<()> {
    App::new().run()
}
```

在 `src/plugins/mod.rs` 的 `get_all_plugins()` 函数中添加：

```rust
pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(Code4renaPlugin::new()),
        Box::new(ExamplePlugin::new()),
        Box::new(MyPlugin::new()),  // 添加这一行
    ]
}
```

#### 步骤 4: 运行！

```bash
cargo run
```

---

## 插件开发基础

### 核心概念

#### 1. Plugin Trait

所有插件都必须实现 `Plugin` trait：

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    
    // 初始化阶段：加载配置，准备环境
    fn init(&mut self) -> Result<()> { Ok(()) }
    
    // 启动阶段：启动服务，生成后台任务
    // 注意：不要在这里阻塞！长任务请使用 tokio::spawn
    fn start(&mut self) -> Result<()> { Ok(()) }
    
    // 停止阶段：清理资源
    fn stop(&mut self) -> Result<()> { Ok(()) }
}
```

#### 2. 生命周期

插件有三个主要生命周期阶段（由 `App` 自动管理）：

1.  **init()** - 初始化（加载配置、准备资源）
2.  **start()** - 启动（建立连接、启动服务、Spawn 后台任务）
3.  **stop()** - 停止（清理资源、保存状态）

> **注意**：旧版 API 中的 `run()` 方法已被废弃，请在 `start()` 中启动异步任务。

#### 3. 元数据

每个插件都有丰富的元数据：

```rust
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled_by_default: bool,
    pub author: Option<String>,
    pub properties: HashMap<String, String>,
}
```

### 创建单文件插件

单文件插件适合简单功能，所有代码在一个文件中：

```rust
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct SimplePlugin {
    metadata: PluginMetadata,
}

impl SimplePlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "simple_plugin",
            "简单插件",
            "0.1.0",
        )
        .enabled_by_default(true);

        Self { metadata }
    }
}

impl Plugin for SimplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn run(&mut self) -> Result<()> {
        tracing::info!("执行简单任务");
        Ok(())
    }
}
```

> ⚠️ **注意**：虽然上面的例子使用了 `run`（如果是旧代码），但推荐在新版中使用 `start`。如果您的 trait 定义中只有 `init/start/stop`，请将逻辑移至 `start`。

### 创建多文件插件

复杂插件可以使用文件夹组织：

```
src/plugins/complex_plugin/
├── mod.rs      # 主模块
├── config.rs   # 配置管理
└── handler.rs  # 业务逻辑
```

**mod.rs** - 主模块：

```rust
mod config;
mod handler;

use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct ComplexPlugin {
    metadata: PluginMetadata,
    config: config::Config,
    handler: handler::Handler,
}

impl ComplexPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "complex_plugin",
            "复杂插件",
            "0.1.0",
        );
        
        Self {
            metadata,
            config: config::Config::default(),
            handler: handler::Handler::new(),
        }
    }
}

impl Plugin for ComplexPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        self.config.load()?;
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        self.handler.process(&self.config)?;
        Ok(())
    }
}
```

### 插件注册与执行

```rust
use amadeus::{App, PluginRegistry};

fn main() -> anyhow::Result<()> {
    // 方式 1: 使用 App 构建器（推荐）
    // App 会自动处理生命周期并监听 Ctrl+C 信号
    App::new()
        .show_metadata(true)
        .run()?;

    Ok(())
}
```

---

## 消息系统

Amadeus 的消息分发系统提供了完整的消息路由和分发功能，支持插件之间的消息通信以及与外部的消息交互。

### 架构概述

消息系统由四个核心组件组成：

1. **分发器（Dispatcher）** - 负责与外界交互（如前端、QQ bot等）
2. **分发中心（Distribution Center）** - 消息路由中心，管理消息订阅和分发
3. **消息管理器（Message Manager）** - 协调消息流
4. **插件（Plugin）** - 可以订阅和发送消息

### 消息流程

#### 外部消息 → 插件

```
外部系统 → 分发器 → 分发中心 → 订阅的插件
```

#### 插件消息 → 外部

```
插件 → 分发中心 → 分发器 → 外部系统
```

### 创建支持消息的应用

```rust
use amadeus::App;
use amadeus::dispatcher::iceoryx2::Iceoryx2Dispatcher;

fn main() -> anyhow::Result<()> {
    let mut app = App::new()
        .with_messaging()  // 启用消息系统
        .show_metadata(false);

    // 注册分发器
    if let Some(msg_mgr) = app.message_manager_mut() {
        let dispatcher = Iceoryx2Dispatcher::new("amadeus_service")
            .with_name("Iceoryx2分发器");
        msg_mgr.register_dispatcher(dispatcher);
    }

    app.run()?;
    Ok(())
}
```

### 测试消息系统

使用提供的 Python 测试脚本验证 Iceoryx2 分发器功能：

```bash
# 1. 启动 Rust 消息应用
cargo run --example messaging

# 2. 在新终端运行 Python 测试
cd examples/iceoryx2
python3 test_integration.py
```

#### Python 测试脚本

项目提供了完整的 Python 测试套件：

- **`publisher.py`** - Python 发布者，向 Rust 分发器发送消息
- **`subscriber.py`** - Python 订阅者，接收来自 Rust 分发器的消息
- **`test_integration.py`** - 集成测试，同时运行发布者和订阅者

**测试验证内容：**
- ✅ 跨语言零拷贝通信（Rust ↔ Python）
- ✅ 消息格式兼容性
- ✅ 服务发现和连接
- ✅ 实时消息传递

### 创建支持消息的插件

```rust
use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use crate::message_context::MessageContext;
use crate::plugin::{MessagePlugin, Plugin, PluginMetadata};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct MyMessagePlugin {
    metadata: PluginMetadata,
    message_context: Option<Arc<MessageContext>>,
}

impl Plugin for MyMessagePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    // ... 实现其他方法
}

impl MessagePlugin for MyMessagePlugin {
    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: mpsc::Sender<Message>,
    ) -> impl std::future::Future<Output = Result<Arc<MessageContext>>> + Send {
        let plugin_name = self.metadata.name.clone();
        let distribution_center = Arc::new(distribution_center.clone());
        
        async move {
            // 创建消息上下文
            let ctx = Arc::new(MessageContext::new(
                distribution_center,
                &plugin_name,
                message_tx,
            ));
            
            // 订阅消息类型并启动接收任务
            let mut command_rx = ctx.subscribe("command").await;
            let ctx_clone = Arc::clone(&ctx);
            
            tokio::spawn(async move {
                while let Ok(message) = command_rx.recv().await {
                    println!("收到命令: {}", message.payload);
                }
            });
            
            Ok(ctx)
        }
    }
}
```

### 发送消息

```rust
// 在插件中发送消息（异步）
if let Some(ctx) = &self.message_context {
    let message = Message::from_plugin(
        "notification",
        json!({
            "content": "处理完成"
        }),
        &self.metadata.name,
    );
    ctx.send(message).await?;
}
```

### 实现自定义分发器

要实现自定义分发器，只需实现 `Dispatcher` trait：

```rust
use amadeus::dispatcher::Dispatcher;
use amadeus::message::Message;
use anyhow::Result;

pub struct MyDispatcher {
    name: String,
}

impl Dispatcher for MyDispatcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        // 启动逻辑
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        // 停止逻辑
        Ok(())
    }

    fn send_message(&self, message: &Message) -> Result<()> {
        // 发送消息到外部
        Ok(())
    }

    fn is_running(&self) -> bool {
        true
    }
}
```

---

## 高级功能

### 配置文件管理

#### 创建配置文件

创建 `plugins_config.json`：

```json
[
  {
    "name": "my_plugin",
    "description": "我的插件",
    "version": "1.0.0",
    "enabled_by_default": true,
    "author": "Your Name",
    "properties": {
      "api_key": "secret-key-123",
      "timeout": "30",
      "max_retries": "3"
    }
  }
]
```

#### 在插件中读取配置

```rust
impl Plugin for ConfigurablePlugin {
    fn init(&mut self) -> Result<()> {
        let props = &self.metadata.properties;
        
        if let Some(key) = props.get("api_key") {
            self.api_key = key.clone();
        }
        
        if let Some(timeout) = props.get("timeout") {
            self.timeout = timeout.parse().unwrap_or(30);
        }
        
        Ok(())
    }
}
```

### 插件间通信

使用共享状态实现插件间通信：

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub type SharedState = Arc<Mutex<HashMap<String, String>>>;

pub struct PluginA {
    metadata: PluginMetadata,
    shared_state: SharedState,
}

impl Plugin for PluginA {
    fn start(&mut self) -> Result<()> {
        let mut state = self.shared_state.lock().unwrap();
        state.insert("plugin_a_data".to_string(), "Hello from A".to_string());
        Ok(())
    }
}

pub struct PluginB {
    metadata: PluginMetadata,
    shared_state: SharedState,
}

impl Plugin for PluginB {
    fn start(&mut self) -> Result<()> {
        let state = self.shared_state.lock().unwrap();
        if let Some(data) = state.get("plugin_a_data") {
            tracing::info!("读取到 Plugin A 的数据: {}", data);
        }
        Ok(())
    }
}

// 使用方式
fn main() -> anyhow::Result<()> {
    let shared_state = Arc::new(Mutex::new(HashMap::new()));
    
    let mut registry = PluginRegistry::new();
    // 注意：手动模式下需要手动调用生命周期，或使用 App::with_plugins
    // 推荐使用 App
    Ok(())
}
```

### 异步插件支持

使用 `async-trait` 创建异步插件：

```rust
use async_trait::async_trait;

#[async_trait]
pub trait AsyncPlugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AsyncPlugin for AsyncHttpPlugin {
    async fn start(&mut self) -> Result<()> {
        let client = self.client.clone();
        // 启动异步任务
        tokio::spawn(async move {
            if let Ok(response) = client.get("https://api.example.com/data").send().await {
                 if let Ok(body) = response.text().await {
                     tracing::info!("响应: {}", body);
                 }
            }
        });
        Ok(())
    }
}
```

### 插件依赖管理

在元数据中定义依赖：

```rust
let metadata = PluginMetadata::new("my_plugin", "描述", "0.1.0")
    .with_dependencies(vec!["plugin_a", "plugin_b"]);
```

### 错误处理最佳实践

```rust
use anyhow::{Result, Context};

impl Plugin for MyPlugin {
    fn init(&mut self) -> Result<()> {
        let config = self.load_config()
            .context("加载配置失败")?;
        
        self.validate_config(&config)
            .context("配置验证失败")?;
        
        Ok(())
    }
}
```

### 测试插件

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_lifecycle() -> Result<()> {
        let mut plugin = MyPlugin::new();

        plugin.init()?;
        plugin.start()?; // 替代 run()
        plugin.stop()?;

        Ok(())
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = MyPlugin::new();
        let meta = plugin.metadata();

        assert_eq!(meta.name, "my_plugin");
        assert_eq!(meta.version, "0.1.0");
    }
}
```

### 跨语言集成测试

使用 Python 测试脚本验证与 Iceoryx2 分发器的集成：

```bash
# 运行集成测试
cd examples/iceoryx2
python3 test_integration.py

# 或分别测试发布者和订阅者
python3 publisher.py    # 在一个终端
python3 subscriber.py   # 在另一个终端
```

**测试最佳实践：**
- ✅ **单元测试**：测试插件的各个生命周期阶段
- ✅ **集成测试**：测试插件间的消息传递
- ✅ **跨语言测试**：验证与外部系统的通信
- ✅ **性能测试**：监控消息传递延迟和吞吐量

---

## 最佳实践

1. **单一职责** - 每个插件只做一件事
2. **优雅错误处理** - 使用 `Result` 和 `?` 操作符
3. **清理资源** - 在 `stop()` 中释放所有资源
4. **日志记录** - 在关键步骤添加日志
5. **配置验证** - 在 `init()` 中验证配置
6. **状态检查** - 在操作前检查插件状态
7. **文档注释** - 为公开 API 添加文档
8. **测试覆盖** - 为每个插件编写单元测试

---

## 参考示例

### Rust 示例

- `src/plugins/code4rena.rs` - 单文件插件示例
- `src/plugins/example_plugin/` - 多文件插件示例
- `examples/messaging.rs` - 消息系统示例
- `examples/usage.rs` - 基础使用示例

### Python 测试示例

- `examples/iceoryx2/amadeus_message_data.py` - Python 消息数据结构
- `examples/iceoryx2/publisher.py` - Python 发布者示例
- `examples/iceoryx2/subscriber.py` - Python 订阅者示例
- `examples/iceoryx2/test_integration.py` - 跨语言集成测试

### 配置文件示例

- `doc/plugins_config.example.json` - 插件配置 JSON 示例

---

**开始创建您的插件吧！** 🚀

---

## 内置插件参考 (Built-in Plugins Reference)

### Core System Plugin

CoreSystem Plugin 是 Amadeus 的核心插件，提供持久化存储（备忘录）和任务调度功能。

#### 1. 消息协议

插件响应以 `system.memo.` 和 `system.schedule.` 开头的消息。

##### 1.1 备忘录管理 (Memo Management)

**创建备忘录/TODO (Create)**
*   **Topic**: `system.memo.create`
*   **Payload**:
    ```json
    {
      "content": "面试准备",
      "cron": "0 9 * * * *",      // 可选：主提醒 Cron 表达式
      "remind_at": 1700000000,    // 可选：Unix 时间戳
      "tags": ["work", "urgent", "stage_goal"], // 可选：标签
      "todo_date": 1700000000,    // 可选：目标日期
      "priority": 1               // 可选：优先级 (0-Low, 1-Normal, 2-High)
    }
    ```
*   **Response**: `system.memo.created`
    ```json
    {
      "id": 1,
      "content": "面试准备"
    }
    ```
*   **自动化行为**:
    *   如果提供了 `cron`，会自动注册主提醒任务。
    *   如果包含特定标签（如 `stage_goal`），会自动注册额外的标签提醒任务（例如每天 10:00 提醒）。

**完成备忘录/TODO (Complete)**
*   **Topic**: `system.memo.complete`
*   **Payload**:
    ```json
    {
      "id": 1
    }
    ```
*   **Response**: `system.memo.complete.success`
    ```json
    {
      "id": 1,
      "status": "completed"
    }
    ```
*   **Side Effect**: 
    *   备忘录状态更新为 `completed`。
    *   **所有**关联的 Cron 任务（包括主提醒和标签提醒）会被自动移除。

**删除备忘录/TODO (Delete)**
*   **Topic**: `system.memo.delete`
*   **Payload**:
    ```json
    {
      "id": 1
    }
    ```
*   **Response**: `system.memo.delete.success`
    ```json
    {
      "id": 1,
      "status": "deleted"
    }
    ```
*   **Side Effect**: 同上，所有关联任务被移除。

**列出活跃项 (List)**
*   **Topic**: `system.memo.list`
*   **Payload**: `{}` (空对象)
*   **Response**: `system.memo.list.reply`
    ```json
    {
      "memos": [
        {
          "id": 1,
          "content": "面试准备",
          "cron": "...",
          "tags": ["work", "stage_goal"],
          "priority": 1,
          "todo_date": 1700000000
        }
      ]
    }
    ```

##### 1.2 调度器 (Scheduler)

**添加通用定时任务**
*   **Topic**: `system.schedule.add`
*   **Payload**:
    ```json
    {
      "cron": "1/5 * * * * *",
      "message": {
        "message_type": "my.custom.topic",
        "payload": { "foo": "bar" }
      }
    }
    ```
*   **Response**: `system.schedule.added`

#### 2. 提醒事件

当 Cron 规则触发时，插件会广播提醒消息：

*   **Topic**: `system.memo.remind`
*   **Payload**:
    ```json
    {
      "id": 1,
      "content": "面试准备",
      "type": "primary" // 或 "tag_reminder"
    }
    ```
    
    如果是标签触发的提醒，会包含额外字段：
    ```json
    {
      "id": 1,
      "content": "面试准备",
      "type": "tag_reminder",
      "tag": "stage_goal"
    }
    ```

#### 3. 持久化与恢复

*   **存储**: 使用 SQLite 数据库 (`amadeus.db`)。
    *   表 `memos` 增加了 `tags` (JSON数组) 和 `metadata` (JSON对象) 列以支持扩展属性。
*   **自动恢复**: 
    *   插件启动时，扫描数据库中所有活跃项。
    *   恢复主 Cron 任务 (`cron` 字段)。
    *   根据 `tags` 重新评估并注册标签相关的自动提醒（例如 `stage_goal` 标签的每日提醒）。

