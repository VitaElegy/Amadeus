#!/usr/bin/env python3
"""
Amadeus 综合功能测试套件

此脚本提供完整的系统功能测试，覆盖：
- 基础消息传递
- 插件系统集成
- 存储系统操作
- 调度系统任务
- 系统监控指标
- 告警通知系统
- 外部API集成
- 高并发处理

使用方法:
1. 启动Rust测试服务: cargo run --example system_test
2. 运行Python测试: python3 comprehensive_test.py

作者: Amadeus Team
版本: 1.0.0
"""

import threading
import time
import json
import statistics
from typing import Dict, List, Any
from dataclasses import dataclass
from enum import Enum
import iceoryx2 as iox2
from amadeus_message_data import AmadeusMessageData


class TestCategory(Enum):
    """测试类别枚举"""
    BASIC = "basic"
    PLUGIN = "plugin"
    STORAGE = "storage"
    SCHEDULER = "scheduler"
    MONITORING = "monitoring"
    ALERTS = "alerts"
    EXTERNAL = "external"
    PERFORMANCE = "performance"


@dataclass
class TestResult:
    """测试结果数据类"""
    category: TestCategory
    test_name: str
    success: bool
    message_count: int
    latency_ms: float
    error_message: str = ""
    start_time: float = 0
    end_time: float = 0

    @property
    def duration(self) -> float:
        return self.end_time - self.start_time


class AmadeusSystemTester:
    """Amadeus系统测试器"""

    def __init__(self, service_name: str = "Amadeus/Message/Service"):
        self.service_name = service_name
        self.node = None
        self.publisher = None
        self.subscriber = None
        self.test_results: List[TestResult] = []
        self.received_messages: List[Dict[str, Any]] = []
        self.setup_connection()

    def setup_connection(self):
        """建立iceoryx2连接"""
        print("🔗 连接到Amadeus系统...")

        # 配置iceoryx2
        iox2.set_log_level_from_env_or(iox2.LogLevel.Warn)
        self.node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

        # 创建发布订阅服务
        service = (
            self.node.service_builder(iox2.ServiceName.new(self.service_name))
            .publish_subscribe(AmadeusMessageData)
            .open_or_create()
        )

        # 创建发布者和订阅者
        self.publisher = service.publisher_builder().create()
        self.subscriber = service.subscriber_builder().create()

        print("✅ 连接成功")

    def send_message(self, message_type: str, payload: dict, priority: int = 1) -> bool:
        """发送消息"""
        try:
            message_data = AmadeusMessageData.from_dict(message_type, payload, priority)

            # 使用零拷贝模式发送
            sample = self.publisher.loan_uninit()
            sample = sample.write_payload(message_data)
            sample.send()

            return True
        except Exception as e:
            print(f"❌ 发送消息失败: {e}")
            return False

    def receive_messages(self, timeout_seconds: float = 1.0) -> List[Dict[str, Any]]:
        """接收消息"""
        messages = []
        start_time = time.time()

        while time.time() - start_time < timeout_seconds:
            sample = self.subscriber.receive()
            if sample is None:
                break

            message_data = sample.payload
            message_dict = {
                "message_type": message_data.get_message_type(),
                "payload": message_data.to_dict(),
                "priority": message_data.priority,
                "timestamp": message_data.timestamp
            }
            messages.append(message_dict)

        return messages

    def run_test(self, category: TestCategory, test_name: str, test_func) -> TestResult:
        """运行单个测试"""
        print(f"\n🧪 运行测试: {category.value}.{test_name}")

        result = TestResult(
            category=category,
            test_name=test_name,
            success=False,
            message_count=0,
            latency_ms=0.0,
            start_time=time.time()
        )

        try:
            # 清空之前接收的消息
            self.received_messages.clear()

            # 运行测试函数
            test_func()

            # 等待消息处理
            time.sleep(0.5)
            received = self.receive_messages(2.0)

            result.message_count = len(received)
            result.success = True
            result.end_time = time.time()
            result.latency_ms = (result.end_time - result.start_time) * 1000

            print(f"✅ 测试通过 - 收到 {result.message_count} 条消息，耗时 {result.latency_ms:.2f}ms")

        except Exception as e:
            result.error_message = str(e)
            result.end_time = time.time()
            print(f"❌ 测试失败: {e}")

        self.test_results.append(result)
        return result

    def test_basic_messaging(self):
        """测试基础消息传递"""
        print("  📤 发送基础消息...")

        for i in range(5):
            success = self.send_message(
                "test.basic",
                {
                    "test_id": i + 1,
                    "message": f"基础测试消息 #{i + 1}",
                    "source": "python_test",
                    "timestamp": int(time.time() * 1000)
                }
            )
            if not success:
                raise Exception(f"发送基础消息 {i + 1} 失败")
            time.sleep(0.1)

    def test_plugin_system(self):
        """测试插件系统"""
        print("  🔌 测试插件消息...")

        plugin_tests = [
            ("plugin.core_system.status", "核心系统状态查询"),
            ("plugin.message_example.trigger", "消息示例插件触发"),
            ("plugin.code4rena.status", "代码安全插件状态")
        ]

        for msg_type, description in plugin_tests:
            success = self.send_message(
                msg_type,
                {
                    "action": "status",
                    "description": description,
                    "source": "python_test"
                }
            )
            if not success:
                raise Exception(f"发送插件消息失败: {msg_type}")

    def test_storage_operations(self):
        """测试存储系统"""
        print("  💾 测试存储操作...")

        # 保存数据
        self.send_message("storage.save", {
            "key": "test_key",
            "value": {"data": "test_value", "timestamp": int(time.time() * 1000)},
            "ttl": 3600
        })

        time.sleep(0.2)

        # 读取数据
        self.send_message("storage.load", {
            "key": "test_key"
        })

        time.sleep(0.2)

        # 删除数据
        self.send_message("storage.delete", {
            "key": "test_key"
        })

    def test_scheduler_operations(self):
        """测试调度系统"""
        print("  ⏰ 测试调度任务...")

        # 添加定时任务
        self.send_message("scheduler.add_job", {
            "job_id": "python_test_job",
            "cron": "*/10 * * * * *",  # 每10秒执行
            "message": {
                "type": "scheduled.python_test",
                "data": "Python测试定时任务"
            }
        })

        time.sleep(0.2)

        # 列出任务
        self.send_message("scheduler.list_jobs", {})

        time.sleep(0.2)

        # 移除任务
        self.send_message("scheduler.remove_job", {
            "job_id": "python_test_job"
        })

    def test_monitoring_system(self):
        """测试监控系统"""
        print("  📊 测试系统监控...")

        monitoring_tests = [
            ("system.health_check", {"component": "all"}),
            ("system.metrics", {"include": ["cpu", "memory", "disk"]}),
            ("system.performance", {"duration": 60})
        ]

        for msg_type, payload in monitoring_tests:
            success = self.send_message(msg_type, payload)
            if not success:
                raise Exception(f"发送监控消息失败: {msg_type}")
            time.sleep(0.1)

    def test_alert_system(self):
        """测试告警系统"""
        print("  🚨 测试告警系统...")

        alerts = [
            ("notification.info", "信息", 0),
            ("notification.warning", "警告", 1),
            ("alert.high", "高优先级告警", 2),
            ("alert.critical", "严重告警", 3)
        ]

        for msg_type, description, priority in alerts:
            success = self.send_message(
                msg_type,
                {
                    "description": description,
                    "source": "python_test",
                    "severity": priority,
                    "action_required": priority >= 2
                },
                priority
            )
            if not success:
                raise Exception(f"发送告警消息失败: {msg_type}")

    def test_external_integration(self):
        """测试外部系统集成"""
        print("  🌐 测试外部集成...")

        # 模拟API调用
        self.send_message("api.request", {
            "method": "GET",
            "endpoint": "/api/test",
            "headers": {"Authorization": "Bearer test_token"}
        })

        time.sleep(0.2)

        # 模拟Webhook接收
        self.send_message("webhook.incoming", {
            "source": "external_service",
            "event": "data_update",
            "payload": {"key": "value"}
        })

    def test_performance(self):
        """测试性能和并发"""
        print("  ⚡ 测试性能...")

        start_time = time.time()
        message_count = 100

        # 发送批量消息
        for i in range(message_count):
            success = self.send_message(
                "test.performance",
                {
                    "sequence": i,
                    "data": f"性能测试消息 {i}",
                    "batch_id": "perf_test_001"
                }
            )
            if not success:
                raise Exception(f"发送性能测试消息失败: {i}")

        end_time = time.time()
        total_time = end_time - start_time
        msg_per_sec = message_count / total_time

        print(".2f"
    def run_all_tests(self):
        """运行所有测试"""
        print("🚀 开始Amadeus综合功能测试")
        print("=" * 50)

        tests = [
            (TestCategory.BASIC, "basic_messaging", self.test_basic_messaging),
            (TestCategory.PLUGIN, "plugin_system", self.test_plugin_system),
            (TestCategory.STORAGE, "storage_operations", self.test_storage_operations),
            (TestCategory.SCHEDULER, "scheduler_operations", self.test_scheduler_operations),
            (TestCategory.MONITORING, "monitoring_system", self.test_monitoring_system),
            (TestCategory.ALERTS, "alert_system", self.test_alert_system),
            (TestCategory.EXTERNAL, "external_integration", self.test_external_integration),
            (TestCategory.PERFORMANCE, "performance", self.test_performance),
        ]

        for category, test_name, test_func in tests:
            self.run_test(category, test_name, test_func)

        self.generate_report()

    def generate_report(self):
        """生成测试报告"""
        print("\n" + "=" * 50)
        print("📊 测试报告")
        print("=" * 50)

        # 按类别分组统计
        category_stats = {}
        total_tests = len(self.test_results)
        passed_tests = 0

        for result in self.test_results:
            if result.success:
                passed_tests += 1

            if result.category not in category_stats:
                category_stats[result.category] = {"total": 0, "passed": 0, "latencies": []}

            category_stats[result.category]["total"] += 1
            if result.success:
                category_stats[result.category]["passed"] += 1
            category_stats[result.category]["latencies"].append(result.latency_ms)

        # 输出总体结果
        success_rate = (passed_tests / total_tests) * 100
        print(f"总体结果: {passed_tests}/{total_tests} 通过 ({success_rate:.1f}%)")

        # 输出各类别结果
        print("\n各功能模块测试结果:")
        for category, stats in category_stats.items():
            passed = stats["passed"]
            total = stats["total"]
            rate = (passed / total) * 100
            avg_latency = statistics.mean(stats["latencies"]) if stats["latencies"] else 0
            print(".1f"
        # 性能统计
        all_latencies = [r.latency_ms for r in self.test_results if r.success]
        if all_latencies:
            print("
性能统计:"            print(".2f"            print(".2f"            print(".2f"
        # 详细失败信息
        failed_tests = [r for r in self.test_results if not r.success]
        if failed_tests:
            print("
❌ 失败的测试:"            for result in failed_tests:
                print(f"  - {result.category.value}.{result.test_name}: {result.error_message}")

        print("\n✅ 测试完成!")


def main():
    """主函数"""
    print("Amadeus 综合功能测试套件 v1.0.0")
    print("请确保Rust测试服务已启动: cargo run --example system_test")

    # 检查iceoryx2是否可用
    try:
        import iceoryx2
    except ImportError:
        print("❌ iceoryx2 未安装，请运行 ./run_test.sh 安装")
        return

    # 创建测试器并运行测试
    tester = AmadeusSystemTester()

    try:
        tester.run_all_tests()
    except KeyboardInterrupt:
        print("\n🛑 测试被用户中断")
    except Exception as e:
        print(f"\n❌ 测试过程中发生错误: {e}")
    finally:
        print("测试套件结束")


if __name__ == "__main__":
    main()
