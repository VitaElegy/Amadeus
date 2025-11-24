# 架构评审与重构路线图

**Commit ID**: 1e426a7 (Base) -> Refactored
**日期**: 2024-11-24
**评审人**: 刻薄专业的开发大师

## 1. 现状评估

### 1.1 核心问题
当前项目 (`1e426a7`) 虽然构建了一个基于消息订阅的初步框架，但存在明显的“过度设计”与“功能缺失”并存的问题。

*   **过度设计 (Over-engineering)**: 引入 `iceoryx2` 处理简单的备忘录消息流。这类似于用 F1 赛车去送外卖，增加了巨大的系统复杂度和 unsafe 代码风险。
*   **伪插件系统**: 目前的 `Plugin` trait 依赖于 Rust 的静态分发 (`Box<dyn Plugin>`)，导致所有插件必须与主程序共同编译。这违反了插件系统的“热插拔”和“独立部署”初衷。
*   **并发瓶颈**: `MessageManager` 在异步运行时中使用了同步互斥锁 (`Mutex`) 来遍历分发器，这是严重的反模式，会导致高并发下的系统停顿。
*   **持久化缺失**: 系统完全基于内存运行，重启即丢失数据，无法满足“备忘录/事件提醒”的基本业务需求。
*   **调度能力缺失**: 缺乏时间轮或 Cron 调度器，无法实现定时的事件触发。

### 1.2 亮点保留
*   **Iceoryx2 实践**: 虽然业务上不需要，但作为学习零拷贝通信的实验场，保留该模块作为“外部高性能IPC网关”。
*   **异步运行时**: `tokio` 基础打得不错，继续保持。

## 2. 目标架构 (Target Architecture)

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

## 3. 技术选型与依赖

| 模块 | 选型 | 说明 |
| :--- | :--- | :--- |
| **持久化** | **SQLx (SQLite)** | 轻量级、无服务器、SQL支持，适合结构化备忘录数据。 |
| **调度器** | **tokio-cron-scheduler** | 成熟的异步 Cron 调度库。 |
| **插件运行时** | **Extism** | 高性能 WASM 运行时，支持多语言编写插件，安全隔离。 |
| **IPC** | **Iceoryx2** | 保留作为学习目的，用于进程间高性能通信。 |
| **前端** | **Tauri** | (规划中) 基于 Web 技术栈的跨平台 GUI。 |

## 4. 重构步骤 (Step-by-Step)

### Phase 1: 基础设施搭建 (Current Scope)
1.  **引入依赖**: 添加 `sqlx`, `tokio-cron-scheduler`, `extism`, `uuid` 到 `Cargo.toml`。
2.  **重构目录**: 建立 `storage`, `scheduler`, `wasm` 模块。
3.  **实现核心**:
    *   **Storage**: 搭建 SQLite 连接池与基础 Schema (`memos` 表)。
    *   **Scheduler**: 集成 Cron 调度器并连接到消息总线。
    *   **WasmHost**: 实现基于 Extism 的 WASM 插件加载器 (`WasmPlugin`)。
4.  **架构调整**:
    *   **MessageManager Refactor**: 使用 `RwLock` 替代 `Mutex` 以减少并发瓶颈。
    *   **Plugin Trait Merge**: 将 `MessagePlugin` 的 `setup_messaging` 方法合并入 `Plugin` trait (默认实现为 no-op)，简化调用链。
    *   **CoreSystemPlugin**: 创建一个原生插件来封装 Storage 和 Scheduler 的初始化与消息处理逻辑。

### Phase 2: 业务功能填充 (Next Steps)
1.  完善备忘录的 CRUD 消息处理逻辑（当前仅实现了 Create）。
2.  编写第一个真实的 WASM 插件示例，并在系统中加载运行。
3.  实现 Iceoryx2 与内部消息总线的双向桥接（目前仅有接收器框架）。

### Phase 3: 交互层 (Future)
1.  集成 Tauri。
2.  实现前端 UI 与 Core 的通信。
