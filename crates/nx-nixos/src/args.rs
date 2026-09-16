use clap::{Args, Subcommand};

#[derive(Args)]
pub struct OsArgs {
    #[command(subcommand)]
    pub action: OsAction,
}

#[derive(Subcommand)]
pub enum OsAction {
    /// Build and activate the system configuration
    Rebuild(RebuildArgs),

    /// Roll back to a previous generation
    Rollback(RollbackArgs),

    /// Show information about current and past generations
    Info,

    /// Delete specific generations by ID
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct RebuildArgs {
    /// Update flake inputs before rebuilding
    #[arg(long)]
    pub update: bool,

    /// Specific inputs to update (can be repeated)
    #[arg(long = "update-input", action = clap::ArgAction::Append)]
    pub update_input: Option<Vec<String>>,

    /// Don't commit dotfiles after rebuild
    #[arg(long)]
    pub no_commit: bool,

    /// Show what would be built without building
    #[arg(long)]
    pub dry: bool,

    /// Path to the flake directory
    #[arg(long, env = "NX_FLAKE")]
    pub flake: Option<String>,
}

#[derive(Args)]
pub struct RollbackArgs {
    /// Roll back to a specific generation (default: previous)
    #[arg(long)]
    pub to: Option<u64>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Generation IDs to delete (interactive if omitted)
    pub ids: Option<Vec<u64>>,

    /// Show what would be deleted without deleting
    #[arg(long)]
    pub dry: bool,

    /// Skip garbage collection after deletion
    #[arg(long)]
    pub no_gc: bool,

    /// Keep generations newer than this duration (e.g. "4d", "1w")
    #[arg(long)]
    pub keep_since: Option<String>,

    /// Number of generations to keep (minimum)
    #[arg(long)]
    pub keep: Option<u64>,
}
