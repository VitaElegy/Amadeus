# Amadeus 插件系统设计文档

## 📋 概述

这是一个功能完善的 Rust 插件系统，充分利用了 Rust 的类型系统、trait 系统和所有权机制，提供了灵活且类型安全的插件架构。

## 🎯 核心特性

### 1. **基于 Trait 的插件接口**
- 使用 `Plugin` trait 定义统一的插件接口
- 所有插件必须实现 `Plugin` trait
- 支持默认实现，简化插件开发

### 2. **完整的生命周期管理**
插件系统提供四个生命周期钩子：
- `init()` - 初始化插件（加载配置、初始化状态）
- `start()` - 启动插件（建立连接、准备资源）
- `run()` - 运行插件主逻辑
- `stop()` - 停止插件（清理资源、保存状态）

### 3. **丰富的元数据系统**
```rust
pub struct PluginMetadata {
    pub name: String,                    // 插件名称
    pub description: String,             // 插件描述
    pub version: String,                 // 版本号
    pub enabled_by_default: bool,        // 是否默认启用
    pub author: Option<String>,          // 作者
    pub properties: HashMap<String, String>, // 自定义属性
}
```

### 4. **JSON 序列化支持**
- 使用 `serde` 实现序列化/反序列化
- 可导出所有插件元数据为 JSON
- 方便与配置文件系统集成

### 5. **灵活的插件结构**
支持两种插件组织方式：
- **单文件插件** - 简单插件，所有代码在一个文件中（如 `code4rena.rs`）
- **多文件插件** - 复杂插件，使用文件夹组织（如 `example_plugin/`）

## 🏗️ 架构设计

```
src/
├── main.rs                      # 主程序入口
├── plugin.rs                    # 插件系统核心
│   ├── Plugin trait             # 插件接口定义
│   ├── PluginMetadata          # 插件元数据
│   └── PluginRegistry          # 插件注册表
└── plugins/                     # 插件目录
    ├── mod.rs                   # 插件模块导出
    ├── code4rena.rs            # 单文件插件示例
    └── example_plugin/          # 多文件插件示例
        ├── mod.rs              # 插件主模块
        ├── config.rs           # 配置模块
        └── handler.rs          # 处理器模块
```

## 💡 Rust 特性应用

### 1. **Trait 系统**
```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    fn init(&mut self) -> Result<()>;
    fn start(&mut self) -> Result<()>;
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}
```
- `Send + Sync` 确保插件可以安全地在线程间传递
- 提供默认实现，减少样板代码

### 2. **构建器模式（Builder Pattern）**
```rust
let metadata = PluginMetadata::new("name", "description", "0.1.0")
    .enabled_by_default(true)
    .author("Amadeus Team")
    .with_property("category", "security");
```
- 链式调用，优雅地构建复杂对象
- 类型安全，编译时检查

### 3. **类型擦除（Type Erasure）**
```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}
```
- 使用 `Box<dyn Plugin>` 存储不同类型的插件
- 运行时多态，统一管理

### 4. **错误处理**
```rust
use anyhow::Result;

fn init(&mut self) -> Result<()> {
    // ...
}
```
- 使用 `anyhow::Result` 简化错误处理
- `?` 操作符优雅地传播错误

## 📝 使用指南

### 创建单文件插件

1. 在 `src/plugins/` 下创建 `your_plugin.rs`：

```rust
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct YourPlugin {
    metadata: PluginMetadata,
    // 插件状态字段
}

impl YourPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "your_plugin",
            "插件描述",
            "0.1.0",
        )
        .enabled_by_default(true)
        .author("Your Name");

        Self { metadata }
    }
}

impl Plugin for YourPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        println!("[YourPlugin] 初始化");
        Ok(())
    }

    // 实现其他生命周期方法...
}
```

2. 在 `src/plugins/mod.rs` 中添加：
```rust
pub mod your_plugin;
```

3. 在 `main.rs` 中注册：
```rust
use plugins::your_plugin::YourPlugin;

registry.register(YourPlugin::new());
```

### 创建多文件插件

1. 创建插件目录：`src/plugins/your_plugin/`

2. 创建 `mod.rs`（主模块）：
```rust
mod config;
mod handler;

use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct YourPlugin {
    metadata: PluginMetadata,
    config: config::Config,
    handler: handler::Handler,
}

impl YourPlugin {
    pub fn new() -> Self {
        // ...
    }
}

impl Plugin for YourPlugin {
    // 实现 trait 方法
}
```

3. 创建辅助模块（`config.rs`, `handler.rs` 等）

4. 在 `src/plugins/mod.rs` 中导出：
```rust
pub mod your_plugin;
```

### 插件注册与执行

```rust
fn main() -> anyhow::Result<()> {
    // 创建注册表
    let mut registry = PluginRegistry::new();

    // 注册插件
    registry.register(Plugin1::new());
    registry.register(Plugin2::new());

    // 列出所有插件
    registry.list_plugins();

    // 导出元数据
    let json = registry.export_metadata()?;
    println!("{}", json);

    // 执行生命周期
    registry.init_all()?;
    registry.start_all()?;
    registry.run_all()?;
    registry.stop_all()?;

    Ok(())
}
```

## 🔧 高级特性

### 1. 条件启用插件

通过 `enabled_by_default` 控制插件是否启用：
```rust
.enabled_by_default(false)
```

只有启用的插件会执行生命周期方法。

### 2. 自定义属性

使用 `with_property` 添加任意键值对：
```rust
.with_property("category", "security")
.with_property("priority", "high")
.with_property("api_version", "v2")
```

### 3. 插件状态管理

在插件结构体中维护状态：
```rust
pub struct MyPlugin {
    metadata: PluginMetadata,
    is_running: bool,
    connection: Option<Connection>,
    data_cache: HashMap<String, String>,
}
```

### 4. 错误处理

使用 `anyhow::bail!` 返回错误：
```rust
fn start(&mut self) -> Result<()> {
    if !self.is_initialized {
        anyhow::bail!("插件未初始化");
    }
    Ok(())
}
```

### 5. 生命周期顺序

- `init_all()` - 按注册顺序初始化
- `start_all()` - 按注册顺序启动
- `run_all()` - 按注册顺序运行
- `stop_all()` - **按相反顺序停止**（确保正确清理依赖）

## 🚀 扩展建议

### 1. 配置文件支持

创建 `plugins_config.json`：
```json
[
  {
    "name": "code4rena",
    "enabled_by_default": true,
    "properties": {
      "api_key": "your-api-key",
      "scan_interval": "3600"
    }
  }
]
```

在 `PluginRegistry` 中添加：
```rust
pub fn load_from_config(path: &str) -> Result<Vec<PluginMetadata>> {
    let file = std::fs::read_to_string(path)?;
    let metadata: Vec<PluginMetadata> = serde_json::from_str(&file)?;
    Ok(metadata)
}
```

### 2. 插件依赖管理

在 `PluginMetadata` 中添加：
```rust
pub dependencies: Vec<String>,
```

实现拓扑排序，按依赖顺序加载插件。

### 3. 热重载支持

使用 `libloading` crate 动态加载插件：
```rust
pub fn load_dynamic_plugin(path: &str) -> Result<Box<dyn Plugin>> {
    // 使用 libloading 加载动态库
}
```

### 4. 插件通信

使用消息总线或事件系统：
```rust
pub trait Plugin {
    fn on_message(&mut self, msg: &Message) -> Result<()>;
}
```

### 5. 异步支持

改用 `async trait`：
```rust
#[async_trait]
pub trait Plugin {
    async fn init(&mut self) -> Result<()>;
    async fn run(&mut self) -> Result<()>;
}
```

### 6. 插件优先级

添加优先级排序：
```rust
pub priority: i32,

// 按优先级排序插件
registry.sort_by_priority();
```

## 📚 示例插件

系统包含两个示例插件：

1. **Code4rena 插件** (`code4rena.rs`)
   - 单文件插件
   - 安全扫描功能
   - 展示基本的生命周期实现

2. **Example 插件** (`example_plugin/`)
   - 多文件插件
   - 包含配置和处理器模块
   - 展示复杂插件的组织方式

## ✅ 最佳实践

1. **单一职责** - 每个插件只做一件事
2. **优雅错误处理** - 使用 `Result` 和 `?` 操作符
3. **清理资源** - 在 `stop()` 中释放所有资源
4. **日志记录** - 在关键步骤添加日志
5. **配置验证** - 在 `init()` 中验证配置
6. **状态检查** - 在操作前检查插件状态
7. **文档注释** - 为公开 API 添加文档
8. **测试** - 为每个插件编写单元测试

## 🎓 总结

这个插件系统充分利用了 Rust 的：
- ✅ **类型安全** - 编译时检查，运行时零成本
- ✅ **所有权系统** - 内存安全，无数据竞争
- ✅ **Trait 系统** - 抽象能力强，代码复用度高
- ✅ **零成本抽象** - 性能与手写代码相当
- ✅ **强大的模块系统** - 清晰的代码组织

这是一个生产级别的插件架构，可以轻松扩展到支持数百个插件！🚀

