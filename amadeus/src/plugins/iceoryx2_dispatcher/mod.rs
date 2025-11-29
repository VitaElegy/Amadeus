pub mod ipc;

use crate::core::messaging::{
    Message, MessagePriority,
    DistributionCenter,
    MessageContext,
};
use crate::plugin::{Plugin, PluginMetadata, PluginType};
use self::ipc::iceoryx2_types::{AmadeusMessageData, service_names};
use self::ipc::prelude::{Service, NodeBuilder, ServiceName};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::pin::Pin;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{info, error};

pub struct Iceoryx2DispatcherPlugin {
    metadata: PluginMetadata,
    node_name: String,
    service_name: String,
    running: Arc<AtomicBool>,
    // Thread handle for receiving external messages
    receiver_thread: Option<std::thread::JoinHandle<()>>,
    // Thread handle for publishing messages to external
    publisher_thread: Option<std::thread::JoinHandle<()>>,
    // Channel to send messages to the publisher thread
    publisher_tx: Option<mpsc::Sender<AmadeusMessageData>>,
}

impl Iceoryx2DispatcherPlugin {
    pub fn new(node_name: impl Into<String>) -> Self {
        Self::with_service(node_name, service_names::AMADEUS_SERVICE)
    }

    pub fn with_service(node_name: impl Into<String>, service_name: impl Into<String>) -> Self {
        let metadata = PluginMetadata::new(
            "Iceoryx2Dispatcher",
            "Core dispatcher plugin using Iceoryx2 for IPC",
            "0.1.0",
        )
        .enabled_by_default(true)
        .with_property("role", "dispatcher");

        Self {
            metadata,
            node_name: node_name.into(),
            service_name: service_name.into(),
            running: Arc::new(AtomicBool::new(false)),
            receiver_thread: None,
            publisher_thread: None,
            publisher_tx: None,
        }
    }
}

impl Plugin for Iceoryx2DispatcherPlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Privileged
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        info!("[Iceoryx2Dispatcher] Initializing...");
        Ok(())
    }

    fn setup_messaging(
        &mut self,
        distribution_center: &DistributionCenter,
        message_tx: tokio_mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let dc = Arc::new(distribution_center.clone());
        let tx = message_tx.clone();
        
        // Clone for closure
        let node_name = self.node_name.clone();
        let service_name = self.service_name.clone();
        let running = self.running.clone();
        
        // We need a way to pass the publisher_tx back to the struct, but setup_messaging consumes &mut self
        // and returns a Future. We can't easily modify self inside the Future if the Future is static.
        // However, we can spawn threads here or prepare them.
        //
        // Actually, we can use a channel to extract the publisher_tx from the setup process if needed,
        // OR we can just start the threads here.
        // But `setup_messaging` is for the plugin to subscribe to INTERNAL messages.
        
        // Let's create the internal message context first.
        let ctx = Arc::new(MessageContext::new(dc, plugin_name.clone(), tx.clone()));
        let ctx_clone = ctx.clone();

        // Create a channel for the publisher thread
        let (pub_tx, pub_rx) = mpsc::channel::<AmadeusMessageData>();
        
        // We need to store pub_tx in self.
        // Since we are in &mut self, we can set it directly.
        self.publisher_tx = Some(pub_tx);
        self.running.store(true, Ordering::Relaxed);

        // Start Publisher Thread (Sends internal messages to External Iceoryx2)
        let pub_running = running.clone();
        let pub_node_name = node_name.clone();
        let pub_service_name = service_name.clone();
        
        self.publisher_thread = Some(std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let node = NodeBuilder::new().create::<self::ipc::prelude::ipc::Service>()
                    .map_err(|e| anyhow::anyhow!("Node creation failed: {}", e))?;
                let service_name_obj = ServiceName::new(&pub_service_name)?;
                let service = node.service_builder(&service_name_obj)
                    .publish_subscribe::<AmadeusMessageData>()
                    .open_or_create()?;
                let publisher = service.publisher_builder().create()?;
                
                info!("[Iceoryx2Dispatcher] Publisher connected to service: {}", pub_service_name);

                while pub_running.load(Ordering::Relaxed) {
                    match pub_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(data) => {
                            let sample = publisher.loan_uninit()?;
                            let sample = sample.write_payload(data);
                            sample.send()?;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                Ok(())
            })();
            if let Err(e) = result {
                error!("[Iceoryx2Dispatcher] Publisher thread error: {:?}", e);
            }
        }));

        // Start Receiver Thread (Receives External Iceoryx2 messages and forwards to Internal)
        let sub_running = running.clone();
        let sub_node_name = node_name.clone();
        let sub_service_name = service_name.clone();
        let internal_tx = tx.clone(); // Clone channel to send to MessageManager

        self.receiver_thread = Some(std::thread::spawn(move || {
             let result = (|| -> Result<()> {
                let node = NodeBuilder::new().create::<self::ipc::prelude::ipc::Service>()
                    .map_err(|e| anyhow::anyhow!("Node creation failed: {}", e))?;
                let service_name_obj = ServiceName::new(&sub_service_name)?;
                let service = node.service_builder(&service_name_obj)
                    .publish_subscribe::<AmadeusMessageData>()
                    .open_or_create()?;
                let subscriber = service.subscriber_builder().create()?;

                info!("[Iceoryx2Dispatcher] Subscriber connected to service: {}", sub_service_name);

                while sub_running.load(Ordering::Relaxed) {
                    match subscriber.receive()? {
                        Some(sample) => {
                            let data: &AmadeusMessageData = sample.payload();
                             // Deserialize and forward to internal system
                             if let Ok(json_str) = data.json_str() {
                                 if let Ok(msg) = Message::from_json(&json_str) {
                                     // Prevent echo loop: check source
                                     if let crate::core::messaging::message::MessageSource::Plugin(ref name) = msg.source {
                                         if name == "Iceoryx2Dispatcher" {
                                             continue;
                                         }
                                     }

                                     // Forward to internal system
                                     // Use blocking send here since we are in a thread
                                     let _ = internal_tx.blocking_send(msg);
                                 }
                             }
                        }
                        None => {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
                Ok(())
            })();
            if let Err(e) = result {
                error!("[Iceoryx2Dispatcher] Subscriber thread error: {:?}", e);
            }
        }));

        // Return the future that subscribes to all internal messages
        // We want to forward ALL internal messages to external (unless they are direct or explicitly internal only)
        // Currently we just subscribe to everything if we want to act as a bridge.
        // However, broadcast::Receiver receives everything sent to that topic.
        // We probably want to listen to ALL topics? 
        // DistributionCenter doesn't support "subscribe all" easily unless we iterate.
        // Or we can rely on specific topics.
        // For now, let's assume we want to forward "public" messages.
        // Since we can't easily subscribe to wildcard "*", we might need a change in DistributionCenter 
        // OR just have plugins send to specific topics that the dispatcher listens to.
        //
        // BUT, the old dispatcher had `subscribed_message_types`.
        // If we want to bridge everything, we might need a "monitor" feature or wildcard support.
        //
        // Let's assume we subscribe to some common topics or rely on plugins sending to topics we know.
        // Wait, the old dispatcher used `handle_dispatcher_message`.
        
        // Solution: Subscribe to specific topics or use a wildcard if implemented.
        // Since I haven't implemented wildcard in DistributionCenter (it uses HashMap),
        // I'll add a temporary hack: Subscribe to a list of known topics or just return ctx.
        //
        // Actually, the previous implementation of MessageManager.message_loop iterated over dispatchers and checked subscription.
        // Now, the dispatcher is a plugin.
        // It should subscribe to topics it wants to bridge.
        //
        // If we want a true bridge, we need `DistributionCenter` to support a "global subscriber" or "wire tap".
        //
        // Let's add a TODO to improve this. For now, let's just return the context and maybe subscribe to "system.*" if possible?
        // Or just let it be.
        
        // Wait, if I can't subscribe to all, how do I forward all messages?
        // The old `MessageManager` sent messages to dispatchers MANUALLY in the loop.
        // `registry.dispatchers()` loop.
        
        // To replicate this, I might need `DistributionCenter` to support "Global Subscribers" (like a wiretap).
        // I will add `subscribe_all` to `DistributionCenter` later if needed.
        
        // For now, I will implement a "wildcard" subscription in DistributionCenter?
        // No, that's complex.
        //
        // Let's look at `MessageManager` again.
        // `distribution_center.distribute(&message)` sends to subscribers.
        
        // I will modify `DistributionCenter` to support `subscribe_all`.
        
        let pub_tx_clone = self.publisher_tx.clone();

        Box::pin(async move {
            // Subscribe to all public messages to forward them externally
            let mut rx = ctx_clone.subscribe_all().await;
            
            // Spawn task to forward internal messages to external publisher thread
            if let Some(pub_tx) = pub_tx_clone {
                tokio::spawn(async move {
                    while let Ok(msg) = rx.recv().await {
                         // Prevent echo: do not forward messages that came from external iceoryx2
                         if let crate::core::messaging::message::MessageSource::External(ref src) = msg.source {
                             // If we had a way to identify if it came from THIS dispatcher node, we could filter better.
                             // But generally, we don't want to echo back what we just received from outside?
                             // Actually, if it's External, it came from some dispatcher.
                             // If it came from THIS dispatcher (plugin name), we shouldn't forward it back.
                             // But MessageSource::External stores a string source.
                             if src == "iceoryx2" {
                                 continue;
                             }
                         }
                         
                         // Prepare data for iceoryx2
                         if let Ok(json) = msg.to_json() {
                             let priority = match msg.priority {
                                MessagePriority::Low => 0,
                                MessagePriority::Normal => 1,
                                MessagePriority::High => 2,
                                MessagePriority::Critical => 3,
                            };
                            
                            if let Ok(data) = AmadeusMessageData::from_json(
                                msg.message_type.as_str(),
                                &json,
                                priority,
                                msg.timestamp,
                            ) {
                                // Send to publisher thread (blocking send ok here? No, async context)
                                // We need an async sender or use spawn_blocking.
                                // But our channel to publisher thread is std::sync::mpsc.
                                // We should use tokio::sync::mpsc or use spawn_blocking.
                                // The publisher thread uses std::sync::mpsc::Receiver.
                                // We can wrap the send in spawn_blocking.
                                let pub_tx_for_task = pub_tx.clone();
                                let _ = tokio::task::spawn_blocking(move || {
                                    let _ = pub_tx_for_task.send(data);
                                }).await;
                            }
                         }
                    }
                });
            }

            Ok(Some(ctx_clone))
        })
    }

    fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.publisher_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.receiver_thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

