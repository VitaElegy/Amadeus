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
use crate::plugin::{Plugin, PluginMetadata, PluginType};
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
    // 必须实现：返回插件 ID（通常是名称）
    fn id(&self) -> &str {
        &self.metadata.name
    }

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

在 `src/plugins/mod.rs` 的 `get_all_plugins()` 函数中添加：

```rust
pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        // ... 其他插件
        Box::new(my_plugin::MyPlugin::new()),  // 添加这一行
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
    // 唯一标识符
    fn id(&self) -> &str;
    
    // 插件类型：Privileged (特权) 或 Normal (普通)
    fn plugin_type(&self) -> PluginType { PluginType::Normal }

    fn metadata(&self) -> &PluginMetadata;
    
    // 初始化阶段
    fn init(&mut self) -> Result<()> { Ok(()) }
    
    // 启动阶段
    fn start(&mut self) -> Result<()> { Ok(()) }
    
    // 停止阶段
    fn stop(&mut self) -> Result<()> { Ok(()) }
    
    // 消息订阅设置
    fn setup_messaging(
        &mut self,
        _dc: &DistributionCenter,
        _tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        Box::pin(async { Ok(None) })
    }
}
```

#### 2. PluginType (插件类型)

*   **PluginType::Normal**: 默认类型。用于普通业务插件。
*   **PluginType::Privileged**: 特权插件。优先加载，通常用于核心基础设施（如 IPC 分发器）。

---

## 消息系统

Amadeus 的消息系统现在支持 **Public (广播)** 和 **Direct (定向)** 两种模式。

### 接收消息

在 `setup_messaging` 中配置订阅：

```rust
fn setup_messaging(
    &mut self,
    distribution_center: &DistributionCenter,
    message_tx: mpsc::Sender<Message>,
) -> Pin<Box<dyn Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
    let plugin_name = self.metadata.name.clone();
    let dc = Arc::new(distribution_center.clone());

    Box::pin(async move {
        let ctx = Arc::new(MessageContext::new(dc, plugin_name, message_tx));

        // 1. 订阅广播消息 (Public)
        let mut public_rx = ctx.subscribe("some.public.topic").await;
        
        // 2. 启用定向消息 (Direct)
        let mut direct_rx = ctx.enable_direct_messaging().await;

        // 处理消息循环
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(msg) = public_rx.recv() => {
                        println!("收到广播消息: {:?}", msg);
                    }
                    Some(msg) = direct_rx.recv() => {
                        println!("收到私密消息: {:?}", msg);
                    }
                }
            }
        });

        Ok(Some(ctx))
    })
}
```

### 发送消息

#### 1. 发送广播消息

```rust
let msg = Message::new("some.public.topic", json!({"data": "hello"}));
ctx.send(msg).await?;
```
所有订阅了 `some.public.topic` 的插件都会收到。

#### 2. 发送定向消息

```rust
let msg = Message::new_direct(
    "target_plugin_id", // 目标插件 ID
    "private.command",
    json!({"secret": "123"})
);
ctx.send(msg).await?;
```
只有 ID 为 `target_plugin_id` 的插件会收到此消息，**不会**被广播。

---

## IPC 与外部通信

现在 IPC 通信（原分发器模块）已封装为 **`Iceoryx2DispatcherPlugin`**。它作为特权插件运行，负责将内部广播消息转发到外部系统，并将外部消息转发到内部总线。

*   **普通插件** 不需要关心 IPC。只需发送广播消息，分发器插件会自动转发（如果配置了）。
*   **定向消息** 默认 **不会** 转发到外部，仅限内部插件间通信。

---
