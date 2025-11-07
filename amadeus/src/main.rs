use amadeus::App;

fn main() -> anyhow::Result<()> {
    // 创建应用并运行 - 所有配置都通过链式调用完成
    App::new()
        .show_metadata(true)
        .run()
}
