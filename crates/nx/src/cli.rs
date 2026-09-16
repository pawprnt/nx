use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

#[derive(Parser)]
#[command(
    name = "nx",
    about = "A nix helper CLI",
    version,
    long_about = None,
    after_help = "Use 'nx <command> --help' for more information on a specific command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: Verbosity,
}

#[derive(Subcommand)]
pub enum Commands {
    /// System operations (rebuild, rollback, info, delete)
    Os(nx_nixos::args::OsArgs),

    /// Garbage collect old generations and optimise the store
    Clean(nx_clean::CleanArgs),

    /// Search for packages in nixpkgs
    Search(nx_search::SearchArgs),
}
