mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let env_filter = match cli.verbose.log_level_filter() {
        log::LevelFilter::Off => "off",
        log::LevelFilter::Error => "error",
        log::LevelFilter::Warn => "warn",
        log::LevelFilter::Info => "info",
        log::LevelFilter::Debug => "debug",
        log::LevelFilter::Trace => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(env_filter)),
        )
        .init();

    match cli.command {
        Commands::Os(args) => nx_nixos::run(args),
        Commands::Clean(args) => nx_clean::run(args),
        Commands::Search(args) => nx_search::run(args),
    }
}
