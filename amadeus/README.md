# 🚀 Amadeus 插件系统

一个简单的的 Rust 插件架构系统。

## ✨ 特性

- 🎯 **基于 Trait** - 使用 Rust trait 定义清晰的插件接口
- 🔄 **完整生命周期** - init → start → run → stop 四阶段管理
- 📝 **元数据系统** - 丰富的插件信息（名称、版本、作者等）
- 💾 **JSON 序列化** - 支持配置文件的导入导出
- 🎨 **灵活结构** - 支持单文件和多文件插件组织
- 🔒 **类型安全** - 编译时检查，零运行时开销
- ⚡ **零成本抽象** - 充分利用 Rust 的性能优势

## 📦 项目结构

```
amadeus/
├── src/
│   ├── main.rs                    # 主程序入口
│   ├── plugin.rs                  # 插件系统核心
│   └── plugins/                   # 插件目录
│       ├── mod.rs                 # 插件模块导出
│       ├── code4rena.rs          # 单文件插件示例
│       └── example_plugin/        # 多文件插件示例
│           ├── mod.rs
│           ├── config.rs
│           └── handler.rs
├── Cargo.toml                     # 项目依赖
├── README.md                      # 本文件
├── QUICKSTART.md                  # 快速入门指南
├── PLUGIN_SYSTEM.md               # 详细设计文档
├── ADVANCED_FEATURES.md           # 高级功能指南
└── plugins_config.example.json    # 配置文件示例
```

## 🚀 快速开始

### 1. 运行示例

```bash
# 克隆项目
cd amadeus

# 运行
cargo run
```

### 2. 查看输出

```
=== Amadeus 插件系统启动 ===

注册插件: code4rena
注册插件: example_plugin

=== 已注册的插件 ===
1. code4rena v0.1.0 - Code4rena 漏洞扫描和分析插件 [启用]
2. example_plugin v0.1.0 - 一个示例插件，展示多文件插件结构 [禁用]

=== 初始化所有插件 ===
[Code4rena] 正在初始化插件...
...
```

## 📖 文档

- **[快速入门](./QUICKSTART.md)** - 5 分钟创建你的第一个插件
- **[系统设计](./PLUGIN_SYSTEM.md)** - 完整的架构设计文档
- **[高级功能](./ADVANCED_FEATURES.md)** - 配置管理、异步支持等

## 💻 创建插件

### 最简示例

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
        );
        Self { metadata }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn run(&mut self) -> Result<()> {
        println!("Hello from MyPlugin!");
        Ok(())
    }
}
```

查看 [QUICKSTART.md](./QUICKSTART.md) 了解详细步骤。

## 🎯 核心概念

### 1. Plugin Trait

所有插件都必须实现 `Plugin` trait：

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    fn init(&mut self) -> Result<()>;
    fn start(&mut self) -> Result<()>;
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}
```

### 2. 生命周期

插件有四个生命周期阶段：

1. **init()** - 初始化（加载配置、准备资源）
2. **start()** - 启动（建立连接、启动服务）
3. **run()** - 运行（执行主要逻辑）
4. **stop()** - 停止（清理资源、保存状态）

### 3. 元数据

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

### 4. 注册表

`PluginRegistry` 管理所有插件：

```rust
let mut registry = PluginRegistry::new();
registry.register(MyPlugin::new());
registry.init_all()?;
registry.run_all()?;
registry.stop_all()?;
```

## 📊 示例插件

### Code4rena 插件（单文件）

一个简单的安全扫描插件，展示单文件插件结构。

### Example 插件（多文件）

一个复杂的示例插件，展示如何组织多文件插件：
- `mod.rs` - 主模块
- `config.rs` - 配置管理
- `handler.rs` - 数据处理

## 🔧 依赖

```toml
[dependencies]
anyhow = "1.0"       # 错误处理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"   # JSON 支持
```

## 🌟 高级功能

- 📁 **配置文件** - JSON 配置导入导出
- 🔄 **插件通信** - 共享状态机制
- ⚡ **异步支持** - async/await 插件
- 🎯 **依赖管理** - 插件间依赖关系
- 🔥 **热重载** - 运行时重新加载插件
- 📊 **优先级** - 控制插件执行顺序

查看 [ADVANCED_FEATURES.md](./ADVANCED_FEATURES.md) 了解详情。

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_plugin_lifecycle

# 带日志输出
cargo test -- --nocapture
```

## 📈 性能

- ✅ 零成本抽象 - trait 在编译时单态化
- ✅ 类型安全 - 编译时检查，无运行时开销
- ✅ 内存安全 - Rust 所有权系统保证
- ✅ 无数据竞争 - Send + Sync 保证线程安全

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证。

## 🙏 致谢

感谢 Rust 社区提供的优秀工具和库！

---

**开始创建您的插件吧！** 🚀

如果有任何问题，请查看文档或提交 issue。

