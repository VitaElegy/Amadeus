# Amadeus 架构文档

本文档详细说明 Amadeus 插件系统的架构设计、技术栈、通信机制和优化策略。

## 目录

- [系统架构](#系统架构)
- [通信机制](#通信机制)
- [技术栈](#技术栈)
- [代码优化](#代码优化)
- [文件结构](#文件结构)
- [项目路线图](#项目路线图-roadmap)

---

## 系统架构

### 整体架构图

Amadeus 采用**双模块设计**：核心（Message/Plugin Center）+ 插件（Plugins）。核心程序只负责“搭台”，即管理插件生命周期和消息分发，不再关心具体的业务或通信实现（如 IPC 通信），所有具体功能都下放给插件。

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
│  • register()      - 注册插件（按优先级排序）                    │
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
│ Iceoryx2 Disp │          │   Core System     │     │  Custom Plugin │
│  (Privileged) │          │     (Normal)      │     │    (Normal)    │
└───────────────┘          └───────────────────┘     └────────────────┘
        │                            │
        │ IPC                        │ Storage & Scheduler
        ▼                            ▼
   外部世界 (IPC)               SQLite / Cron
```

### 插件体系

插件分为两种类型（`PluginType`）：

1.  **特权插件 (Privileged)**：
    *   拥有更高优先级，并在普通插件之前加载。
    *   用于核心基础设施，如 IPC 分发器（`Iceoryx2DispatcherPlugin`）。
    *   为未来的权限管理和沙盒控制打下基础。
2.  **普通插件 (Normal)**：
    *   标准的业务逻辑插件（如 `CoreSystemPlugin`, `Code4renaPlugin`）。

### 消息系统架构

消息系统支持**广播（Public）**和**定向（Direct）**两种模式。

```
┌─────────────────────────────────────────────────────────┐
│                    外部系统/进程                          │
│              (前端、QQ bot、其他服务)                     │
└────────────────────┬────────────────────────────────────┘
                     │ Iceoryx2 (进程间通信)
                     │ 零拷贝 IPC
                     ▼
┌─────────────────────────────────────────────────────────┐
│          Iceoryx2DispatcherPlugin (特权插件)            │
│           (原 Dispatcher 模块，现已插件化)                │
└────────────────────┬────────────────────────────────────┘
                     │ 全局订阅 (Subscribe All) / 定向发送
                     ▼
┌─────────────────────────────────────────────────────────┐
│              MessageManager (消息管理器)                 │
│           (原 Message Center + Plugin Center)           │
└────────────────────┬────────────────────────────────────┘
                     │ Tokio Channels (进程内通信)
                     ▼
┌─────────────────────────────────────────────────────────┐
│          DistributionCenter (分发中心)                   │
│                                                         │
│  1. Broadcast Channels (Public Topic -> Subscribers)    │
│  2. Direct Channels (Plugin ID -> Specific Plugin)      │
│  3. Global Subscribers (All Public Messages)            │
└───────┬───────────────┬───────────────┬────────────────┘
        │               │               │
        ▼               ▼               ▼
    ┌───────┐      ┌───────┐      ┌───────┐
    │插件 A │      │插件 B │      │插件 C │
    └───────┘      └───────┘      └───────┘
```

---

## 通信机制

### 消息路由模式

1.  **Public 消息（广播）**
    *   **机制**：发布/订阅（Pub/Sub）。
    *   **流程**：插件 A 发送消息到 Topic `X` → 分发中心广播给所有订阅了 `X` 的插件。
    *   **用途**：系统事件、公共通知、解耦的业务逻辑。

2.  **Direct 消息（定向）**
    *   **机制**：点对点（P2P）。
    *   **流程**：插件 A 指定接收者 ID 发送消息 → 分发中心直接投递到目标插件的私有信道。
    *   **用途**：
        *   WASM 插件的安全沙盒通信。
        *   请求/响应模式。
        *   敏感数据传输（不希望被广播）。
    *   **启用**：插件需调用 `ctx.enable_direct_messaging()` 开启接收。

3.  **Global 订阅（全局监听）**
    *   **机制**：Wiretap。
    *   **用途**：分发器插件（如 `Iceoryx2Dispatcher`）需要监听所有内部公共消息并转发到外部 IPC。

---

## 文件结构

```
amadeus/src/
├── app.rs                <-- 应用入口配置
├── lib.rs                <-- 库定义
├── main.rs               <-- 二进制入口
├── plugin.rs             <-- 插件 trait 定义 (含 PluginType)
├── core/                 <-- [NEW] 核心架构层
│   └── messaging/        <-- 消息系统核心
│       ├── distribution_center.rs  <-- 路由逻辑 (Broadcast/Direct)
│       ├── message.rs              <-- 消息定义 (含 recipient 字段)
│       ├── message_context.rs      <-- 插件消息接口
│       └── message_manager.rs      <-- 消息循环
└── plugins/              <-- 插件层
    ├── core_system/      <-- 核心系统插件 (Storage/Scheduler)
    ├── iceoryx2_dispatcher/ <-- [NEW] IPC 分发器插件
    │   └── ipc/          <-- 私有 IPC 定义
    ├── wasm_plugin/      <-- [NEW] WASM 插件基础
    ├── code4rena.rs
    └── ...
```

---

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

#### Phase 1: 基础设施搭建 (已完成)
- [x] **架构重构**: 双模块设计，分发器插件化。
- [x] **消息系统**: 支持 Public/Direct 路由。
- [x] **IPC 集成**: Iceoryx2 作为特权插件集成。
- [x] **Core Plugins**: Storage 和 Scheduler 作为核心插件实现。

#### Phase 2: 业务功能与扩展能力 (进行中)
- [ ] **WASM 运行时**:
    - [ ] 集成 `Extism`。
    - [ ] 完善 `WasmPlugin` 加载器。
    - [ ] 利用 Direct 消息实现宿主与 WASM 的安全通信。
- [ ] **权限系统**:
    - [ ] 基于 `PluginType` 限制插件行为（如禁止普通插件调用 `subscribe_all`）。

#### Phase 3: 交互层 (未来规划)
- [ ] **Tauri 集成**: 构建跨平台 GUI。
- [ ] **API 层**: 实现前端 UI 与 Core 的通信接口。
