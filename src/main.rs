extern crate dope;

use clap::Parser;
use dope::launcher::Launcher;
use sark::{Build, ServerCfg};
use tracing_subscriber::{self, EnvFilter};

mod config;
mod lair;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("lair requires explicit --cpus with at least one core")]
    MissingCpus,
    #[error("invalid bind address: {0}")]
    BindAddr(#[from] std::net::AddrParseError),
    #[error("cpu id out of u16 range: {0}")]
    CpuId(#[from] std::num::TryFromIntError),
    #[error("runtime: {0}")]
    Runtime(#[from] std::io::Error),
}

fn main() -> Result<(), Error> {
    let config = config::Config::parse();

    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("lair starting");

    let core = config
        .cpus
        .and_then(|mut cpus| cpus.drain(..).next())
        .ok_or(Error::MissingCpus)?;
    let cfg = ServerCfg {
        bind: config.bind.parse()?,
        max_conn: 1024,
        backlog: 4096,
    };
    let cpu = u16::try_from(core)?;
    Launcher::new(vec![cpu]).run(move |ctx| {
        let lair_state: &'static lair::State = Box::leak(Box::new(lair::State::new()));
        Build::http(lair::lair::new(lair_state), cfg.clone(), ctx, None)
    })?;
    Ok(())
}
