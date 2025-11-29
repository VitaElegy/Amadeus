# Amadeus

Amadeus 是一个高性能、模块化的 Rust 插件系统框架。它旨在提供一个灵活的基础设施，用于构建复杂的、分布式的应用程序。

## 🌟 特性

*   **双模块核心架构**：精简的 Message/Plugin Center + 强大的插件生态。
*   **灵活的插件系统**：
    *   **Native 插件**：Rust 原生编写，高性能。
    *   **WASM 插件** (开发中)：支持多语言，安全沙盒隔离。
    *   **特权分级**：支持 `Privileged` 和 `Normal` 插件类型。
*   **强大的消息总线**：
    *   **Public 广播**：基于 `tokio::broadcast` 的高效发布/订阅。
    *   **Direct 定向**：基于 `tokio::mpsc` 的点对点私密通信。
*   **高性能 IPC**：集成 **Iceoryx2** 实现零拷贝进程间通信（通过特权插件）。
*   **异步优先**：全链路基于 Tokio 异步运行时。

## 🚀 快速开始

### 运行示例

```bash
# 运行消息系统示例
cargo run --example messaging
```

### 项目结构

```
amadeus/
├── src/
│   ├── core/           # 核心架构 (消息总线、路由)
│   ├── plugins/        # 内置插件 (CoreSystem, IPC Dispatcher 等)
│   ├── app.rs          # 应用入口
│   └── lib.rs          # 库定义
└── doc/                # 文档
    ├── ARCHITECTURE.md # 架构设计
    └── ...
```

## 📖 文档

*   [架构设计 (Architecture)](doc/ARCHITECTURE.md)
*   [开发者指南 (Developer Guide)](doc/DEVELOPER_GUIDE.md)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

[MIT License](LICENSE)
