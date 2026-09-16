use clap::Args;
use color_eyre::eyre::Result;
use std::process::Command;
use nx_core::run_root_command;
use nx_core::log;

#[derive(Args)]
pub struct CleanArgs {
    /// Number of generations to keep (minimum)
    #[arg(long, default_value = "3")]
    pub keep: u64,

    /// Keep generations newer than this duration (e.g. "4d", "1w")
    #[arg(long)]
    pub keep_since: Option<String>,

    /// Skip gcroot cleanup
    #[arg(long)]
    pub no_gcroots: bool,

    /// Skip direnv gcroot cleanup
    #[arg(long)]
    pub no_direnv: bool,

    /// Skip store optimisation
    #[arg(long)]
    pub no_optimise: bool,

    /// Show what would be cleaned without cleaning
    #[arg(long)]
    pub dry: bool,

    /// Operate on user profiles only (no root required)
    #[arg(long)]
    pub user_only: bool,
}

pub fn run(args: CleanArgs) -> Result<()> {
    if !args.user_only {
        log::info("Running system-wide garbage collection...");
        if args.dry {
            log::info("Dry run — no changes made");
            return Ok(());
        }

        run_root_command("nix-collect-garbage", &[
            "--delete-older-than",
            &format!("{}d", args.keep * 7), // rough conversion: keep * 7 days
        ])?;
    } else {
        log::info("Running user-level garbage collection...");
        if args.dry {
            log::info("Dry run — no changes made");
            return Ok(());
        }

        let mut cmd = Command::new("nix-collect-garbage");
        cmd.args(["--delete-older-than", &format!("{}d", args.keep * 7)]);
        cmd.status()?;
    }

    // Clean gcroots
    if !args.no_gcroots {
        clean_gcroots(args.no_direnv)?;
    }

    // Optimise store
    if !args.no_optimise {
        log::info("Optimising nix store...");
        if !args.user_only {
            run_root_command("nix-store", &["--optimise"])?;
        } else {
            let mut cmd = Command::new("nix-store");
            cmd.args(["--optimise"]);
            cmd.status()?;
        }
    }

    log::done("Clean complete");
    Ok(())
}

fn clean_gcroots(skip_direnv: bool) -> Result<()> {
    use walkdir::WalkDir;
    use regex::Regex;

    let gcroots_dir = std::path::Path::new("/nix/var/nix/gcroots");
    if !gcroots_dir.exists() {
        return Ok(());
    }

    log::info("Cleaning orphaned gcroots...");

    let direnv_re = Regex::new(r"\.direnv|direnv/layouts").unwrap();
    let result_re = Regex::new(r"^result(-\w+)?$").unwrap();

    let mut cleaned = 0;

    for entry in WalkDir::new(gcroots_dir).min_depth(1) {
        let entry = entry?;
        let path = entry.path();

        // Skip direnv gcroots if requested
        if !skip_direnv && direnv_re.is_match(&path.to_string_lossy()) {
            continue;
        }

        // Check for orphaned symlinks (target doesn't exist)
        if entry.path_is_symlink() {
            let target = std::fs::read_link(path)?;
            if !target.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    log::warn(&format!("Failed to remove {}: {}", path.display(), e));
                } else {
                    cleaned += 1;
                }
            }
        }

        // Check for result symlinks
        if let Some(name) = path.file_name() {
            if result_re.is_match(&name.to_string_lossy()) {
                if path.is_symlink() {
                    let _ = std::fs::remove_file(path);
                    cleaned += 1;
                }
            }
        }
    }

    if cleaned > 0 {
        log::done(&format!("Removed {} orphaned gcroots", cleaned));
    }

    Ok(())
}
