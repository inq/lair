use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lair")]
pub(super) struct Config {
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub(super) bind: String,

    #[arg(long, value_delimiter = ',')]
    pub(super) cpus: Option<Vec<usize>>,
}
