use sunaba::App;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("Starting Sunaba");
    
    pollster::block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let app = App::new().await?;
    app.run()
}
