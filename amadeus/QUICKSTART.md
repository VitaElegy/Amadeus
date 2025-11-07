# 快速上手 - 创建您的第一个插件

## 5 分钟创建一个插件

### 步骤 1: 创建插件文件

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
        println!("🚀 [MyPlugin] 插件初始化成功！");
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        println!("✨ [MyPlugin] 插件正在运行...");
        println!("✨ [MyPlugin] 做一些有趣的事情！");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!("👋 [MyPlugin] 插件已停止");
        Ok(())
    }
}
```

### 步骤 2: 导出插件模块

编辑 `src/plugins/mod.rs`，添加：

```rust
pub mod my_plugin;
```

### 步骤 3: 注册插件

编辑 `src/main.rs`，在顶部添加 use 语句：

```rust
use plugins::my_plugin::MyPlugin;
```

然后在 `main` 函数中注册：

```rust
registry.register(MyPlugin::new());
```

### 步骤 4: 运行！

```bash
cargo run
```

就这么简单！您的插件已经运行了！🎉

## 运行结果示例

```
=== Amadeus 插件系统启动 ===

注册插件: code4rena
注册插件: example_plugin
注册插件: my_plugin        ← 您的新插件！

=== 已注册的插件 ===
1. code4rena v0.1.0 - Code4rena 漏洞扫描和分析插件 [启用]
2. example_plugin v0.1.0 - 一个示例插件，展示多文件插件结构 [禁用]
3. my_plugin v0.1.0 - 我的第一个插件 [启用]

=== 初始化所有插件 ===
🚀 [MyPlugin] 插件初始化成功！

=== 运行所有插件 ===
✨ [MyPlugin] 插件正在运行...
✨ [MyPlugin] 做一些有趣的事情！

=== 停止所有插件 ===
👋 [MyPlugin] 插件已停止
```

## 下一步

- 📖 阅读 [PLUGIN_SYSTEM.md](./PLUGIN_SYSTEM.md) 了解完整的设计文档
- 🔍 查看 `src/plugins/code4rena.rs` 学习单文件插件
- 📦 查看 `src/plugins/example_plugin/` 学习多文件插件
- 🚀 添加更多的生命周期方法和功能

祝您开发愉快！✨

