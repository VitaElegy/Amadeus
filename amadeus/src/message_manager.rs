use crate::dispatcher::{Dispatcher, DispatcherRegistry};
use crate::distribution_center::DistributionCenter;
use crate::message::Message;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

/// 消息管理器
/// 
/// 协调分发中心和分发器，处理消息的路由和分发
/// 使用 tokio 的异步通道实现消息传递
pub struct MessageManager {
    distribution_center: Arc<DistributionCenter>,
    dispatcher_registry: DispatcherRegistry,
    /// 消息接收通道（接收来自插件和分发器的消息）
    message_rx: Option<mpsc::Receiver<Message>>,
    message_tx: mpsc::Sender<Message>,
    /// 消息处理任务句柄
    message_task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MessageManager {
    /// 创建新的消息管理器
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        
        Self {
            distribution_center: Arc::new(DistributionCenter::new()),
            dispatcher_registry: DispatcherRegistry::new(),
            message_rx: Some(rx),
            message_tx: tx,
            message_task_handle: None,
        }
    }

    /// 获取分发中心的引用
    pub fn distribution_center(&self) -> &Arc<DistributionCenter> {
        &self.distribution_center
    }

    /// 获取分发器注册表的可变引用
    pub fn dispatcher_registry_mut(&mut self) -> &mut DispatcherRegistry {
        &mut self.dispatcher_registry
    }

    /// 获取消息发送通道（用于插件发送消息）
    pub fn message_tx(&self) -> mpsc::Sender<Message> {
        self.message_tx.clone()
    }

    /// 注册分发器
    pub fn register_dispatcher<D: Dispatcher + 'static>(&mut self, dispatcher: D) {
        self.dispatcher_registry.register(dispatcher);
    }


    /// 处理来自分发器的消息
    /// 
    /// 当分发器接收到外部消息时，调用此方法将消息路由给订阅的插件
    pub async fn handle_dispatcher_message(&self, message: Message) -> Result<()> {
        // 通过消息通道发送，由消息处理任务统一处理
        self.message_tx.send(message).await
            .map_err(|e| anyhow::anyhow!("发送消息失败: {}", e))?;
        Ok(())
    }

    /// 处理来自插件的消息
    /// 
    /// 当插件发送消息时，调用此方法将消息发送给分发器
    pub async fn handle_plugin_message(&self, message: Message) -> Result<()> {
        // 通过消息通道发送，由消息处理任务统一处理
        self.message_tx.send(message).await
            .map_err(|e| anyhow::anyhow!("发送消息失败: {}", e))?;
        Ok(())
    }

    /// 启动消息处理任务
    /// 
    /// 启动一个异步任务来处理消息路由
    pub fn start_message_loop(&mut self) {
        let distribution_center = Arc::clone(&self.distribution_center);
        // 使用 Arc 来共享分发器注册表
        let dispatcher_registry = std::sync::Arc::new(std::sync::Mutex::new(
            std::mem::replace(&mut self.dispatcher_registry, DispatcherRegistry::new())
        ));
        let mut message_rx = self.message_rx.take().expect("消息接收器已被使用");

        let handle = tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                // 分发消息给订阅的插件
                let subscriber_count = distribution_center.distribute(&message).await;
                
                if subscriber_count > 0 {
                    println!(
                        "[消息管理器] 消息分发: 类型={}, 订阅者={}",
                        message.message_type.as_str(),
                        subscriber_count
                    );
                }

                // 发送给订阅了该消息类型的分发器（让它们转发到外部）
                let registry = dispatcher_registry.lock().unwrap();
                for dispatcher in registry.dispatchers() {
                    if dispatcher.is_running() && dispatcher.is_subscribed_to(&message.message_type) {
                        if let Err(e) = dispatcher.send_message(&message) {
                            eprintln!(
                                "[消息管理器] 分发器 {} 发送消息失败: {}",
                                dispatcher.name(),
                                e
                            );
                        }
                    }
                }
            }
        });

        self.message_task_handle = Some(handle);
    }

    /// 停止消息处理任务
    pub async fn stop_message_loop(&mut self) {
        // 关闭发送通道，这会使得消息处理任务退出
        drop(self.message_tx.clone());
        
        if let Some(handle) = self.message_task_handle.take() {
            let _ = handle.await;
        }
    }

    /// 启动所有分发器
    pub fn start_dispatchers(&mut self) -> Result<()> {
        self.dispatcher_registry.start_all()?;
        Ok(())
    }

    /// 停止所有分发器
    pub fn stop_dispatchers(&mut self) -> Result<()> {
        self.dispatcher_registry.stop_all()?;
        Ok(())
    }
}

impl Default for MessageManager {
    fn default() -> Self {
        Self::new()
    }
}

