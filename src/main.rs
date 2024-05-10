use std::net::SocketAddr;
use tokio::net::TcpListener;

mod lair;
mod server;

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = server::TokioIo::new(stream);

        let service = lair::LairService::new();

        tokio::task::spawn_local(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                tracing::error!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_writer(std::io::stderr)
        .init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, main_async())
}
