use anyhow::Result;
use extism::{Plugin as ExtismPlugin, Manifest, Wasm};
use crate::plugin::{Plugin, PluginMetadata};
use std::sync::{Arc, Mutex};
use std::path::Path;

pub struct WasmPlugin {
    metadata: PluginMetadata,
    // Extism Plugin is not Sync, so we wrap it in Mutex
    plugin: Arc<Mutex<ExtismPlugin>>, 
}

impl WasmPlugin {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        
        let manifest = Manifest::new([Wasm::file(path)]);
        // Enable WASI by default for file/net access if needed
        let plugin = ExtismPlugin::new(&manifest, [], true)?;
        
        // TODO: Try to call a metadata function in WASM to get real metadata
        // For now, use filename
        let metadata = PluginMetadata::new(
            &name,
            "WASM Plugin",
            "0.0.1"
        );

        Ok(Self {
            metadata,
            plugin: Arc::new(Mutex::new(plugin)),
        })
    }
}

impl Plugin for WasmPlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        let mut plugin = self.plugin.lock().unwrap();
        if plugin.function_exists("init") {
            plugin.call::<(), ()>("init", ())?;
        }
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let mut plugin = self.plugin.lock().unwrap();
        if plugin.function_exists("start") {
            plugin.call::<(), ()>("start", ())?;
        }
        // Support 'run' as well if it exists
        if plugin.function_exists("run") {
            plugin.call::<(), ()>("run", ())?;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let mut plugin = self.plugin.lock().unwrap();
        if plugin.function_exists("stop") {
            plugin.call::<(), ()>("stop", ())?;
        }
        Ok(())
    }
}
