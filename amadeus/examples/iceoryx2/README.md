# Amadeus Iceoryx2 Python 测试

这个目录包含用于测试 Amadeus 系统 iceoryx2 分发器交互的 Python 脚本。这些脚本演示了如何使用 Python 与 Rust 的 iceoryx2 分发器进行零拷贝进程间通信。

## 环境准备

### 1. 安装 iceoryx2 Python 绑定

#### 自动安装（推荐）

使用提供的脚本自动安装：

```bash
cd examples/iceoryx2
./run_test.sh
```

脚本会自动：
- 检查 Python 3 和 maturin
- 创建虚拟环境
- 构建并安装 iceoryx2 Python 绑定
- 运行验证测试

#### 手动安装

如果需要手动安装，从项目根目录运行：

```bash
# 安装 maturin（如果还没有安装）
pip3 install maturin

# 创建虚拟环境
cd iceoryx2/iceoryx2-ffi/python
python3 -m venv venv

# 激活虚拟环境并构建
source venv/bin/activate
maturin develop --manifest-path Cargo.toml --target-dir ../../target/ff/python
```

#### 从 PyPI 安装（如果已发布）

```bash
pip install iceoryx2
```

### 2. 验证安装

```bash
python3 -c "import iceoryx2; print('iceoryx2 version:', iceoryx2.__version__)"
```

## 测试脚本说明

### 1. `amadeus_message_data.py`

定义了与 Rust `AmadeusMessageData` 结构体兼容的 Python 类。这个类实现了 iceoryx2 的零拷贝通信要求。

### 2. `publisher.py`

Python 发布者脚本，向 Amadeus iceoryx2 服务发送测试消息。

**运行方法：**
```bash
python3 publisher.py
```

### 3. `subscriber.py`

Python 订阅者脚本，从 Amadeus iceoryx2 服务接收消息。

**运行方法：**
```bash
python3 subscriber.py
```

### 4. `test_integration.py`

集成测试脚本，同时运行发布者和订阅者来测试双向通信。

**运行方法：**
```bash
python3 test_integration.py
```

### 5. `comprehensive_test.py` ⭐ **推荐**

完整的系统功能测试套件，覆盖所有Amadeus功能模块：
- ✅ 基础消息传递
- ✅ 插件系统集成
- ✅ 存储系统操作
- ✅ 调度系统任务
- ✅ 系统监控指标
- ✅ 告警通知系统
- ✅ 外部API集成
- ✅ 高并发处理

**运行方法：**
```bash
# 首先启动Rust测试服务
cargo run --example system_test

# 然后运行Python测试
python3 comprehensive_test.py
```

### 6. `interactive_test.py` ⭐ **推荐**

交互式的系统功能测试器，提供友好的菜单界面：
- 🎮 菜单驱动的操作界面
- 📤 发送各种类型的消息
- 📊 实时监控消息传递
- 📋 查看消息历史记录
- 🔧 自定义消息发送

**运行方法：**
```bash
# 首先启动Rust测试服务
cargo run --example system_test

# 然后运行交互式测试器
python3 interactive_test.py
```

### 7. `run_comprehensive_test.sh` ⭐ **一键测试**

自动化测试脚本，一键运行完整的系统测试：

**运行方法：**
```bash
# 完整测试流程（推荐）
./run_comprehensive_test.sh

# 仅构建项目
./run_comprehensive_test.sh --build-only

# 仅运行Python测试（假设Rust服务已运行）
./run_comprehensive_test.sh --python-only

# 运行集成测试模式
./run_comprehensive_test.sh --integration

# 设置测试时长60秒
./run_comprehensive_test.sh --duration 60

# 启用详细日志
./run_comprehensive_test.sh --verbose
```

## 🚀 使用方法

### 🚀 快速开始（推荐）

最简单的完整测试方式：

```bash
cd examples/iceoryx2

# 一键运行完整系统测试
./run_comprehensive_test.sh
```

这个脚本会自动：
1. ✅ 检查和安装所有依赖
2. 🔨 构建Rust项目
3. 🚀 启动测试服务
4. 🧪 运行全面的功能测试
5. 📊 生成详细的测试报告

### 🎮 交互式测试

如果您想手动控制测试过程：

```bash
# 终端1：启动Rust测试服务
cd amadeus
cargo run --example system_test

# 终端2：运行交互式测试器
cd examples/iceoryx2
python3 interactive_test.py
```

交互式测试器提供菜单驱动的界面，让您可以：
- 选择特定的功能模块进行测试
- 发送自定义消息
- 实时查看消息传递情况
- 查看详细的消息历史

### 🔧 传统测试方法

如果需要基本的iceoryx2通信测试：

```bash
# 启动基础的Rust服务
cd amadeus
cargo run --example messaging

# 运行基础的Python测试
cd examples/iceoryx2
python3 test_integration.py
```

### ⚡ 快速验证

仅验证iceoryx2通信是否正常：

```bash
cd examples/iceoryx2
python3 quick_test.py
```

## 📋 详细测试流程

### 完整系统测试

```bash
# 方式1：自动化脚本（推荐）
./run_comprehensive_test.sh

# 方式2：手动步骤
cd amadeus
cargo run --example system_test &
cd examples/iceoryx2
python3 comprehensive_test.py
```

### 基础通信测试

```bash
# 终端1：启动Rust服务
cd amadeus
cargo run --example messaging

# 终端2：运行Python集成测试
cd examples/iceoryx2
python3 test_integration.py
```

### 自定义测试

```bash
# 启动Rust服务
cd amadeus
cargo run --example system_test

# 使用交互式测试器进行自定义测试
cd examples/iceoryx2
python3 interactive_test.py
```

## 消息格式

Python 脚本发送的消息类型包括：

- `notification`: 通知消息
- `alert`: 警告消息
- `custom_event`: 自定义事件消息
- `python_test`: 集成测试消息

每条消息包含 JSON 格式的负载，包括时间戳、优先级等信息。

## 服务名称

所有脚本使用的服务名称是：`Amadeus/Message/Service`

这个名称与 Rust 代码中的 `service_names::AMADEUS_SERVICE` 常量匹配。

## 故障排除

### 常见问题

1. **ImportError: No module named 'iceoryx2'**
   - 确保正确安装了 iceoryx2 Python 绑定
   - 检查 Python 路径和虚拟环境
   - 运行 `./run_test.sh` 自动安装

2. **maturin: command not found**
   - 安装 maturin: `pip3 install maturin`
   - 或运行 `./run_test.sh` 自动安装

3. **服务连接失败**
   - 确保 Rust Amadeus 分发器正在运行
   - 检查服务名称是否匹配 (`Amadeus/Message/Service`)
   - 验证 iceoryx2 服务权限设置

4. **构建失败**
   - 确保 Rust 和 Cargo 已安装
   - 检查 iceoryx2 子模块是否正确克隆
   - 尝试删除 `target/` 目录后重新构建

5. **零拷贝传输失败**
   - 确保 AmadeusMessageData 结构体定义与 Rust 版本完全匹配
   - 检查内存对齐和填充
   - 验证消息大小不超过限制（4096字节 JSON，64字节类型名）

### 日志调试

设置环境变量启用详细日志：

```bash
export RUST_LOG=info
export ICEORYX2_LOG_LEVEL=info
```

## 技术细节

- **零拷贝通信**: 使用 iceoryx2 的共享内存机制，无需数据拷贝
- **类型安全**: 通过 ctypes.Structure 确保内存布局与 Rust 兼容
- **服务发现**: 自动发现和连接到 iceoryx2 服务
- **跨语言通信**: Python ↔ Rust 进程间通信

## 扩展

这些脚本可以作为基础，用于：

- 开发更复杂的消息类型
- 实现请求-响应模式通信
- 添加消息过滤和路由功能
- 集成到更大的测试套件中
