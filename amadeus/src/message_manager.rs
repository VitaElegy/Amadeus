use crate::dispatcher::{Dispatcher, DispatcherRegistry};
use crate::distribution_center::DistributionCenter;
use crate::ipc::Iceoryx2Receiver;
use crate::message::Message;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// 消息管理器
/// 
/// 协调分发中心和分发器，处理消息的路由和分发
/// 使用 tokio 的异步通道实现消息传递
pub struct MessageManager {
    distribution_center: Arc<DistributionCenter>,
    dispatcher_registry: Arc<RwLock<DispatcherRegistry>>,
    /// 消息接收通道（接收来自插件和分发器的消息）
    message_rx: Option<mpsc::Receiver<Message>>,
    message_tx: mpsc::Sender<Message>,
    /// 消息处理任务句柄
    message_task_handle: Option<tokio::task::JoinHandle<()>>,
    /// iceoryx2 接收器列表（用于从外部接收消息）
    iceoryx2_receivers: Vec<Iceoryx2Receiver>,
}

impl MessageManager {
    /// 创建新的消息管理器
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        
        Self {
            distribution_center: Arc::new(DistributionCenter::new()),
            dispatcher_registry: Arc::new(RwLock::new(DispatcherRegistry::new())),
            message_rx: Some(rx),
            message_tx: tx,
            message_task_handle: None,
            iceoryx2_receivers: Vec::new(),
        }
    }

    /// 获取分发中心的引用
    pub fn distribution_center(&self) -> &Arc<DistributionCenter> {
        &self.distribution_center
    }

    /// 获取分发器注册表的可变引用 (通过 RwLockWriteGuard 暴露，或者直接提供方法)
    /// 这里我们提供一个异步方法来获取锁，或者直接提供注册方法
    pub async fn register_dispatcher<D: Dispatcher + 'static>(&self, dispatcher: D) {
        let mut registry = self.dispatcher_registry.write().await;
        registry.register(dispatcher);
    }

    /// 获取消息发送通道（用于插件发送消息）
    pub fn message_tx(&self) -> mpsc::Sender<Message> {
        self.message_tx.clone()
    }

    /// 注册 iceoryx2 接收器
    pub fn register_iceoryx2_receiver(&mut self, mut receiver: Iceoryx2Receiver) -> Result<()> {
        // 设置消息发送通道
        receiver = receiver.with_message_tx(self.message_tx.clone());
        self.iceoryx2_receivers.push(receiver);
        Ok(())
    }

    /// 启动所有 iceoryx2 接收器
    pub async fn start_iceoryx2_receivers(&mut self) -> Result<()> {
        for receiver in &mut self.iceoryx2_receivers {
            receiver.start()?;
        }
        Ok(())
    }

    /// 停止所有 iceoryx2 接收器
    pub async fn stop_iceoryx2_receivers(&mut self) {
        for receiver in &mut self.iceoryx2_receivers {
            receiver.stop().await;
        }
    }

    /// 处理来自分发器的消息
    pub async fn handle_dispatcher_message(&self, message: Message) -> Result<()> {
        self.message_tx.send(message).await
            .map_err(|e| anyhow::anyhow!("发送消息失败: {}", e))?;
        Ok(())
    }

    /// 处理来自插件的消息
    pub async fn handle_plugin_message(&self, message: Message) -> Result<()> {
        self.message_tx.send(message).await
            .map_err(|e| anyhow::anyhow!("发送消息失败: {}", e))?;
        Ok(())
    }

    /// 启动消息处理任务
    pub fn start_message_loop(&mut self) {
        let distribution_center = Arc::clone(&self.distribution_center);
        let dispatcher_registry = Arc::clone(&self.dispatcher_registry);
        let mut message_rx = self.message_rx.take().expect("消息接收器已被使用");

        let handle = tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                // 分发消息给订阅的插件
                let subscriber_count = distribution_center.distribute(&message).await;
                
                if subscriber_count > 0 {
                    // Optional logging could be reduced to debug level to avoid spam
                    // println!(...); 
                }

                // 发送给订阅了该消息类型的分发器（让它们转发到外部）
                // 使用 read lock，允许多个读者，且不阻塞写者（如果有）
                let registry = dispatcher_registry.read().await;
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
        // 关闭发送通道
        drop(self.message_tx.clone()); // Hacky way to ensure dropped? No, self.message_tx holds one.
        // We can't close the channel easily if we hold a sender.
        // We should just abort the handle or rely on Drop.
        // Or better: `self.message_rx.close()` if we had access to it, but we moved it.
        
        if let Some(handle) = self.message_task_handle.take() {
            handle.abort(); // Force stop
            let _ = handle.await;
        }
    }

    /// 启动所有分发器和接收器
    pub async fn start_dispatchers(&mut self) -> Result<()> {
        // 启动分发器
        {
            let mut registry = self.dispatcher_registry.write().await;
            registry.start_all()?;
        }
        // 启动 iceoryx2 接收器
        self.start_iceoryx2_receivers().await?;
        Ok(())
    }

    /// 停止所有分发器和接收器
    pub async fn stop_dispatchers(&mut self) -> Result<()> {
        // 停止 iceoryx2 接收器
        self.stop_iceoryx2_receivers().await;
        // 停止分发器
        {
            let mut registry = self.dispatcher_registry.write().await;
            registry.stop_all()?;
        }
        Ok(())
    }
}

impl Default for MessageManager {
    fn default() -> Self {
        Self::new()
    }
}
