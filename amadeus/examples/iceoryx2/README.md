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

## 🚀 使用方法

### 快速开始

最简单的开始方式：

```bash
cd examples/iceoryx2
./run_test.sh
```

脚本会引导你完成：
1. 自动安装 iceoryx2 Python 绑定
2. 运行验证测试
3. 选择测试模式（独立测试或完整系统测试）

### 验证测试

运行快速验证：

```bash
python3 quick_test.py
```

这会测试所有组件是否正常工作，无需启动完整的系统。

## 测试流程

### 基本测试

1. **启动 Rust Amadeus 分发器：**
   ```bash
   cd amadeus
   cargo run --example messaging
   ```

2. **在新终端运行 Python 发布者：**
   ```bash
   cd examples/iceoryx2
   python3 publisher.py
   ```

3. **在另一个终端运行 Python 订阅者：**
   ```bash
   cd examples/iceoryx2
   python3 subscriber.py
   ```

### 集成测试

直接运行集成测试：

```bash
cd examples/iceoryx2
python3 test_integration.py
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
