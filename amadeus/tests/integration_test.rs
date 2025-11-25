use amadeus::plugin::{Plugin, PluginMetadata, PluginRegistry};
use std::sync::{Arc, Mutex};

struct TestState {
    init_called: bool,
    start_called: bool,
    stop_called: bool,
}

struct TestPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<TestState>>,
}

impl TestPlugin {
    fn new(state: Arc<Mutex<TestState>>) -> Self {
        Self {
            metadata: PluginMetadata::new("test_plugin", "A test plugin", "0.1.0"),
            state,
        }
    }
}

impl Plugin for TestPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.state.lock().unwrap().init_called = true;
        Ok(())
    }

    fn start(&mut self) -> anyhow::Result<()> {
        self.state.lock().unwrap().start_called = true;
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        self.state.lock().unwrap().stop_called = true;
        Ok(())
    }
}

#[test]
fn test_plugin_metadata_serialization() {
    let metadata = PluginMetadata::new("my_plugin", "My Description", "1.0.0")
        .author("Alice")
        .enabled_by_default(false)
        .with_property("key", "value");

    let json = serde_json::to_string(&metadata).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["name"], "my_plugin");
    assert_eq!(value["description"], "My Description");
    assert_eq!(value["version"], "1.0.0");
    assert_eq!(value["author"], "Alice");
    assert_eq!(value["enabled_by_default"], false);
    assert_eq!(value["properties"]["key"], "value");
}

#[test]
fn test_plugin_lifecycle() -> anyhow::Result<()> {
    // Initialize tracing (optional, won't show in standard test output without --nocapture)
    let _ = tracing_subscriber::fmt::try_init();

    let state = Arc::new(Mutex::new(TestState {
        init_called: false,
        start_called: false,
        stop_called: false,
    }));

    let plugin = TestPlugin::new(state.clone());
    let mut registry = PluginRegistry::new();
    registry.register(plugin);

    // Startup (Init -> Start)
    registry.startup()?;

    {
        let s = state.lock().unwrap();
        assert!(s.init_called, "Init should be called");
        assert!(s.start_called, "Start should be called");
        assert!(!s.stop_called, "Stop should NOT be called yet");
    }

    // Shutdown (Stop)
    registry.shutdown()?;

    {
        let s = state.lock().unwrap();
        assert!(s.stop_called, "Stop should be called");
    }
    
    Ok(())
}

#[test]
fn test_plugin_filtering() {
    let state = Arc::new(Mutex::new(TestState {
        init_called: false,
        start_called: false,
        stop_called: false,
    }));

    let p1 = TestPlugin::new(state);
    
    let mut p2_meta = PluginMetadata::new("disabled_plugin", "Disabled", "0.1.0");
    p2_meta = p2_meta.enabled_by_default(false);
    
    struct DisabledPlugin { meta: PluginMetadata }
    impl Plugin for DisabledPlugin {
        fn metadata(&self) -> &PluginMetadata { &self.meta }
    }
    let p2 = DisabledPlugin { meta: p2_meta };

    let mut registry = PluginRegistry::new();
    
    // Register only enabled plugins
    let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(p1), Box::new(p2)];
    registry.register_enabled(plugins);

    assert_eq!(registry.plugins().len(), 1);
    assert_eq!(registry.plugins()[0].metadata().name, "test_plugin");
}
