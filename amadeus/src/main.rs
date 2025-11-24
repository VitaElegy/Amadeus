use amadeus::App;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // Create app and run - all configuration via method chaining
    App::new()
        .show_metadata(true)
        .run()
}
