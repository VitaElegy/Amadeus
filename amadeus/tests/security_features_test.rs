use amadeus::core::messaging::message_manager::MessageManager;
use amadeus::core::messaging::message::Message;
use amadeus::core::messaging::distribution_center::DistributionCenter;
use amadeus::core::messaging::message_context::MessageContext;
use amadeus::plugin::{Plugin, PluginMetadata};
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::mpsc;
use std::time::Duration;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::thread_rng;

// Mock Plugin
struct MockPlugin {
    metadata: PluginMetadata,
}

impl MockPlugin {
    fn new(name: &str) -> Self {
        Self {
            metadata: PluginMetadata::new(name, "Mock Plugin", "0.1.0"),
        }
    }
}

impl Plugin for MockPlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn setup_messaging(
        &mut self,
        dc: &DistributionCenter,
        tx: mpsc::Sender<Message>,
    ) -> Pin<Box<dyn std::future::Future<Output = anyhow::Result<Option<Arc<MessageContext>>>> + Send>> {
        let plugin_name = self.metadata.name.clone();
        let plugin_uid = self.metadata.uid.clone();
        let dc = Arc::new(dc.clone());

        Box::pin(async move {
            let ctx = Arc::new(MessageContext::new(dc, plugin_name, plugin_uid, tx));
            Ok(Some(ctx))
        })
    }
}

#[tokio::test]
async fn test_plugin_uid_and_direct_messaging() -> anyhow::Result<()> {
    // 1. Setup
    let mut message_manager = MessageManager::new();
    let dc = message_manager.distribution_center();
    let tx = message_manager.message_tx();

    // 2. Create Plugins
    let mut plugin_a = MockPlugin::new("plugin_a");
    let mut plugin_b = MockPlugin::new("plugin_b");
    let mut plugin_c = MockPlugin::new("plugin_c");

    // 3. Get UIDs
    let uid_a = plugin_a.metadata().uid.clone();
    let uid_b = plugin_b.metadata().uid.clone();
    let uid_c = plugin_c.metadata().uid.clone();

    assert_ne!(uid_a, uid_b);
    assert_ne!(uid_b, uid_c);

    println!("UID A: {}", uid_a);
    println!("UID B: {}", uid_b);

    // 4. Setup Messaging Contexts
    let ctx_a = plugin_a.setup_messaging(dc, tx.clone()).await?.unwrap();
    let ctx_b = plugin_b.setup_messaging(dc, tx.clone()).await?.unwrap();
    let ctx_c = plugin_c.setup_messaging(dc, tx.clone()).await?.unwrap();

    // 5. Start Message Loop
    message_manager.start_message_loop();

    // 6. Enable Direct Messaging
    let mut rx_b = ctx_b.enable_direct_messaging().await;
    let mut rx_c = ctx_c.enable_direct_messaging().await;

    // 7. Plugin A sends private message to Plugin B using UID
    let secret_msg = Message::new_direct(
        uid_b.clone(), // Target UID
        "secret.chat",
        serde_json::json!({"content": "Hello B, this is secret!"})
    );
    ctx_a.send(secret_msg).await?;

    // 8. Verify
    tokio::select! {
        Some(msg) = rx_b.recv() => {
            assert_eq!(msg.message_type.as_str(), "secret.chat");
            println!("Plugin B received secret message: {:?}", msg.payload);
        }
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            panic!("Plugin B did not receive the message");
        }
    }

    // Verify Plugin C did NOT receive it
    tokio::select! {
        Some(_) = rx_c.recv() => {
            panic!("Plugin C received a message it shouldn't have!");
        }
        _ = tokio::time::sleep(Duration::from_millis(100)) => {
            println!("Verified Plugin C did not receive the message");
        }
    }

    Ok(())
}

#[test]
fn test_encryption_logic_simulation() -> anyhow::Result<()> {
    // This test simulates the logic used inside Iceoryx2DispatcherPlugin
    // since we cannot easily run the full dispatcher in a unit test environment without Iceoryx2 setup.

    // 1. Generate Keypair (Simulate External providing Public Key)
    let mut rng = thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let public_key = RsaPublicKey::from(&private_key);

    // 2. Mock Payload
    let original_payload = serde_json::json!({"foo": "bar", "secret": "password123"}).to_string();
    
    // 3. Encrypt (Simulate Dispatcher Logic)
    let data = original_payload.as_bytes();
    let encrypted_bytes = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
    let encoded = BASE64.encode(encrypted_bytes);
    
    // The dispatcher wraps the encrypted payload in a new JSON structure
    let sent_json_str = serde_json::json!({
        "secure_payload": encoded
    }).to_string();

    println!("Encrypted Payload: {}", sent_json_str);

    // 4. Verify/Decrypt (Simulate External Receiver)
    let sent_json: serde_json::Value = serde_json::from_str(&sent_json_str)?;
    
    // Check structure
    assert!(sent_json.get("secure_payload").is_some());
    assert!(sent_json.get("foo").is_none()); // Original fields should be hidden

    let received_encoded = sent_json["secure_payload"].as_str().unwrap();
    let decoded_bytes = BASE64.decode(received_encoded)?;
    
    let decrypted_bytes = private_key.decrypt(Pkcs1v15Encrypt, &decoded_bytes)?;
    let decrypted_payload = String::from_utf8(decrypted_bytes)?;

    assert_eq!(original_payload, decrypted_payload);
    println!("Decryption successful!");

    Ok(())
}

