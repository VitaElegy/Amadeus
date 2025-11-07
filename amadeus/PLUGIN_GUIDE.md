# Amadeus 插件系统开发指南

## 概述

Amadeus 插件系统是一个灵活、类型安全的插件架构，基于 Rust 的 trait 系统实现。

## 核心特性

- ✅ **类型安全**: 基于 Rust trait 的强类型系统
- ✅ **生命周期管理**: 完整的插件生命周期 (init → start → run → stop)
- ✅ **元数据支持**: 可序列化的插件元信息（支持 JSON）
- ✅ **灵活配置**: 支持自定义属性和配置
- ✅ **易于扩展**: 简单的插件注册机制

## 插件生命周期

```
注册 → 初始化 (init) → 启动 (start) → 运行 (run) → 停止 (stop)
```

1. **init**: 插件初始化，加载配置、初始化资源
2. **start**: 插件启动，建立连接、准备运行环境
3. **run**: 插件主要逻辑执行
4. **stop**: 插件停止，清理资源、保存状态

## 如何创建新插件

### 步骤 1: 创建插件文件

在 `src/plugins/` 目录下创建新的插件文件，例如 `my_plugin.rs`：

```rust
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct MyPlugin {
    metadata: PluginMetadata,
    // 添加你的插件状态字段
}

impl MyPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "my_plugin",              // 插件名称
            "我的插件描述",            // 插件描述
            "0.1.0",                  // 版本号
        )
        .enabled_by_default(true)     // 是否默认启用
        .author("Your Name")          // 作者
        .with_property("category", "utility"); // 自定义属性

        Self { metadata }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        println!("[MyPlugin] 初始化...");
        // 你的初始化逻辑
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        println!("[MyPlugin] 启动...");
        // 你的启动逻辑
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        println!("[MyPlugin] 运行...");
        // 你的主要逻辑
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!("[MyPlugin] 停止...");
        // 清理资源
        Ok(())
    }
}
```

### 步骤 2: 在 mod.rs 中声明模块

编辑 `src/plugins/mod.rs`，添加：

```rust
pub mod my_plugin;
```

### 步骤 3: 在 main.rs 中注册插件

编辑 `src/main.rs`，注册你的插件：

```rust
use plugins::my_plugin::MyPlugin;

fn main() -> anyhow::Result<()> {
    let mut registry = PluginRegistry::new();
    
    // 注册插件
    registry.register(MyPlugin::new());
    
    // ... 其他代码
}
```

## 插件元数据

插件元数据支持以下字段：

```rust
PluginMetadata {
    name: String,                    // 插件名称（必填）
    description: String,             // 插件描述（必填）
    version: String,                 // 版本号（必填）
    enabled_by_default: bool,        // 是否默认启用
    author: Option<String>,          // 作者
    properties: HashMap<String, String>, // 自定义属性
}
```

## 插件结构示例

### 单文件插件
```
src/plugins/
  ├── mod.rs
  ├── code4rena.rs    # 单文件插件
  └── my_plugin.rs    # 单文件插件
```

### 多文件插件（推荐用于复杂插件）
```
src/plugins/
  ├── mod.rs
  ├── code4rena.rs
  └── complex_plugin/  # 文件夹插件
      ├── mod.rs
      ├── config.rs
      ├── handler.rs
      └── utils.rs
```

对于多文件插件，在 `src/plugins/mod.rs` 中声明：
```rust
pub mod complex_plugin;
```

然后在 `complex_plugin/mod.rs` 中组织你的模块。

## JSON 配置支持

插件元数据可以导出为 JSON 格式：

```rust
let json = registry.export_metadata()?;
println!("{}", json);
```

输出示例：
```json
[
  {
    "name": "code4rena",
    "description": "Code4rena 漏洞扫描和分析插件",
    "version": "0.1.0",
    "enabled_by_default": true,
    "author": "Amadeus Team",
    "properties": {
      "category": "security",
      "priority": "high"
    }
  }
]
```

## 高级用法

### 自定义生命周期逻辑

你可以在任何生命周期方法中添加自定义逻辑，如果不需要，可以使用默认实现：

```rust
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    // 只实现你需要的方法，其他使用默认实现
    fn run(&mut self) -> Result<()> {
        // 只有 run 有自定义逻辑
        println!("执行主要任务");
        Ok(())
    }
}
```

### 条件启用插件

```rust
let mut plugin = MyPlugin::new();
// 在运行时修改元数据
if some_condition {
    plugin.metadata.enabled_by_default = false;
}
registry.register(plugin);
```

### 错误处理

所有生命周期方法都返回 `anyhow::Result<()>`，可以使用 `?` 运算符传播错误：

```rust
fn init(&mut self) -> Result<()> {
    let config = self.load_config()?;  // 自动传播错误
    self.validate(config)?;
    Ok(())
}
```

## 最佳实践

1. **清晰的职责划分**: 每个生命周期方法应该有明确的职责
2. **资源管理**: 在 `stop` 方法中清理所有资源
3. **错误处理**: 使用 `anyhow` 提供清晰的错误信息
4. **日志输出**: 在关键操作点添加日志输出
5. **元数据完整性**: 提供完整的插件元信息

## 参考示例

查看 `src/plugins/code4rena.rs` 获取完整的插件实现示例。

