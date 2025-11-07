# 高级功能指南

## 📁 从配置文件加载插件设置

### 1. 创建配置文件

创建 `plugins_config.json`：

```json
[
  {
    "name": "my_plugin",
    "description": "我的插件",
    "version": "1.0.0",
    "enabled_by_default": true,
    "author": "Your Name",
    "properties": {
      "api_key": "secret-key-123",
      "timeout": "30",
      "max_retries": "3"
    }
  }
]
```

### 2. 在插件中读取配置

```rust
use crate::plugin::{Plugin, PluginMetadata};
use anyhow::Result;

pub struct ConfigurablePlugin {
    metadata: PluginMetadata,
    api_key: String,
    timeout: u64,
    max_retries: u32,
}

impl ConfigurablePlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "my_plugin",
            "我的插件",
            "1.0.0",
        );

        Self {
            metadata,
            api_key: String::new(),
            timeout: 30,
            max_retries: 3,
        }
    }

    /// 从配置中加载设置
    pub fn load_config_from_metadata(&mut self) -> Result<()> {
        let props = &self.metadata.properties;
        
        if let Some(key) = props.get("api_key") {
            self.api_key = key.clone();
        }
        
        if let Some(timeout) = props.get("timeout") {
            self.timeout = timeout.parse().unwrap_or(30);
        }
        
        if let Some(retries) = props.get("max_retries") {
            self.max_retries = retries.parse().unwrap_or(3);
        }
        
        Ok(())
    }
}

impl Plugin for ConfigurablePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn init(&mut self) -> Result<()> {
        println!("[MyPlugin] 加载配置...");
        self.load_config_from_metadata()?;
        
        println!("[MyPlugin] API Key: {}", self.api_key);
        println!("[MyPlugin] Timeout: {}s", self.timeout);
        println!("[MyPlugin] Max Retries: {}", self.max_retries);
        
        Ok(())
    }
}
```

### 3. 在 main.rs 中使用

```rust
fn main() -> anyhow::Result<()> {
    let mut registry = PluginRegistry::new();

    // 从配置文件加载元数据
    match PluginRegistry::load_config("plugins_config.json") {
        Ok(configs) => {
            println!("✅ 成功从配置文件加载 {} 个插件配置", configs.len());
            
            for config in configs {
                println!("配置: {} - {}", config.name, config.description);
                // 根据配置创建插件实例
                // 这里需要根据 name 匹配对应的插件
            }
        }
        Err(e) => {
            println!("⚠️  加载配置文件失败: {}", e);
            println!("使用默认配置");
        }
    }

    // 注册插件
    registry.register(ConfigurablePlugin::new());

    // 导出当前配置
    let metadata: Vec<_> = registry.plugins()
        .iter()
        .map(|p| p.metadata().clone())
        .collect();
    
    PluginRegistry::save_config("plugins_config_export.json", &metadata)?;
    println!("✅ 配置已导出到 plugins_config_export.json");

    // 执行插件生命周期
    registry.init_all()?;
    registry.start_all()?;
    registry.run_all()?;
    registry.stop_all()?;

    Ok(())
}
```

## 🔄 插件间通信

### 使用共享状态

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// 共享状态
pub type SharedState = Arc<Mutex<HashMap<String, String>>>;

pub struct PluginA {
    metadata: PluginMetadata,
    shared_state: SharedState,
}

impl PluginA {
    pub fn new(shared_state: SharedState) -> Self {
        let metadata = PluginMetadata::new("plugin_a", "插件 A", "1.0.0");
        Self { metadata, shared_state }
    }
}

impl Plugin for PluginA {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn run(&mut self) -> Result<()> {
        let mut state = self.shared_state.lock().unwrap();
        state.insert("plugin_a_data".to_string(), "Hello from A".to_string());
        println!("[PluginA] 已写入共享状态");
        Ok(())
    }
}

pub struct PluginB {
    metadata: PluginMetadata,
    shared_state: SharedState,
}

impl PluginB {
    pub fn new(shared_state: SharedState) -> Self {
        let metadata = PluginMetadata::new("plugin_b", "插件 B", "1.0.0");
        Self { metadata, shared_state }
    }
}

impl Plugin for PluginB {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn run(&mut self) -> Result<()> {
        let state = self.shared_state.lock().unwrap();
        if let Some(data) = state.get("plugin_a_data") {
            println!("[PluginB] 读取到 Plugin A 的数据: {}", data);
        }
        Ok(())
    }
}

// 使用方式
fn main() -> anyhow::Result<()> {
    let shared_state = Arc::new(Mutex::new(HashMap::new()));
    
    let mut registry = PluginRegistry::new();
    registry.register(PluginA::new(shared_state.clone()));
    registry.register(PluginB::new(shared_state.clone()));
    
    registry.init_all()?;
    registry.run_all()?;
    registry.stop_all()?;
    
    Ok(())
}
```

## ⚡ 异步插件支持

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

### 2. 定义异步插件 Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait AsyncPlugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn run(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
```

### 3. 实现异步插件

```rust
pub struct AsyncHttpPlugin {
    metadata: PluginMetadata,
    client: reqwest::Client,
}

#[async_trait]
impl AsyncPlugin for AsyncHttpPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn init(&mut self) -> Result<()> {
        println!("[AsyncHttp] 初始化 HTTP 客户端");
        self.client = reqwest::Client::new();
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        println!("[AsyncHttp] 发送 HTTP 请求...");
        
        let response = self.client
            .get("https://api.example.com/data")
            .send()
            .await?;
        
        let body = response.text().await?;
        println!("[AsyncHttp] 响应: {}", body);
        
        Ok(())
    }
}
```

## 🎯 插件依赖管理

### 定义依赖

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled_by_default: bool,
    pub author: Option<String>,
    pub dependencies: Vec<String>,  // 新增
    pub properties: HashMap<String, String>,
}

impl PluginMetadata {
    pub fn with_dependencies(mut self, deps: Vec<&str>) -> Self {
        self.dependencies = deps.iter().map(|s| s.to_string()).collect();
        self
    }
}
```

### 拓扑排序

```rust
impl PluginRegistry {
    /// 按依赖顺序排序插件
    pub fn sort_by_dependencies(&mut self) -> Result<()> {
        // 使用拓扑排序确保依赖的插件先加载
        // 实现略...
        Ok(())
    }
}
```

## 🔥 热重载

```rust
use std::time::Duration;
use notify::{Watcher, RecursiveMode, recommended_watcher};

pub struct HotReloadRegistry {
    registry: PluginRegistry,
    watch_path: String,
}

impl HotReloadRegistry {
    pub fn watch(&mut self) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(
            std::path::Path::new(&self.watch_path),
            RecursiveMode::Recursive
        )?;

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    println!("检测到文件变更: {:?}", event);
                    println!("重新加载插件...");
                    // 重新加载插件逻辑
                }
                Err(_) => {
                    // 超时，继续等待
                }
            }
        }
    }
}
```

## 📊 插件优先级

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub priority: i32,  // 新增：数字越大优先级越高
    // ... 其他字段
}

impl PluginRegistry {
    /// 按优先级排序插件
    pub fn sort_by_priority(&mut self) {
        self.plugins.sort_by(|a, b| {
            b.metadata().priority.cmp(&a.metadata().priority)
        });
    }
}
```

## 🧪 插件测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_lifecycle() -> Result<()> {
        let mut plugin = MyPlugin::new();
        
        // 测试初始化
        plugin.init()?;
        
        // 测试运行
        plugin.run()?;
        
        // 测试停止
        plugin.stop()?;
        
        Ok(())
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = MyPlugin::new();
        let meta = plugin.metadata();
        
        assert_eq!(meta.name, "my_plugin");
        assert_eq!(meta.version, "1.0.0");
        assert!(meta.enabled_by_default);
    }

    #[test]
    fn test_registry() -> Result<()> {
        let mut registry = PluginRegistry::new();
        
        registry.register(MyPlugin::new());
        assert_eq!(registry.plugins().len(), 1);
        
        registry.init_all()?;
        registry.run_all()?;
        registry.stop_all()?;
        
        Ok(())
    }
}
```

## 🎨 插件宏

创建一个宏来简化插件定义：

```rust
#[macro_export]
macro_rules! define_plugin {
    (
        name: $name:expr,
        description: $desc:expr,
        version: $version:expr,
        $(author: $author:expr,)?
        $(enabled: $enabled:expr,)?
        init: $init:block,
        run: $run:block,
        $(stop: $stop:block,)?
    ) => {
        pub struct GeneratedPlugin {
            metadata: PluginMetadata,
        }

        impl GeneratedPlugin {
            pub fn new() -> Self {
                let mut metadata = PluginMetadata::new($name, $desc, $version);
                $(metadata = metadata.author($author);)?
                $(metadata.enabled_by_default = $enabled;)?
                
                Self { metadata }
            }
        }

        impl Plugin for GeneratedPlugin {
            fn metadata(&self) -> &PluginMetadata {
                &self.metadata
            }

            fn init(&mut self) -> Result<()> {
                $init
                Ok(())
            }

            fn run(&mut self) -> Result<()> {
                $run
                Ok(())
            }

            $(fn stop(&mut self) -> Result<()> {
                $stop
                Ok(())
            })?
        }
    };
}

// 使用宏
define_plugin! {
    name: "quick_plugin",
    description: "快速创建的插件",
    version: "1.0.0",
    author: "Me",
    enabled: true,
    init: {
        println!("快速初始化!");
    },
    run: {
        println!("快速运行!");
    },
    stop: {
        println!("快速停止!");
    },
}
```

## 🌟 最佳实践总结

1. **配置管理** - 使用 JSON 配置文件管理插件设置
2. **状态隔离** - 每个插件维护自己的状态
3. **错误处理** - 使用 `Result` 类型，优雅处理错误
4. **日志记录** - 在关键步骤添加日志
5. **测试覆盖** - 为每个插件编写单元测试
6. **文档完善** - 添加详细的文档注释
7. **性能优化** - 考虑使用异步处理 I/O 密集型任务
8. **安全性** - 验证配置，避免注入攻击

这些高级功能让您的插件系统更加强大和灵活！🚀

