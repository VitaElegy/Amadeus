use anyhow::Result;

/// 数据处理器
pub struct DataHandler {
    processed_count: usize,
}

impl DataHandler {
    pub fn new() -> Self {
        Self {
            processed_count: 0,
        }
    }

    /// 初始化处理器
    pub fn init(&mut self) -> Result<()> {
        println!("[Handler] 初始化数据处理器");
        self.processed_count = 0;
        Ok(())
    }

    /// 启动处理器
    pub fn start(&mut self) -> Result<()> {
        println!("[Handler] 启动数据处理器");
        // 可以在这里建立连接、打开文件等
        Ok(())
    }

    /// 处理数据
    pub fn process(&mut self, data: &[&str]) -> Result<()> {
        println!("[Handler] 处理 {} 条数据", data.len());
        
        for (idx, item) in data.iter().enumerate() {
            println!("[Handler] 处理第 {} 条: {}", idx + 1, item);
            self.processed_count += 1;
        }

        println!("[Handler] 累计处理 {} 条数据", self.processed_count);
        Ok(())
    }

    /// 清理资源
    pub fn cleanup(&mut self) -> Result<()> {
        println!("[Handler] 清理资源，总共处理了 {} 条数据", self.processed_count);
        self.processed_count = 0;
        Ok(())
    }
}

