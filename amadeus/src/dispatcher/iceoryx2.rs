use crate::dispatcher::Dispatcher;
use crate::ipc::iceoryx2_types::{AmadeusMessageData, service_names};
use crate::message::{Message, MessagePriority, MessageType};
use anyhow::Result;
use iceoryx2::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

/// Iceoryx2 分发器实现
/// 
/// 使用 iceoryx2 进行零拷贝进程间通信
/// 负责将消息发布到 iceoryx2 服务，供其他进程订阅
/// 
/// 注意：由于 iceoryx2 的 Publisher 不是 Send + Sync，我们使用一个通道
/// 和一个后台任务来处理发布操作
pub struct Iceoryx2Dispatcher {
    name: String,
    node_name: String,
    service_name: String,
    running: Arc<AtomicBool>,
    /// 订阅的消息类型集合（空集合表示订阅所有消息类型）
    subscribed_types: HashSet<MessageType>,
    /// 消息发送通道（用于将消息发送到后台发布任务）
    message_tx: Option<mpsc::Sender<AmadeusMessageData>>,
    /// 后台发布任务句柄
    publisher_task_handle: Option<std::thread::JoinHandle<()>>,
}

impl Iceoryx2Dispatcher {
    /// 创建新的 Iceoryx2 分发器
    /// 
    /// 默认订阅所有消息类型，使用默认服务名称
    pub fn new(node_name: impl Into<String>) -> Self {
        Self::with_service(node_name, service_names::AMADEUS_SERVICE)
    }

    /// 使用指定的服务名称创建分发器
    pub fn with_service(node_name: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            name: "Iceoryx2Dispatcher".to_string(),
            node_name: node_name.into(),
            service_name: service_name.into(),
            running: Arc::new(AtomicBool::new(false)),
            subscribed_types: HashSet::new(), // 空集合表示订阅所有消息类型
            message_tx: None,
            publisher_task_handle: None,
        }
    }

    /// 设置分发器名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 订阅指定的消息类型
    /// 
    /// # 参数
    /// - `message_types`: 要订阅的消息类型列表
    /// 
    /// # 示例
    /// ```rust
    /// let dispatcher = Iceoryx2Dispatcher::new("node")
    ///     .subscribe_to(&["notification", "alert"]);
    /// ```
    pub fn subscribe_to(mut self, message_types: &[&str]) -> Self {
        self.subscribed_types = message_types
            .iter()
            .map(|s| MessageType::from(*s))
            .collect();
        self
    }

    /// 订阅单个消息类型
    pub fn subscribe_to_one(mut self, message_type: impl Into<MessageType>) -> Self {
        self.subscribed_types.insert(message_type.into());
        self
    }
}

impl Dispatcher for Iceoryx2Dispatcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let node_name = self.node_name.clone();
        let service_name = self.service_name.clone();
        let running = Arc::clone(&self.running);

        // 创建消息通道（使用 std::sync::mpsc）
        let (tx, rx) = mpsc::channel::<AmadeusMessageData>();
        self.message_tx = Some(tx);

        // 启动后台发布任务（使用 std::thread，因为 Publisher 不是 Send）
        let handle = std::thread::spawn(move || {
            // 在后台线程中创建 iceoryx2 节点和发布者
            let service_name_ref: &str = &service_name;
            
            let (node, mut publisher) = match NodeBuilder::new()
                .create::<ipc::Service>()
            {
                Ok(node) => {
                    let service_name_result: Result<_, _> = service_name_ref.try_into();
                    match service_name_result {
                        Ok(service_name) => {
                            match node
                                .service_builder(&service_name)
                                .publish_subscribe::<AmadeusMessageData>()
                                .open_or_create()
                            {
                                Ok(service) => {
                                    match service.publisher_builder().create() {
                                        Ok(pub_) => {
                                            println!("[Iceoryx2Dispatcher] 已连接到服务: {} (节点: {})", 
                                                     service_name, node_name);
                                            (Some(node), Some(pub_))
                                        }
                                        Err(e) => {
                                            eprintln!("[Iceoryx2Dispatcher] 创建发布者失败: {}", e);
                                            (Some(node), None)
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[Iceoryx2Dispatcher] 打开服务失败: {}", e);
                                    (Some(node), None)
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[Iceoryx2Dispatcher] 创建服务名称失败: {}", e);
                            (Some(node), None)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Iceoryx2Dispatcher] 创建节点失败: {}", e);
                    (None, None)
                }
            };

            // 发布循环（使用阻塞接收）
            while running.load(Ordering::Relaxed) {
                // 使用阻塞接收，但设置超时以避免无限等待
                match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(data) => {
                        if let Some(ref mut pub_) = publisher {
                            match pub_.loan_uninit() {
                                Ok(sample) => {
                                    let sample = sample.write_payload(data);
                                    if let Err(e) = sample.send() {
                                        eprintln!("[Iceoryx2Dispatcher] 发送消息失败: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[Iceoryx2Dispatcher] 获取样本失败: {}", e);
                                }
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // 超时，继续循环检查运行状态
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // 通道已关闭，退出循环
                        break;
                    }
                }
            }

            // 清理资源
            drop(publisher);
            drop(node);
            println!("[Iceoryx2Dispatcher] 发布任务已停止");
        });

        self.publisher_task_handle = Some(handle);
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // 停止后台任务
        self.running.store(false, Ordering::Relaxed);
        
        // 关闭通道
        drop(self.message_tx.take());
        
        // 等待任务完成
        if let Some(_handle) = self.publisher_task_handle.take() {
            // 注意：这里不能使用 await，因为 stop 是同步方法
            // 任务会在接收到 running=false 信号后自动退出
            // 在实际使用中，应该使用异步的 stop 方法
        }
        
        println!("[{}] 已断开 Iceoryx2 服务连接", self.name);
        
        Ok(())
    }

    fn send_message(&self, message: &Message) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("分发器未运行"));
        }

        // 获取消息发送通道
        let tx = self.message_tx.as_ref()
            .ok_or_else(|| anyhow::anyhow!("消息通道未初始化"))?;

        // 将 Message 转换为 AmadeusMessageData
        let json = message.to_json()?;
        let priority = match message.priority {
            MessagePriority::Low => 0,
            MessagePriority::Normal => 1,
            MessagePriority::High => 2,
            MessagePriority::Critical => 3,
        };

        let data = AmadeusMessageData::from_json(
            message.message_type.as_str(),
            &json,
            priority,
            message.timestamp,
        ).map_err(|e| anyhow::anyhow!("{}", e))?;

        // 发送到后台任务（阻塞发送，如果通道满了会阻塞）
        tx.send(data)
            .map_err(|e| anyhow::anyhow!("发送消息到发布任务失败: {}", e))?;

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn subscribed_message_types(&self) -> HashSet<MessageType> {
        self.subscribed_types.clone()
    }
}

