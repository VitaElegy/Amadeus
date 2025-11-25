// iceoryx2 消息接收器
// 负责从 iceoryx2 订阅消息并转发到分发中心

use crate::message::Message;
use crate::ipc::iceoryx2_types::{AmadeusMessageData, service_names};
use anyhow::Result;
use iceoryx2::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// iceoryx2 消息接收器
/// 
/// 在后台运行，订阅 iceoryx2 服务并接收消息，
/// 然后将消息转发到分发中心
pub struct Iceoryx2Receiver {
    /// 节点名称
    node_name: String,
    /// 服务名称
    service_name: String,
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 消息发送通道（用于将接收到的消息发送到消息管理器）
    message_tx: Option<mpsc::Sender<Message>>,
    /// 接收任务句柄
    receiver_task_handle: Option<std::thread::JoinHandle<()>>,
}

impl Iceoryx2Receiver {
    /// 创建新的接收器
    /// 
    /// # 参数
    /// - `node_name`: iceoryx2 节点名称
    /// - `service_name`: iceoryx2 服务名称
    pub fn new(node_name: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
            service_name: service_name.into(),
            running: Arc::new(AtomicBool::new(false)),
            message_tx: None,
            receiver_task_handle: None,
        }
    }

    /// 使用默认服务名称创建接收器
    pub fn with_default_service(node_name: impl Into<String>) -> Self {
        Self::new(node_name, service_names::AMADEUS_SERVICE)
    }

    /// 设置消息发送通道
    pub fn with_message_tx(mut self, tx: mpsc::Sender<Message>) -> Self {
        self.message_tx = Some(tx);
        self
    }

    /// 启动接收器
    /// 
    /// 在后台启动一个异步任务来接收 iceoryx2 消息
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let message_tx = self.message_tx.take()
            .ok_or_else(|| anyhow::anyhow!("消息发送通道未设置"))?;
        
        let node_name = self.node_name.clone();
        let service_name = self.service_name.clone();
        let running = Arc::clone(&self.running);

        // 启动接收任务（使用 std::thread，因为 Subscriber 不是 Send）
        let handle = std::thread::spawn(move || {
            if let Err(e) = Self::receive_loop(node_name, service_name, message_tx, running.clone()) {
                tracing::error!("[Iceoryx2Receiver] 接收循环错误: {}", e);
            }
            running.store(false, Ordering::Relaxed);
        });

        self.receiver_task_handle = Some(handle);
        self.running.store(true, Ordering::Relaxed);
        
        tracing::info!("[Iceoryx2Receiver] 已启动，服务: {}", self.service_name);
        Ok(())
    }

    /// 停止接收器
    pub async fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.receiver_task_handle.take() {
            let _ = handle.join();
        }
        
        tracing::info!("[Iceoryx2Receiver] 已停止");
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// 接收循环（内部方法）
    fn receive_loop(
        _node_name: String,
        service_name: String,
        message_tx: mpsc::Sender<Message>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        // 创建 iceoryx2 节点
        let service_name_ref: &str = &service_name;
        
        let node = NodeBuilder::new()
            .create::<ipc::Service>()?;

        // 打开或创建服务
        let service = node
            .service_builder(&service_name_ref.try_into()?)
            .publish_subscribe::<AmadeusMessageData>()
            .open_or_create()?;

        // 创建订阅者
        let subscriber = service.subscriber_builder().create()?;

        tracing::info!("[Iceoryx2Receiver] 已连接到服务: {}", service_name);

        // 接收循环
        while running.load(Ordering::Relaxed) {
            // 尝试接收消息
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    // 将 AmadeusMessageData 转换为 Message
                    match Self::convert_to_message(&*sample) {
                        Ok(message) => {
                            // 发送到消息管理器（阻塞发送）
                            if let Err(e) = message_tx.blocking_send(message) {
                                tracing::error!("[Iceoryx2Receiver] 发送消息失败: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("[Iceoryx2Receiver] 消息转换失败: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    // 没有消息，短暂等待后继续循环
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    tracing::error!("[Iceoryx2Receiver] 接收消息错误: {}", e);
                    // 短暂等待后重试
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
        }

        Ok(())
    }

    /// 将 AmadeusMessageData 转换为 Message
    fn convert_to_message(data: &AmadeusMessageData) -> Result<Message> {
        let message_type_str = data.message_type_str()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let json_str = data.json_str()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // 解析 JSON
        let payload: serde_json::Value = serde_json::from_str(&json_str)?;
        
        // 创建 Message
        let mut message = Message::new(message_type_str, payload);
        message.timestamp = data.timestamp;
        
        // 设置优先级
        use crate::message::MessagePriority;
        message.priority = match data.priority {
            0 => MessagePriority::Low,
            1 => MessagePriority::Normal,
            2 => MessagePriority::High,
            3 => MessagePriority::Critical,
            _ => MessagePriority::Normal,
        };
        
        // 设置来源
        message.source = crate::message::MessageSource::External("iceoryx2".to_string());
        
        Ok(message)
    }
}

