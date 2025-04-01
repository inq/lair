use tracing_subscriber::{self, EnvFilter};
use sark::prelude::*;

mod lair;

#[monoio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Lair service starting");
    
    let app = App::with_empty_state(lair::LairService::new());
    let server = Server::bind("127.0.0.1:8080");
    
    tracing::info!("Server listening on: 127.0.0.1:8080");
    server.serve(&app).await?;

    Ok(())
}
