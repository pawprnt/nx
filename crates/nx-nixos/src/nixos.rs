use std::path::{Path, PathBuf};
use std::process::Command;
use color_eyre::eyre::{Result, eyre};
use nx_core::command::{run_command, RunOptions};
use nx_core::log;
use nx_core::nix;
use crate::args::{RebuildArgs, RollbackArgs, DeleteArgs};
use crate::generations;

pub fn rebuild(args: RebuildArgs) -> Result<()> {
    let flake_dir = args.flake
        .map(PathBuf::from)
        .unwrap_or_else(|| nix::resolve_flake_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if !flake_dir.join("flake.nix").exists() {
        return Err(eyre!("No flake.nix found in {}", flake_dir.display()));
    }

    let hostname = nix::get_hostname()?;

    // Update flake inputs if requested
    if args.update {
        log::info("Updating flake inputs...");
        if let Some(inputs) = &args.update_input {
            for input in inputs {
                let mut cmd = Command::new("nix");
                cmd.args(["flake", "lock", "--update-input", input]);
                cmd.current_dir(&flake_dir);
                let status = cmd.status()?;
                if !status.success() {
                    log::warn(&format!("Failed to update input: {}", input));
                }
            }
        } else {
            nix::nix_flake_update(&flake_dir)?;
        }
        log::done("Flake inputs updated");
    }

    // Update configuration revision
    let date = chrono::Local::now().format("%Y.%m.%d.%I.%M.%p").to_string();
    let nix_nix = flake_dir.join("modules/core/nix.nix");
    if nix_nix.exists() {
        let content = std::fs::read_to_string(&nix_nix)?;
        let updated = content.replace(
            &extract_revision(&content),
            &date.to_string(),
        );
        if updated != content {
            std::fs::write(&nix_nix, &updated)?;
            log::info(&format!("Updated configuration revision to {}", date));
        }
    }

    // Build
    log::info("Building system...");
    let toplevel = format!("nixosConfigurations.{}.config.system.build.toplevel", hostname);

    if args.dry {
        log::info(&format!("Would build: {}", toplevel));
        return Ok(());
    }

    let spinner = nx_core::progress::Spinner::new("Building system configuration...");

    let mut cmd = Command::new("nix");
    cmd.args(["build", "--no-link", "--print-out-paths"]);
    cmd.arg(&toplevel);
    cmd.current_dir(&flake_dir);

    let (status, stdout, stderr) = run_command(&mut cmd, RunOptions {
        show_output: false,
        capture: true,
    })?;

    if !status.success() {
        spinner.fail_with_message("Build failed");
        eprintln!();
        log::error("Build output:");
        eprintln!("{}", stderr);
        return Err(eyre!("nix build failed"));
    }

    let store_path = stdout.trim().to_string();
    spinner.finish_with_message(&format!("Built: {}", store_path));

    // Activate
    log::info("Activating system...");
    let stc = Path::new(&store_path).join("bin/switch-to-configuration");

    if stc.exists() {
        let mut cmd = Command::new(&stc);
        cmd.arg("switch");
        let status = cmd.status()?;
        if !status.success() {
            return Err(eyre!("switch-to-configuration failed"));
        }
    } else {
        log::warn("switch-to-configuration not found, skipping activation");
    }

    // Set system profile
    let mut cmd = Command::new("nix");
    cmd.args(["build", "--profile", "/nix/var/nix/profiles/system"]);
    cmd.arg(&toplevel);
    cmd.current_dir(&flake_dir);
    cmd.status()?;

    log::done(&format!("Rebuild complete — revision {}", date));

    // Commit dotfiles
    if !args.no_commit {
        log::info("Committing dotfiles...");
        let mut cmd = Command::new("git");
        cmd.args(["add", "-A"]);
        cmd.current_dir(&flake_dir);
        cmd.status()?;

        let mut cmd = Command::new("git");
        cmd.args(["diff", "--cached", "--quiet"]);
        cmd.current_dir(&flake_dir);
        let status = cmd.status()?;

        if !status.success() {
            let commit_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let mut cmd = Command::new("git");
            cmd.args(["commit", "-m", &commit_date]);
            cmd.current_dir(&flake_dir);
            cmd.status()?;
            log::done(&format!("Committed: {}", commit_date));
        } else {
            log::info("No changes to commit");
        }
    }

    Ok(())
}

pub fn rollback(args: RollbackArgs) -> Result<()> {
    let generations = nix::list_generations()?;
    if generations.is_empty() {
        return Err(eyre!("No generations found"));
    }

    let target = if let Some(id) = args.to {
        generations.iter().find(|g| g.id == id)
            .ok_or_else(|| eyre!("Generation {} not found", id))?
    } else {
        generations.iter().rev().find(|g| !g.is_current)
            .ok_or_else(|| eyre!("No previous generation to roll back to"))?
    };

    log::info(&format!("Rolling back to generation {}...", target.id));

    let profile_link = nix::system_profiles_dir().join(format!("system-{}-link", target.id));
    if !profile_link.exists() {
        return Err(eyre!("Generation {} profile not found", target.id));
    }

    // Set the system profile
    let mut cmd = Command::new("nix");
    cmd.args(["profile", "install"]);
    cmd.args(["--profile", "/nix/var/nix/profiles/system"]);
    cmd.arg(&profile_link);
    cmd.status()?;

    // Activate
    let stc = profile_link.join("bin/switch-to-configuration");
    if stc.exists() {
        let mut cmd = Command::new(&stc);
        cmd.arg("switch");
        cmd.status()?;
    }

    log::done(&format!("Rolled back to generation {}", target.id));
    Ok(())
}

pub fn info() -> Result<()> {
    let generations = nix::list_generations()?;
    generations::print_generations(&generations);
    Ok(())
}

pub fn delete(args: DeleteArgs) -> Result<()> {
    let generations = nix::list_generations()?;
    if generations.is_empty() {
        log::warn("No generations found");
        return Ok(());
    }

    let ids = if let Some(ids) = args.ids {
        // Validate IDs
        for id in &ids {
            if let Some(entry) = generations.iter().find(|g| g.id == *id) {
                if entry.is_current {
                    return Err(eyre!("Cannot delete current generation {}", id));
                }
            } else {
                return Err(eyre!("Generation {} not found", id));
            }
        }
        ids
    } else {
        // Interactive selection
        match generations::select_generations_interactive(&generations) {
            Some(ids) => ids,
            None => {
                log::info("Cancelled");
                return Ok(());
            }
        }
    };

    // Confirm
    eprintln!();
    log::warn(&format!("Will delete {} generation(s): {:?}", ids.len(), ids));
    if args.dry {
        log::info("Dry run — no changes made");
        return Ok(());
    }

    eprint!("  Continue? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        log::info("Cancelled");
        return Ok(());
    }

    // Delete generation profiles
    let profiles_dir = nix::system_profiles_dir();
    for id in &ids {
        let link = profiles_dir.join(format!("system-{}-link", id));
        if link.exists() {
            std::fs::remove_file(&link)?;
            log::done(&format!("Deleted generation {}", id));
        }
    }

    // Run garbage collection
    if !args.no_gc {
        log::info("Running garbage collection...");
        let mut cmd = Command::new("nix-collect-garbage");
        cmd.status()?;
        log::done("Garbage collection complete");
    }

    // Refresh boot menu
    let stc = Path::new("/run/current-system/bin/switch-to-configuration");
    if stc.exists() {
        let mut cmd = Command::new(stc);
        cmd.arg("boot");
        cmd.status()?;
    }

    log::done("Deletion complete");
    Ok(())
}

fn extract_revision(content: &str) -> String {
    for line in content.lines() {
        if let Some(start) = line.find("system.configurationRevision = \"") {
            let rest = &line[start + 30..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_string();
            }
        }
    }
    String::new()
}
