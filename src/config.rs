use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lair")]
pub(super) struct Config {
    /// Address to bind, e.g. 0.0.0.0:8080.
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub(super) bind: String,

    /// CPUs to pin workers to (comma-separated). Defaults to every CPU the
    /// process is allowed to run on (thread-per-core).
    #[arg(long, value_delimiter = ',')]
    pub(super) cpus: Option<Vec<usize>>,
}
