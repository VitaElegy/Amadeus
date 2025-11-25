#!/usr/bin/env python3
"""
Amadeus 交互式功能测试器

此脚本提供交互式的系统功能测试界面，用户可以：
- 选择要测试的功能模块
- 发送自定义消息
- 实时查看消息传递情况
- 监控系统状态

使用方法:
1. 启动Rust测试服务: cargo run --example system_test
2. 运行交互式测试: python3 interactive_test.py

作者: Amadeus Team
版本: 1.0.0
"""

import threading
import time
import json
from typing import Dict, List, Any, Optional
import iceoryx2 as iox2
from amadeus_message_data import AmadeusMessageData


class InteractiveTester:
    """交互式测试器"""

    def __init__(self, service_name: str = "Amadeus/Message/Service"):
        self.service_name = service_name
        self.node = None
        self.publisher = None
        self.subscriber = None
        self.message_history: List[Dict[str, Any]] = []
        self.monitoring_active = False
        self.setup_connection()

    def setup_connection(self):
        """建立iceoryx2连接"""
        print("🔗 连接到Amadeus系统...")

        try:
            iox2.set_log_level_from_env_or(iox2.LogLevel.Warn)
            self.node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

            service = (
                self.node.service_builder(iox2.ServiceName.new(self.service_name))
                .publish_subscribe(AmadeusMessageData)
                .open_or_create()
            )

            self.publisher = service.publisher_builder().create()
            self.subscriber = service.subscriber_builder().create()

            print("✅ 连接成功")
            return True
        except Exception as e:
            print(f"❌ 连接失败: {e}")
            return False

    def send_message(self, message_type: str, payload: dict, priority: int = 1) -> bool:
        """发送消息"""
        try:
            message_data = AmadeusMessageData.from_dict(message_type, payload, priority)
            sample = self.publisher.loan_uninit()
            sample = sample.write_payload(message_data)
            sample.send()

            # 记录到历史
            self.message_history.append({
                "direction": "sent",
                "message_type": message_type,
                "payload": payload,
                "priority": priority,
                "timestamp": time.time()
            })

            return True
        except Exception as e:
            print(f"❌ 发送消息失败: {e}")
            return False

    def receive_messages(self, timeout_seconds: float = 0.1) -> List[Dict[str, Any]]:
        """接收消息"""
        messages = []
        start_time = time.time()

        while time.time() - start_time < timeout_seconds:
            sample = self.subscriber.receive()
            if sample is None:
                break

            message_data = sample.payload
            message_dict = {
                "direction": "received",
                "message_type": message_data.get_message_type(),
                "payload": message_data.to_dict(),
                "priority": message_data.priority,
                "timestamp": message_data.timestamp / 1000  # 转换为秒
            }
            messages.append(message_dict)
            self.message_history.append(message_dict)

        return messages

    def start_monitoring(self):
        """启动后台监控"""
        if self.monitoring_active:
            print("📊 监控已在运行")
            return

        self.monitoring_active = True
        monitor_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
        monitor_thread.start()
        print("📊 后台监控已启动")

    def stop_monitoring(self):
        """停止后台监控"""
        self.monitoring_active = False
        print("📊 后台监控已停止")

    def _monitoring_loop(self):
        """监控循环"""
        print("📊 监控循环启动...")
        last_count = len(self.message_history)

        while self.monitoring_active:
            try:
                messages = self.receive_messages(0.5)
                if messages:
                    new_count = len(self.message_history) - last_count
                    if new_count > 0:
                        print(f"📥 收到 {new_count} 条新消息")
                        last_count = len(self.message_history)

                time.sleep(1)
            except Exception as e:
                print(f"监控错误: {e}")
                break

        print("📊 监控循环结束")

    def show_menu(self):
        """显示主菜单"""
        print("\n" + "="*50)
        print("🎮 Amadeus 交互式功能测试器")
        print("="*50)
        print("1. 📤 发送基础消息")
        print("2. 🔌 测试插件系统")
        print("3. 💾 测试存储操作")
        print("4. ⏰ 测试调度任务")
        print("5. 📊 测试监控系统")
        print("6. 🚨 测试告警系统")
        print("7. 🌐 测试外部集成")
        print("8. 📝 发送自定义消息")
        print("9. 📋 查看消息历史")
        print("0. 📊 切换监控模式")
        print("q. 🚪 退出")
        print("="*50)

    def handle_basic_messaging(self):
        """处理基础消息测试"""
        print("\n📤 基础消息测试")
        count = int(input("发送消息数量 (1-10): ") or "3")

        for i in range(count):
            success = self.send_message(
                "test.basic",
                {
                    "sequence": i + 1,
                    "message": f"交互式测试消息 #{i + 1}",
                    "source": "interactive_test",
                    "timestamp": int(time.time() * 1000)
                }
            )
            if success:
                print(f"✅ 发送消息 #{i + 1}")
            else:
                print(f"❌ 发送消息 #{i + 1} 失败")
                break

            time.sleep(0.2)

        print("基础消息测试完成")

    def handle_plugin_system(self):
        """处理插件系统测试"""
        print("\n🔌 插件系统测试")
        print("可选插件:")
        print("1. 核心系统插件")
        print("2. 消息示例插件")
        print("3. 代码安全插件")

        choice = input("选择插件 (1-3): ") or "1"

        plugin_map = {
            "1": ("plugin.core_system.status", "核心系统状态查询"),
            "2": ("plugin.message_example.trigger", "消息示例插件触发"),
            "3": ("plugin.code4rena.status", "代码安全插件状态")
        }

        if choice in plugin_map:
            msg_type, description = plugin_map[choice]
            success = self.send_message(
                msg_type,
                {
                    "action": "status",
                    "description": description,
                    "source": "interactive_test"
                }
            )
            if success:
                print(f"✅ 发送插件消息: {description}")
            else:
                print("❌ 发送插件消息失败")
        else:
            print("❌ 无效选择")

    def handle_storage_operations(self):
        """处理存储操作测试"""
        print("\n💾 存储操作测试")
        print("可选操作:")
        print("1. 保存数据")
        print("2. 读取数据")
        print("3. 删除数据")
        print("4. 列出所有数据")

        choice = input("选择操作 (1-4): ") or "1"

        operations = {
            "1": ("storage.save", {"key": "interactive_key", "value": "interactive_value"}),
            "2": ("storage.load", {"key": "interactive_key"}),
            "3": ("storage.delete", {"key": "interactive_key"}),
            "4": ("storage.list", {})
        }

        if choice in operations:
            msg_type, payload = operations[choice]
            success = self.send_message(msg_type, payload)
            if success:
                print(f"✅ 发送存储操作: {msg_type}")
            else:
                print("❌ 发送存储操作失败")
        else:
            print("❌ 无效选择")

    def handle_scheduler_operations(self):
        """处理调度操作测试"""
        print("\n⏰ 调度操作测试")
        print("可选操作:")
        print("1. 添加定时任务")
        print("2. 列出所有任务")
        print("3. 移除任务")

        choice = input("选择操作 (1-3): ") or "1"

        if choice == "1":
            job_id = input("任务ID: ") or "interactive_job"
            cron = input("Cron表达式 (默认每30秒): ") or "*/30 * * * * *"
            success = self.send_message("scheduler.add_job", {
                "job_id": job_id,
                "cron": cron,
                "message": {
                    "type": "scheduled.interactive",
                    "data": f"交互式定时任务: {job_id}"
                }
            })
        elif choice == "2":
            success = self.send_message("scheduler.list_jobs", {})
        elif choice == "3":
            job_id = input("要移除的任务ID: ") or "interactive_job"
            success = self.send_message("scheduler.remove_job", {"job_id": job_id})
        else:
            print("❌ 无效选择")
            return

        if success:
            print("✅ 发送调度操作成功")
        else:
            print("❌ 发送调度操作失败")

    def handle_monitoring_system(self):
        """处理监控系统测试"""
        print("\n📊 监控系统测试")
        print("可选监控:")
        print("1. 系统健康检查")
        print("2. 系统指标收集")
        print("3. 性能监控")

        choice = input("选择监控类型 (1-3): ") or "1"

        monitor_map = {
            "1": ("system.health_check", {"component": "all"}),
            "2": ("system.metrics", {"include": ["cpu", "memory", "disk"]}),
            "3": ("system.performance", {"duration": 60, "interval": 5})
        }

        if choice in monitor_map:
            msg_type, payload = monitor_map[choice]
            success = self.send_message(msg_type, payload)
            if success:
                print(f"✅ 发送监控请求: {msg_type}")
            else:
                print("❌ 发送监控请求失败")
        else:
            print("❌ 无效选择")

    def handle_alert_system(self):
        """处理告警系统测试"""
        print("\n🚨 告警系统测试")
        print("可选告警级别:")
        print("1. 信息 (Info)")
        print("2. 警告 (Warning)")
        print("3. 高优先级告警 (High)")
        print("4. 严重告警 (Critical)")

        choice = input("选择告警级别 (1-4): ") or "2"

        alert_map = {
            "1": ("notification.info", "信息通知", 0),
            "2": ("notification.warning", "警告通知", 1),
            "3": ("alert.high", "高优先级告警", 2),
            "4": ("alert.critical", "严重告警", 3)
        }

        if choice in alert_map:
            msg_type, description, priority = alert_map[choice]
            content = input("告警内容: ") or f"{description} - 交互式测试"

            success = self.send_message(
                msg_type,
                {
                    "description": description,
                    "content": content,
                    "source": "interactive_test",
                    "severity": priority,
                    "action_required": priority >= 2
                },
                priority
            )

            if success:
                print(f"✅ 发送告警: {description}")
            else:
                print("❌ 发送告警失败")
        else:
            print("❌ 无效选择")

    def handle_external_integration(self):
        """处理外部集成测试"""
        print("\n🌐 外部集成测试")
        print("可选集成:")
        print("1. API请求模拟")
        print("2. WebHook接收模拟")
        print("3. 外部服务调用")

        choice = input("选择集成类型 (1-3): ") or "1"

        if choice == "1":
            endpoint = input("API端点 (默认/health): ") or "/health"
            method = input("HTTP方法 (默认GET): ") or "GET"
            success = self.send_message("api.request", {
                "method": method,
                "endpoint": endpoint,
                "headers": {"User-Agent": "Amadeus-Interactive-Test/1.0"}
            })
        elif choice == "2":
            source = input("WebHook来源 (默认github): ") or "github"
            event = input("事件类型 (默认push): ") or "push"
            success = self.send_message("webhook.incoming", {
                "source": source,
                "event": event,
                "payload": {
                    "repository": "amadeus-project",
                    "ref": "refs/heads/main",
                    "action": "test"
                }
            })
        elif choice == "3":
            service = input("外部服务名: ") or "external_service"
            success = self.send_message("external.service_call", {
                "service": service,
                "action": "status",
                "parameters": {"test": True}
            })
        else:
            print("❌ 无效选择")
            return

        if success:
            print("✅ 发送外部集成消息成功")
        else:
            print("❌ 发送外部集成消息失败")

    def handle_custom_message(self):
        """处理自定义消息发送"""
        print("\n📝 发送自定义消息")

        message_type = input("消息类型: ").strip()
        if not message_type:
            print("❌ 消息类型不能为空")
            return

        print("输入JSON格式的消息负载 (例如: {\"key\": \"value\"})")
        payload_str = input("消息负载: ").strip()

        try:
            if payload_str:
                payload = json.loads(payload_str)
            else:
                payload = {}

            priority = int(input("优先级 (0-3, 默认1): ") or "1")
            priority = max(0, min(3, priority))

            success = self.send_message(message_type, payload, priority)
            if success:
                print(f"✅ 发送自定义消息: {message_type}")
            else:
                print("❌ 发送自定义消息失败")

        except json.JSONDecodeError as e:
            print(f"❌ JSON解析错误: {e}")
        except ValueError as e:
            print(f"❌ 优先级格式错误: {e}")

    def show_message_history(self):
        """显示消息历史"""
        print("\n📋 消息历史 (最近20条)")

        if not self.message_history:
            print("暂无消息历史")
            return

        # 显示最近20条消息
        recent_messages = self.message_history[-20:]

        for i, msg in enumerate(recent_messages):
            direction = "📤 发送" if msg["direction"] == "sent" else "📥 接收"
            msg_type = msg["message_type"]
            timestamp = time.strftime("%H:%M:%S", time.localtime(msg["timestamp"]))

            print(f"{i+1:2d}. {direction} {msg_type} [{timestamp}]")

            # 显示关键信息
            if "payload" in msg:
                payload = msg["payload"]
                if "content" in payload:
                    content = payload["content"]
                    if len(content) > 50:
                        content = content[:47] + "..."
                    print(f"      内容: {content}")
                elif "description" in payload:
                    print(f"      描述: {payload['description']}")

        print(f"\n总共 {len(self.message_history)} 条消息")

    def run(self):
        """运行交互式测试器"""
        print("🎮 欢迎使用 Amadeus 交互式功能测试器")
        print("请确保Rust测试服务已启动: cargo run --example system_test")
        print("输入 'q' 退出，输入 'help' 或 'h' 查看帮助")

        # 启动后台监控
        self.start_monitoring()

        try:
            while True:
                self.show_menu()
                choice = input("请选择功能 (0-9,q): ").strip().lower()

                if choice == 'q':
                    break
                elif choice == 'h' or choice == 'help':
                    self.show_help()
                elif choice == '0':
                    if self.monitoring_active:
                        self.stop_monitoring()
                    else:
                        self.start_monitoring()
                elif choice == '1':
                    self.handle_basic_messaging()
                elif choice == '2':
                    self.handle_plugin_system()
                elif choice == '3':
                    self.handle_storage_operations()
                elif choice == '4':
                    self.handle_scheduler_operations()
                elif choice == '5':
                    self.handle_monitoring_system()
                elif choice == '6':
                    self.handle_alert_system()
                elif choice == '7':
                    self.handle_external_integration()
                elif choice == '8':
                    self.handle_custom_message()
                elif choice == '9':
                    self.show_message_history()
                else:
                    print("❌ 无效选择，请重新输入")

                input("\n按Enter键继续...")

        except KeyboardInterrupt:
            print("\n🛑 收到中断信号，正在退出...")
        finally:
            self.stop_monitoring()
            print("👋 感谢使用 Amadeus 交互式功能测试器！")

    def show_help(self):
        """显示帮助信息"""
        print("\n" + "="*60)
        print("🎮 Amadeus 交互式功能测试器 - 帮助")
        print("="*60)
        print("此工具允许您与运行中的Amadeus系统进行交互式测试")
        print()
        print("功能说明:")
        print("1. 📤 发送基础消息    - 测试基本的消息传递功能")
        print("2. 🔌 测试插件系统    - 与各种插件进行交互")
        print("3. 💾 测试存储操作    - 测试数据存储和检索")
        print("4. ⏰ 测试调度任务    - 管理定时任务")
        print("5. 📊 测试监控系统    - 查看系统状态和指标")
        print("6. 🚨 测试告警系统    - 发送各种级别的告警")
        print("7. 🌐 测试外部集成    - 模拟外部API和Webhook")
        print("8. 📝 发送自定义消息   - 发送任意类型的消息")
        print("9. 📋 查看消息历史    - 查看发送和接收的消息")
        print("0. 📊 切换监控模式    - 开启/关闭后台消息监控")
        print()
        print("使用提示:")
        print("- 所有测试都会通过iceoryx2发送到Rust服务")
        print("- 消息历史会记录所有发送和接收的消息")
        print("- 后台监控会自动显示新接收的消息")
        print("- 可以使用JSON格式输入自定义消息负载")
        print()
        print("快捷键:")
        print("- q: 退出程序")
        print("- h 或 help: 显示此帮助")
        print("="*60)


def main():
    """主函数"""
    print("Amadeus 交互式功能测试器 v1.0.0")

    # 检查iceoryx2
    try:
        import iceoryx2
    except ImportError:
        print("❌ iceoryx2 未安装，请运行 ./run_test.sh 安装")
        return

    # 创建并运行测试器
    tester = InteractiveTester()

    if not tester.node:
        print("❌ 无法连接到Amadeus系统")
        return

    tester.run()


if __name__ == "__main__":
    main()
