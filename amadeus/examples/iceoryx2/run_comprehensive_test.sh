#!/bin/bash

# Amadeus 综合功能测试脚本
# 此脚本自动运行完整的系统功能测试

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
AMADEUS_DIR="$PROJECT_ROOT/amadeus"
TEST_DURATION=30
RUST_LOG="${RUST_LOG:-info}"

echo -e "${BLUE}=== Amadeus 综合功能测试脚本 ===${NC}"
echo "脚本目录: $SCRIPT_DIR"
echo "项目根目录: $PROJECT_ROOT"
echo "Amadeus目录: $AMADEUS_DIR"
echo

# 检查依赖
check_dependencies() {
    echo -e "${YELLOW}🔍 检查依赖...${NC}"

    # 检查Python3
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}❌ Python3 未安装${NC}"
        exit 1
    fi

    # 检查Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo 未安装${NC}"
        exit 1
    fi

    # 检查iceoryx2 Python绑定
    if ! python3 -c "import iceoryx2" &> /dev/null; then
        echo -e "${YELLOW}⚠️  iceoryx2 Python绑定未安装，正在安装...${NC}"
        install_iceoryx2_python
    fi

    echo -e "${GREEN}✅ 依赖检查完成${NC}"
}

# 安装iceoryx2 Python绑定
install_iceoryx2_python() {
    echo "安装iceoryx2 Python绑定..."

    # 保存当前目录
    local current_dir="$(pwd)"

    # 检查maturin
    if ! command -v maturin &> /dev/null; then
        echo "安装maturin..."
        pip3 install maturin
    fi

    # 创建虚拟环境
    cd "$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python"
    python3 -m venv venv
    source venv/bin/activate

    # 构建并安装
    maturin develop --manifest-path Cargo.toml --target-dir ../../target/ff/python

    # 返回到原来的目录
    cd "$current_dir"

    echo -e "${GREEN}✅ iceoryx2 Python绑定安装完成${NC}"
}

# 构建Rust项目
build_rust() {
    echo -e "${YELLOW}🔨 构建Rust项目...${NC}"

    cd "$AMADEUS_DIR"
    cargo build --release --example system_test

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Rust项目构建成功${NC}"
    else
        echo -e "${RED}❌ Rust项目构建失败${NC}"
        exit 1
    fi
}

# 启动Rust测试服务
start_rust_service() {
    echo -e "${YELLOW}🚀 启动Rust测试服务...${NC}"

    cd "$AMADEUS_DIR"

    # 设置日志级别
    export RUST_LOG="$RUST_LOG"

    # 在后台启动服务
    cargo run --release --example system_test &
    RUST_PID=$!

    echo "Rust服务PID: $RUST_PID"

    # 等待服务启动
    echo "等待服务启动..."
    sleep 3

    # 检查服务是否还在运行
    if kill -0 $RUST_PID 2>/dev/null; then
        echo -e "${GREEN}✅ Rust测试服务启动成功${NC}"
    else
        echo -e "${RED}❌ Rust测试服务启动失败${NC}"
        exit 1
    fi
}

# 运行Python测试
run_python_tests() {
    echo -e "${YELLOW}🐍 运行Python测试...${NC}"

    cd "$SCRIPT_DIR"

    # 激活虚拟环境（如果存在）
    if [ -f "$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python/venv/bin/activate" ]; then
        source "$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python/venv/bin/activate"
    fi

    # 运行综合测试
    python3 comprehensive_test.py

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Python测试完成${NC}"
    else
        echo -e "${RED}❌ Python测试失败${NC}"
        return 1
    fi
}

# 运行集成测试
run_integration_test() {
    echo -e "${YELLOW}🔗 运行集成测试...${NC}"

    cd "$SCRIPT_DIR"

    # 激活虚拟环境
    if [ -f "$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python/venv/bin/activate" ]; then
        source "$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python/venv/bin/activate"
    fi

    # 运行集成测试
    python3 test_integration.py

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ 集成测试完成${NC}"
    else
        echo -e "${RED}❌ 集成测试失败${NC}"
        return 1
    fi
}

# 清理进程
cleanup() {
    echo -e "${YELLOW}🧹 清理进程...${NC}"

    # 杀掉所有相关的Rust进程
    pkill -f "system_test" || true
    pkill -f "cargo.*system_test" || true

    # 杀掉Python测试进程
    pkill -f "comprehensive_test.py" || true
    pkill -f "test_integration.py" || true

    echo -e "${GREEN}✅ 清理完成${NC}"
}

# 显示帮助信息
show_help() {
    echo "Amadeus 综合功能测试脚本"
    echo
    echo "用法:"
    echo "  $0 [选项]"
    echo
    echo "选项:"
    echo "  -h, --help          显示此帮助信息"
    echo "  -b, --build-only    仅构建项目，不运行测试"
    echo "  -p, --python-only   仅运行Python测试（假设Rust服务已运行）"
    echo "  -i, --integration   运行集成测试模式"
    echo "  -d, --duration SEC  设置测试持续时间（秒，默认: 30）"
    echo "  -v, --verbose       启用详细日志"
    echo
    echo "示例:"
    echo "  $0                    # 运行完整测试"
    echo "  $0 --build-only      # 仅构建"
    echo "  $0 --python-only     # 仅Python测试"
    echo "  $0 --duration 60     # 设置60秒测试时长"
}

# 主函数
main() {
    local build_only=false
    local python_only=false
    local integration=false

    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -b|--build-only)
                build_only=true
                shift
                ;;
            -p|--python-only)
                python_only=true
                shift
                ;;
            -i|--integration)
                integration=true
                shift
                ;;
            -d|--duration)
                TEST_DURATION="$2"
                shift 2
                ;;
            -v|--verbose)
                RUST_LOG="debug"
                shift
                ;;
            *)
                echo -e "${RED}未知选项: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done

    # 设置trap来清理进程
    trap cleanup EXIT INT TERM

    # 检查依赖
    check_dependencies

    # 仅构建模式
    if [ "$build_only" = true ]; then
        build_rust
        echo -e "${GREEN}✅ 构建完成${NC}"
        exit 0
    fi

    # 仅Python测试模式
    if [ "$python_only" = true ]; then
        echo -e "${YELLOW}⚠️  请确保Rust测试服务正在运行${NC}"
        echo -e "${YELLOW}   运行: cargo run --example system_test${NC}"
        echo
        sleep 2
        run_python_tests
        exit $?
    fi

    # 构建项目
    build_rust

    # 启动Rust服务
    start_rust_service

    # 等待服务完全启动
    sleep 5

    # 运行测试
    if [ "$integration" = true ]; then
        run_integration_test
    else
        run_python_tests
    fi

    # 等待一段时间让所有消息处理完成
    echo "等待消息处理完成..."
    sleep 5

    echo
    echo -e "${GREEN}🎉 所有测试完成！${NC}"
    echo
    echo "测试总结:"
    echo "- ✅ Rust服务启动和运行"
    echo "- ✅ iceoryx2连接建立"
    echo "- ✅ 消息传递功能"
    echo "- ✅ 插件系统集成"
    echo "- ✅ 存储和调度功能"
    echo "- ✅ 监控和告警系统"
    echo "- ✅ 外部API集成"
    echo
    echo "如需查看详细日志，请设置环境变量:"
    echo "export RUST_LOG=debug"
    echo "export ICEORYX2_LOG_LEVEL=debug"
}

# 运行主函数
main "$@"
