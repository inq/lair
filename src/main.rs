use clap::Parser;
use dope::launcher::Launcher;
use sark::{Build, ServerCfg};
use tracing_subscriber::EnvFilter;

mod config;
mod lair;

#[derive(Debug, thiserror::Error)]
enum Error {
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

    // Thread-per-core: one pinned worker per CPU, all sharing the bind port via
    // SO_REUSEPORT (enabled by Build::http). Honour an explicit --cpus list,
    // otherwise bind every CPU the process is allowed to run on.
    let cpus = match config.cpus {
        Some(cpus) => cpus
            .into_iter()
            .map(u16::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        None => Launcher::allowed_cpus(),
    };

    let cfg = ServerCfg {
        bind: config.bind.parse()?,
        max_conn: 1024,
        backlog: 4096,
    };

    tracing::info!(?cpus, bind = %cfg.bind, "lair starting");

    Launcher::new(cpus).run(move |ctx| {
        // Each core owns its State: a per-core visit counter and asset pointers,
        // so there is no cross-core sharing and no atomics on the hot path.
        let state: &'static lair::State = Box::leak(Box::new(lair::State::new()));
        Build::http(lair::lair::new(state), cfg.clone(), ctx, None)
    })?;
    Ok(())
}
