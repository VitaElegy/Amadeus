# Amadeus 架构文档

本文档详细说明 Amadeus 插件系统的架构设计、技术栈、通信机制和优化策略。

## 目录

- [系统架构](#系统架构)
- [通信机制](#通信机制)
- [技术栈](#技术栈)
- [代码优化](#代码优化)
- [文件结构](#文件结构)

---

## 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                          Main Application                        │
│                            (main.rs)                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           │ 创建并管理
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                       PluginRegistry                             │
│                      (plugin.rs)                                 │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  plugins: Vec<Box<dyn Plugin>>                         │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Methods:                                                        │
│  • register()      - 注册插件                                   │
│  • init_all()      - 初始化所有插件                             │
│  • start_all()     - 启动所有插件                               │
│  • stop_all()      - 停止所有插件                               │
│  • startup()       - 执行 init -> start                         │
│  • shutdown()      - 执行 stop                                  │
└──────────────────┬──────────────────┬────────────────┬──────────┘
                   │                  │                │
                   │                  │                │
        ┌──────────┘                  │                └─────────┐
        │                             │                          │
        ▼                             ▼                          ▼
┌───────────────┐          ┌───────────────────┐     ┌────────────────┐
│  Code4rena    │          │  Example Plugin   │     │  Custom Plugin │
│   Plugin      │          │   (Multi-file)    │     │    (Future)    │
│ (Single file) │          └───────────────────┘     └────────────────┘
└───────────────┘                    │
        │                            │
        │                   ┌────────┴────────┐
        │                   │                 │
        │                   ▼                 ▼
        │          ┌─────────────┐   ┌──────────────┐
        │          │  config.rs  │   │ handler.rs   │
        │          └─────────────┘   └──────────────┘
        │
        └──────────────────────┐
                               │
        ┌──────────────────────┴──────────────────────────┐
        │         实现 Plugin Trait                       │
        │                                                  │
        │  • metadata() -> &PluginMetadata                │
        │  • init()     -> Result<()>                     │
        │  • start()    -> Result<()>                     │
        │  • stop()     -> Result<()>                     │
        └──────────────────────────────────────────────────┘
```

### 生命周期流程

```
应用启动
   │
   ▼
创建 PluginRegistry
   │
   ▼
注册插件 (register)
   │
   ▼
──────────────────────
初始化阶段 (init_all)
──────────────────────
   │
   ├─► Plugin 1.init()
   ├─► Plugin 2.init()
   └─► Plugin 3.init()
   │
   ▼
──────────────────────
启动阶段 (start_all)
──────────────────────
   │
   ├─► Plugin 1.start() ──► spawn(Task 1)
   ├─► Plugin 2.start() ──► spawn(Task 2)
   └─► Plugin 3.start()
   │
   ▼
──────────────────────
运行阶段 (Running)
──────────────────────
   │
   ● 主线程挂起 (await Ctrl+C)
   ● 后台任务并行运行
   │
   ▼
──────────────────────
停止阶段 (shutdown)
(收到信号，按相反顺序)
──────────────────────
   │
   ├─► Plugin 3.stop()
   ├─► Plugin 2.stop()
   └─► Plugin 1.stop()
   │
   ▼
应用退出
```

### 消息系统架构

```
┌─────────────────────────────────────────────────────────┐
│                    外部系统/进程                          │
│              (前端、QQ bot、其他服务)                     │
└────────────────────┬────────────────────────────────────┘
                     │ Iceoryx2 (进程间通信)
                     │ 零拷贝 IPC
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  Dispatcher (分发器)                     │
│              Iceoryx2Dispatcher, HTTPDispatcher...       │
└────────────────────┬────────────────────────────────────┘
                     │ 消息格式转换
                     ▼
┌─────────────────────────────────────────────────────────┐
│              MessageManager (消息管理器)                 │
│              使用 tokio::sync::mpsc 队列                 │
└────────────────────┬────────────────────────────────────┘
                     │ Tokio Broadcast (进程内通信)
                     │ 内存通道，极低延迟
                     ▼
┌─────────────────────────────────────────────────────────┐
│          DistributionCenter (分发中心)                   │
│        使用 tokio::sync::broadcast 广播                  │
└───────┬───────────────┬───────────────┬────────────────┘
        │               │               │
        ▼               ▼               ▼
    ┌───────┐      ┌───────┐      ┌───────┐
    │插件 A │      │插件 B │      │插件 C │
    └───────┘      └───────┘      └───────┘
```

---

## 通信机制

### Tokio vs Iceoryx2

**核心区别：它们不重复，是互补的，服务于不同的通信场景。**

#### Tokio Broadcast - 进程内通信（In-Process）

**使用场景：**
- ✅ **同一进程内**的组件通信
- ✅ 插件之间的消息传递
- ✅ 分发中心到插件的消息路由

**技术特点：**
- 使用内存通道，极低延迟
- 零拷贝（在同一进程内）
- 支持一对多广播
- 异步非阻塞

**在项目中的使用：**
```rust
// DistributionCenter 使用 tokio::sync::broadcast
// 用于插件之间的消息分发
pub struct DistributionCenter {
    channels: HashMap<MessageType, broadcast::Sender<Message>>,
}
```

**消息流向：**
```
插件A → DistributionCenter (Tokio broadcast) → 插件B、插件C...
```

#### Iceoryx2 - 进程间通信（Inter-Process）

**使用场景：**
- ✅ **不同进程之间**的通信
- ✅ 与外部系统（如前端、QQ bot、其他服务）通信
- ✅ 跨进程边界的零拷贝通信

**技术特点：**
- 使用共享内存，零拷贝
- 跨进程边界
- 支持发布-订阅和请求-响应模式
- 高性能 IPC

**在项目中的使用：**
```rust
// Iceoryx2Dispatcher 使用 iceoryx2
// 用于与外部进程通信
pub struct Iceoryx2Dispatcher {
    service_name: String,
    // iceoryx2 连接
}
```

**消息流向：**
```
外部进程 → Iceoryx2Dispatcher (Iceoryx2) → MessageManager → DistributionCenter → 插件
```

### 分发器订阅机制

分发器支持选择性订阅消息类型，只有订阅了特定消息类型的分发器才会接收到相应的消息。

#### 订阅功能

**Dispatcher Trait 扩展：**
```rust
pub trait Dispatcher: Send + Sync {
    // ... 其他方法 ...
    
    /// 获取分发器订阅的消息类型列表
    /// 返回空集合表示订阅所有消息类型
    fn subscribed_message_types(&self) -> HashSet<MessageType>;
    
    /// 检查分发器是否订阅了指定的消息类型
    fn is_subscribed_to(&self, message_type: &MessageType) -> bool;
}
```

**使用示例：**
```rust
// 创建订阅特定消息类型的分发器
let dispatcher = Iceoryx2Dispatcher::new("service")
    .with_name("通知分发器")
    .subscribe_to(&["notification", "alert"]); // 只订阅这两种类型

// 创建订阅所有消息类型的分发器（默认）
let dispatcher_all = Iceoryx2Dispatcher::new("service2");
```

#### 消息路由逻辑

**插件发送消息后的路由流程：**

```
1. 插件发送消息
   ↓
2. MessageManager.handle_plugin_message()
   ↓
3. DistributionCenter.distribute() → 订阅的插件接收
   ↓
4. 过滤并发送给订阅的分发器
   - 检查 dispatcher.is_subscribed_to(&message.message_type)
   - 只有订阅了该类型的分发器才会收到消息
   - 空订阅列表表示订阅所有消息类型
```

**详细路由算法：**

```rust
// 在 MessageManager 中
pub async fn route_to_dispatchers(&self, message: &Message) -> Result<()> {
    for dispatcher in &self.dispatchers {
        // 检查分发器是否订阅了此消息类型
        if dispatcher.is_subscribed_to(&message.message_type) {
            // 异步发送消息到分发器
            dispatcher.send_message(message).await?;
        }
    }
    Ok(())
}
```

**订阅匹配规则：**
- **精确匹配**：分发器订阅了具体消息类型（如 `"notification"`）
- **通配符匹配**：分发器订阅列表为空，表示订阅所有类型
- **多类型订阅**：分发器可订阅多个消息类型

**示例场景：**

假设有三个分发器：
- **全量分发器**：订阅所有消息类型（`subscribed_types` 为空）
- **通知分发器**：只订阅 `"notification"` 类型
- **警报分发器**：订阅 `"alert"` 和 `"warning"` 类型

当插件发送不同类型的消息时：
- `"notification"` 消息 → 全量分发器 ✅、通知分发器 ✅
- `"alert"` 消息 → 全量分发器 ✅、警报分发器 ✅
- `"warning"` 消息 → 全量分发器 ✅、警报分发器 ✅
- `"other"` 消息 → 全量分发器 ✅（只有它订阅所有类型）

#### 优势

- ✅ **精确控制**：分发器可以只接收感兴趣的消息类型
- ✅ **减少开销**：避免不必要的消息传递和处理
- ✅ **灵活配置**：支持订阅单个或多个消息类型
- ✅ **向后兼容**：默认订阅所有消息类型，保持向后兼容

### 职责划分

| 特性 | Tokio Broadcast | Iceoryx2 |
|------|----------------|----------|
| **通信范围** | 进程内 | 进程间 |
| **使用场景** | 插件之间 | 与外部系统 |
| **技术实现** | 内存通道 | 共享内存 |
| **序列化** | 不需要 | 需要 |
| **延迟** | 极低（纳秒级） | 低（微秒级） |
| **适用数据** | 小到中等 | 大 |

### 为什么需要两者？

#### 场景 1：插件之间的通信

```
插件A 发送消息 → DistributionCenter (Tokio broadcast) → 插件B、C 接收
```

**为什么用 Tokio？**
- ✅ 同一进程内，无需跨进程开销
- ✅ 内存通道，延迟极低
- ✅ 简单高效

**为什么不用 Iceoryx2？**
- ❌ 过度设计，插件在同一进程内
- ❌ 增加不必要的序列化开销
- ❌ 增加系统复杂度

#### 场景 2：与外部系统通信

```
外部进程 → Iceoryx2Dispatcher (Iceoryx2) → MessageManager → 插件
```

**为什么用 Iceoryx2？**
- ✅ 跨进程边界，必须使用 IPC
- ✅ 零拷贝，适合大数据传输
- ✅ 高性能进程间通信

**为什么不用 Tokio？**
- ❌ Tokio broadcast 只能在同一进程内使用
- ❌ 无法跨进程边界
- ❌ 不适用于外部系统通信

---

## 技术栈

### 第三方库功能概览

#### 1. Tokio (v1.0) - 异步运行时和通道

**Tokio 提供的核心功能：**
- **异步运行时（Async Runtime）**
  - 多线程任务调度器（基于工作窃取算法）
  - 异步任务管理（spawn、join、abort）
  - 异步 I/O 操作（TCP、UDP、Unix 套接字等）

- **同步原语（Async Primitives）**
  - `tokio::sync::broadcast` - 一对多消息广播通道
  - `tokio::sync::mpsc` - 多生产者单消费者通道
  - `tokio::sync::RwLock` - 异步读写锁
  - `tokio::sync::Mutex` - 异步互斥锁

**项目如何使用 Tokio：**

1. **分发中心（DistributionCenter）**
   - ✅ 使用 `tokio::sync::broadcast` 实现发布-订阅模式
   - ✅ 使用 `tokio::sync::RwLock` 保护共享状态
   - ✅ 每个消息类型对应一个 broadcast channel
   - ✅ 异步订阅和消息分发

2. **消息管理器（MessageManager）**
   - ✅ 使用 `tokio::sync::mpsc` 实现消息队列
   - ✅ 使用 `tokio::spawn` 启动异步消息处理任务
   - ✅ 异步消息路由和分发

3. **消息上下文（MessageContext）**
   - ✅ 使用 `tokio::sync::broadcast::Receiver` 接收订阅的消息
   - ✅ 使用 `tokio::sync::mpsc::Sender` 发送消息

4. **应用运行时（App）**
   - ✅ 使用 `tokio::runtime::Runtime` 创建异步运行时
   - ✅ 使用 `block_on` 在同步代码中执行异步操作

**避免重复造轮子：**
- ❌ **不自己实现**：消息通道、异步运行时、任务调度器
- ✅ **使用 Tokio**：所有异步通信和并发控制

#### 2. Iceoryx2 (v0.4) - 零拷贝进程间通信

**Iceoryx2 提供的核心功能：**
- **零拷贝 IPC（Inter-Process Communication）**
  - 共享内存通信，避免数据拷贝
  - 高性能进程间消息传递
  - 支持发布-订阅和请求-响应模式

- **服务发现**
  - 自动服务注册和发现
  - 动态服务管理

**项目如何使用 Iceoryx2：**

1. **Iceoryx2Dispatcher（分发器实现）**
   - ✅ **当前状态**：已完全集成，支持零拷贝通信
   - 📝 **实际用途**：作为分发器的一种实现，用于与外部系统（如前端、QQ bot、其他服务）进行零拷贝通信
   - 📝 **已实现**：完整的 iceoryx2 连接初始化、消息发送逻辑和跨语言兼容性

2. **Python 测试套件**
   - ✅ **跨语言验证**：完整的 Python 测试脚本用于验证 Rust ↔ Python 通信
   - 📝 **测试覆盖**：发布者、订阅者、集成测试，支持多种消息类型
   - 📝 **文档完善**：详细的测试指南和使用说明

**避免重复造轮子：**
- ❌ **不自己实现**：共享内存 IPC、零拷贝通信机制
- ✅ **使用 Iceoryx2**：高性能进程间通信（已完全集成）

#### 3. Serde + Serde JSON - 序列化/反序列化

**Serde 提供的核心功能：**
- **通用序列化框架**
  - 支持多种数据格式（JSON、MessagePack、CBOR 等）
  - 通过 derive 宏自动实现序列化/反序列化
  - 类型安全的序列化

**项目如何使用 Serde：**

1. **消息格式（Message）**
   - ✅ 使用 `#[derive(Serialize, Deserialize)]` 自动实现序列化
   - ✅ 使用 `serde_json::to_string` 序列化为 JSON
   - ✅ 使用 `serde_json::from_str` 从 JSON 反序列化

2. **插件元数据（PluginMetadata）**
   - ✅ 支持序列化为 JSON 配置文件
   - ✅ 支持从 JSON 配置文件加载

**避免重复造轮子：**
- ❌ **不自己实现**：JSON 解析器、序列化框架
- ✅ **使用 Serde**：所有数据序列化需求

#### 4. Anyhow - 错误处理

**Anyhow 提供的核心功能：**
- **统一的错误类型**
  - `anyhow::Result<T>` - 简化的错误处理
  - `anyhow::Error` - 动态错误类型
  - 错误链和上下文信息

**项目如何使用 Anyhow：**

1. **所有函数返回类型**
   - ✅ 使用 `anyhow::Result<T>` 作为统一的错误返回类型
   - ✅ 使用 `anyhow::anyhow!` 创建错误
   - ✅ 使用 `?` 操作符进行错误传播

**避免重复造轮子：**
- ❌ **不自己实现**：错误类型系统、错误链管理
- ✅ **使用 Anyhow**：所有错误处理需求

### 技术栈职责划分

| 库 | 职责 | 项目中的使用 |
|---|---|---|
| **Tokio** | 异步运行时、通道、并发控制 | 消息分发、异步任务、并发管理 |
| **Iceoryx2** | 零拷贝 IPC | 分发器实现（待完全集成） |
| **Serde** | 序列化框架 | 消息和元数据序列化 |
| **Serde JSON** | JSON 格式支持 | JSON 序列化/反序列化 |
| **Anyhow** | 错误处理 | 统一错误类型和处理 |

| 模块 | 职责 | 实现方式 |
|---|---|---|
| **插件系统** | 插件生命周期、注册、管理 | 自定义实现，使用 Serde 序列化 |
| **消息系统** | 消息格式、路由、分发 | 基于 Tokio 通道实现 |
| **分发器系统** | 分发器抽象和管理 | 自定义 trait，使用 Iceoryx2 实现 |
| **应用构建器** | 应用配置和启动 | 自定义实现，集成所有系统 |

---

## 代码优化

### 优化前后对比

#### ❌ 优化前的 main.rs (41 行)

```rust
mod plugin;
mod plugins;

use plugin::PluginRegistry;
use plugins::code4rena::Code4renaPlugin;
use plugins::example_plugin::ExamplePlugin;

fn main() -> anyhow::Result<()> {
    println!("=== Amadeus 插件系统启动 ===\n");

    let mut registry = PluginRegistry::new();

    // 手动注册每个插件 - 繁琐！
    registry.register(Code4renaPlugin::new());
    registry.register(ExamplePlugin::new());
    // 添加新插件需要修改这里...
    
    registry.list_plugins();

    match registry.export_metadata() {
        Ok(json) => {
            println!("\n=== 插件元数据 (JSON) ===");
            println!("{}", json);
        }
        Err(e) => eprintln!("导出元数据失败: {}", e),
    }

    // 手动调用每个生命周期
    registry.init_all()?;
    registry.start_all()?;
    registry.run_all()?;
    registry.stop_all()?;

    println!("\n=== Amadeus 插件系统已关闭 ===");
    Ok(())
}
```

#### ✅ 优化后的 main.rs (8 行)

```rust
use amadeus::App;

fn main() -> anyhow::Result<()> {
    // 一行搞定！
    App::new()
        .show_metadata(true)
        .run()
}
```

**减少了 80% 的代码！**

### 主要优化内容

#### 1. 🎯 自动插件收集系统

**创建 `src/plugins/mod.rs`：**

```rust
pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(Code4renaPlugin::new()),
        Box::new(ExamplePlugin::new()),
        // 添加新插件只需要在这里添加一行
    ]
}
```

**优势：**
- ✅ 集中管理所有插件
- ✅ 添加新插件只需修改一个地方
- ✅ 自动返回所有可用插件

#### 2. 🏗️ 应用构建器（App）

**创建 `src/app.rs`：**

```rust
pub struct App {
    registry: PluginRegistry,
    show_metadata: bool,
    show_startup_message: bool,
}

impl App {
    pub fn new() -> Self { /* ... */ }
    pub fn show_metadata(mut self, show: bool) -> Self { /* ... */ }
    pub fn run(mut self) -> Result<()> { /* ... */ }
}
```

**使用示例：**
```rust
// 最简单的使用
App::new().run()?;

// 带配置
App::new()
    .show_metadata(true)
    .show_startup_message(false)
    .run()?;
```

**优势：**
- ✅ 封装所有常见操作
- ✅ 提供优雅的配置 API
- ✅ 隐藏实现细节

#### 3. 📦 库和二进制分离

**创建 `src/lib.rs`：**

```rust
pub mod app;
pub mod plugin;
pub mod plugins;

pub use app::App;
pub use plugin::{Plugin, PluginMetadata, PluginRegistry};
```

**优势：**
- ✅ 可以作为库被其他项目使用
- ✅ 支持编写示例和测试
- ✅ 更好的代码组织

### 代码质量提升

| 指标 | 优化前 | 优化后 | 改进 |
|------|--------|--------|------|
| main.rs 行数 | 41 | 8 | ⬇️ 80% |
| 添加新插件步骤 | 3 处 | 1 处 | ⬇️ 66% |
| 手动注册代码 | 必需 | 可选 | ✅ |
| API 灵活性 | 低 | 高 | ⬆️ |
| 代码复用性 | 低 | 高 | ⬆️ |

### 设计模式应用

1. **构建器模式（Builder Pattern）**
   - `App::new().show_metadata(true).run()`
   - `PluginMetadata::new().author().with_property()`

2. **工厂模式（Factory Pattern）**
   - `plugins::get_all_plugins()`
   - `PluginRegistry::with_enabled_plugins()`

3. **门面模式（Facade Pattern）**
   - `App` 封装复杂的注册和生命周期管理

---

## 文件结构

### 项目结构概览

```
amadeus/
├── 📄 Cargo.toml                        # 项目配置和依赖
├── 📄 Cargo.lock                        # 依赖锁定文件
│
├── 📖 README.md                         # 项目主文档
├── 📖 DEVELOPER_GUIDE.md               # 开发者指南
├── 📖 ARCHITECTURE.md                  # 架构文档（本文件）
│
├── 📋 plugins_config.example.json      # JSON 配置文件示例
│
├── 📂 src/                              # 源代码目录
│   ├── 🦀 main.rs                      # 程序入口点
│   ├── 🦀 lib.rs                       # 库入口
│   ├── 🦀 app.rs                       # 应用构建器
│   ├── 🦀 plugin.rs                    # 插件系统核心
│   ├── 🦀 message.rs                   # 消息格式
│   ├── 🦀 message_context.rs           # 消息上下文
│   ├── 🦀 message_manager.rs           # 消息管理器
│   ├── 🦀 distribution_center.rs       # 分发中心
│   │
│   ├── 📂 dispatcher/                  # 分发器目录
│   │   ├── mod.rs                      # 分发器 trait
│   │   └── iceoryx2.rs                 # Iceoryx2 分发器
│   │
│   └── 📂 plugins/                     # 插件目录
│       ├── 🦀 mod.rs                   # 插件模块导出
│       ├── 🦀 code4rena.rs            # Code4rena 插件（单文件）
│       └── 📂 example_plugin/         # Example 插件（多文件）
│           ├── 🦀 mod.rs              # 插件主模块
│           ├── 🦀 config.rs           # 配置管理模块
│           └── 🦀 handler.rs          # 数据处理模块
│
└── 📂 target/                          # 编译输出（自动生成）
```

### 核心文件说明

#### `src/main.rs` - 程序入口
- 功能：创建应用并运行
- 行数：约 8 行（优化后）

#### `src/lib.rs` - 库入口
- 功能：导出公共 API
- 用途：作为库被其他项目使用

#### `src/app.rs` - 应用构建器
- 功能：封装插件注册和生命周期管理
- 设计模式：构建器模式、门面模式

#### `src/plugin.rs` - 插件系统核心
- 包含：
  - Plugin trait（插件接口）
  - PluginMetadata（元数据结构）
  - PluginRegistry（注册表）
- 功能：
  - 定义插件接口
  - 管理插件生命周期
  - JSON 配置加载/保存
  - 插件列表和导出

#### `src/dispatcher/mod.rs` - 分发器系统
- 功能：定义分发器接口和注册表
- 核心特性：
  - `Dispatcher` trait：分发器抽象接口
  - `DispatcherRegistry`：分发器注册和管理
  - **消息订阅机制**：分发器可以订阅特定消息类型
- 订阅功能：
  - `subscribed_message_types()`：获取订阅的消息类型
  - `is_subscribed_to()`：检查是否订阅了特定消息类型
  - 默认订阅所有消息类型（空集合）

#### `src/dispatcher/iceoryx2.rs` - Iceoryx2 分发器
- 功能：基于 Iceoryx2 的分发器实现
- 特性：
  - 零拷贝进程间通信
  - 支持消息类型订阅
  - `subscribe_to()`：订阅多个消息类型
  - `subscribe_to_one()`：订阅单个消息类型

#### `src/message.rs` - 消息格式
- 功能：定义统一的消息格式
- 特性：支持序列化、优先级、来源追踪

#### `src/distribution_center.rs` - 分发中心
- 功能：消息路由和分发
- 技术：使用 Tokio broadcast 实现发布-订阅

#### `src/message_manager.rs` - 消息管理器
- 功能：协调分发中心和分发器，处理消息路由
- 核心逻辑：
  - 接收来自插件和分发器的消息
  - 通过 `DistributionCenter` 分发给订阅的插件
  - **只发送给订阅了该消息类型的分发器**（使用 `is_subscribed_to()` 过滤）
- 消息处理流程：
  1. 接收消息（通过 mpsc 通道）
  2. 分发给订阅的插件（通过 broadcast）
  3. 过滤并发送给订阅的分发器（基于消息类型）
- 功能：协调消息流
- 技术：使用 Tokio mpsc 实现消息队列

### 文件依赖关系

```
main.rs
  └─► lib.rs
        ├─► app.rs
        │     ├─► plugin.rs
        │     └─► message_manager.rs
        │           ├─► distribution_center.rs
        │           ├─► message.rs
        │           └─► dispatcher/
        │
        └─► plugins/
              ├─► code4rena.rs
              │     └─► plugin.rs (trait)
              │
              └─► example_plugin/
                    ├─► mod.rs (实现 Plugin trait)
                    ├─► config.rs
                    └─► handler.rs
```

### 代码统计

```
语言：Rust
核心代码：约 2000+ 行
文档：约 3000+ 行
总计：约 5000+ 行

代码分布：
- src/plugin.rs:        约 200 行
- src/app.rs:           约 150 行
- src/message*.rs:      约 400 行
- src/dispatcher/:      约 200 行
- src/plugins/:         约 300 行
- src/main.rs:          约 10 行
```

---

## 总结

### 架构优势

1. **类型安全** ✓
   - 编译时检查
   - 无运行时类型错误

2. **内存安全** ✓
   - 所有权系统保证
   - 无内存泄漏

3. **线程安全** ✓
   - Send + Sync 约束
   - 无数据竞争

4. **零成本抽象** ✓
   - Trait 编译时单态化
   - 无虚函数表开销（静态分发）

5. **易于扩展** ✓
   - 清晰的接口
   - 灵活的元数据系统

6. **高性能** ✓
   - 进程内使用 Tokio（极低延迟）
   - 进程间使用 Iceoryx2（零拷贝）

### 通信层次划分

- **进程内通信**：Tokio broadcast（插件之间）
- **进程间通信**：Iceoryx2（与外部系统）
- **两者互补**：不重复，服务于不同场景

### 职责划分

- **第三方库**：提供基础设施（异步、序列化、IPC、错误处理）
- **项目本体**：实现业务逻辑（插件管理、消息路由、应用架构）

---

这个架构图展示了 Amadeus 插件系统的完整设计思路和实现细节！🎯

## 项目路线图 (Roadmap)

### 1. 目标架构 (Target Architecture)

重构后的 Amadeus 将采用**混合内核**架构：

```mermaid
graph TD
    User[用户前端 (Tauri/TUI)] <--> API[API Layer]
    
    subgraph Core System
        MsgBus[异步消息总线 (Tokio MPSC/Broadcast)]
        Scheduler[时间调度器 (Tokio-Cron)]
        DB[(持久化存储 (SQLite))]
    end
    
    subgraph Plugin Host
        NativePlugins[Core Native Plugins]
        WasmRuntime[WASM Runtime (Extism)]
    end
    
    subgraph External IPC
        Iceoryx2[Iceoryx2 Gateway]
    end
    
    API --> MsgBus
    MsgBus <--> Scheduler
    MsgBus <--> DB
    MsgBus <--> NativePlugins
    MsgBus <--> WasmRuntime
    MsgBus <--> Iceoryx2
    
    WasmRuntime -.-> |加载| WasmFiles[External .wasm Plugins]
```

### 2. 演进计划

#### Phase 1: 基础设施搭建 (当前阶段)
- [x] **架构重构**: 废弃阻塞式 `run`，引入异步生命周期。
- [x] **日志升级**: 全面迁移至 `tracing` 生态。
- [ ] **Core Plugins**:
    - [ ] **Storage**: 搭建 SQLite 连接池与基础 Schema (`memos` 表)。
    - [ ] **Scheduler**: 集成 `tokio-cron-scheduler` 并连接到消息总线。
- [ ] **IPC 完善**: 实现 Iceoryx2 与内部消息总线的双向桥接。

#### Phase 2: 业务功能与扩展能力
- [ ] **业务逻辑**: 完善备忘录的 CRUD 消息处理逻辑。
- [ ] **WASM 运行时**:
    - [ ] 集成 `Extism`。
    - [ ] 实现 WASM 插件加载器 (`WasmPlugin`)。
    - [ ] 编写第一个 WASM 插件示例。

#### Phase 3: 交互层 (未来规划)
- [ ] **Tauri 集成**: 构建跨平台 GUI。
- [ ] **API 层**: 实现前端 UI 与 Core 的通信接口。

### 3. 技术选型

| 模块 | 选型 | 说明 |
| :--- | :--- | :--- |
| **持久化** | **SQLx (SQLite)** | 轻量级、无服务器、SQL支持，适合结构化备忘录数据。 |
| **调度器** | **tokio-cron-scheduler** | 成熟的异步 Cron 调度库。 |
| **插件运行时** | **Extism** | 高性能 WASM 运行时，支持多语言编写插件，安全隔离。 |
| **IPC** | **Iceoryx2** | 零拷贝进程间通信，作为外部高性能 IPC 网关。 |
| **前端** | **Tauri** | (规划中) 基于 Web 技术栈的跨平台 GUI。 |
