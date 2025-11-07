use anyhow::Result;

/// 插件配置结构
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub max_workers: usize,
    pub timeout_seconds: u64,
    pub enable_logging: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            max_workers: 4,
            timeout_seconds: 30,
            enable_logging: true,
        }
    }
}

impl PluginConfig {
    /// 加载配置
    pub fn load(&mut self) -> Result<()> {
        println!("[Config] 加载配置文件...");
        
        // 这里可以从文件、环境变量等加载配置
        // 示例中使用默认值
        
        println!("[Config] 配置加载完成");
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.max_workers == 0 {
            anyhow::bail!("max_workers 必须大于 0");
        }
        if self.timeout_seconds == 0 {
            anyhow::bail!("timeout_seconds 必须大于 0");
        }
        Ok(())
    }
}

