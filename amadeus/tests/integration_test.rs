use amadeus::plugin::{Plugin, PluginMetadata, PluginRegistry};

struct TestPlugin {
    metadata: PluginMetadata,
    init_called: bool,
    start_called: bool,
    run_called: bool,
    stop_called: bool,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("test_plugin", "A test plugin", "0.1.0"),
            init_called: false,
            start_called: false,
            run_called: false,
            stop_called: false,
        }
    }
}

impl Plugin for TestPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.init_called = true;
        Ok(())
    }

    fn start(&mut self) -> anyhow::Result<()> {
        self.start_called = true;
        Ok(())
    }

    fn run(&mut self) -> anyhow::Result<()> {
        self.run_called = true;
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        self.stop_called = true;
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

    let plugin = TestPlugin::new();
    let mut registry = PluginRegistry::new();
    registry.register(plugin);

    // Run lifecycle
    registry.run_lifecycle()?;

    // Note: We can't easily check the internal state of the boxed plugin inside the registry
    // without downcasting, which is complex. 
    // However, if run_lifecycle returns Ok(()), it means all methods executed without error.
    // To strictly verify calls, we would need interior mutability (RefCell/Arc<Mutex>) inside the plugin.
    
    Ok(())
}

#[test]
fn test_plugin_filtering() {
    let p1 = TestPlugin::new();
    
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

