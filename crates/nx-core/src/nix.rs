use std::path::{Path, PathBuf};
use std::process::Command;
use color_eyre::eyre::{Result, eyre};
use crate::command::run_simple;

pub fn nix_command(name: &str) -> Command {
    let mut cmd = Command::new(name);
    cmd.env("NIXPKGS_ALLOW_UNFREE", "1");
    cmd
}

pub fn nix_build(flake_ref: &str, out_link: Option<&Path>) -> Result<()> {
    let mut cmd = nix_command("nix");
    cmd.arg("build");
    if let Some(link) = out_link {
        cmd.args(["--out-link", &link.to_string_lossy()]);
    }
    cmd.arg(flake_ref);

    let status = cmd.status()?;
    if !status.success() {
        return Err(eyre!("nix build failed"));
    }
    Ok(())
}

pub fn nix_eval(flake_ref: &str) -> Result<String> {
    let mut cmd = nix_command("nix");
    cmd.args(["eval", "--raw", flake_ref]);
    run_simple(&mut cmd)
}

pub fn nix_flake_update(flake_dir: &Path) -> Result<()> {
    let mut cmd = nix_command("nix");
    cmd.args(["flake", "update"]);
    cmd.current_dir(flake_dir);

    let status = cmd.status()?;
    if !status.success() {
        return Err(eyre!("nix flake update failed"));
    }
    Ok(())
}

pub fn resolve_flake_dir() -> Result<PathBuf> {
    // Try NX_FLAKE env var first
    if let Ok(flake) = std::env::var("NX_FLAKE") {
        let path = PathBuf::from(flake);
        if path.join("flake.nix").exists() {
            return Ok(path);
        }
    }

    // Try common locations
    let candidates = [
        PathBuf::from("/etc/nixos"),
        dirs::home_dir()
            .map(|h| h.join(".dotfiles"))
            .unwrap_or_default(),
    ];

    for path in &candidates {
        if path.join("flake.nix").exists() {
            return Ok(path.clone());
        }
    }

    Err(eyre!("Could not find flake.nix. Set NX_FLAKE or place flake.nix in ~/.dotfiles or /etc/nixos"))
}

pub fn get_hostname() -> Result<String> {
    let hostname = nix::unistd::gethostname()
        .map_err(|e| eyre!("Failed to get hostname: {}", e))?;
    Ok(hostname.to_string_lossy().to_string())
}

pub fn system_profiles_dir() -> PathBuf {
    PathBuf::from("/nix/var/nix/profiles")
}

pub fn list_generations() -> Result<Vec<Generation>> {
    let profiles_dir = system_profiles_dir();
    let mut generations = Vec::new();

    // System profiles are symlinks like system-1-link, system-2-link, etc.
    if !profiles_dir.exists() {
        return Ok(generations);
    }

    for entry in std::fs::read_dir(&profiles_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if let Some(num) = name.strip_prefix("system-").and_then(|s| s.strip_suffix("-link")) {
            if let Ok(id) = num.parse::<u64>() {
                let target = std::fs::read_link(entry.path())?;
                let nixos_version = read_file_content(&target.join("nixos-version")).unwrap_or_default();
                let kernel = read_file_content(&target.join("kernel-modules")).map(|_| {
                    std::fs::read_dir(target.join("lib/modules"))
                        .ok()
                        .and_then(|mut d| d.next())
                        .and_then(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .unwrap_or_default()
                }).unwrap_or_default();

                let config_rev = read_file_content(&target.join("etc/os-revision")).unwrap_or_default();

                let date = entry.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        Some(datetime.format("%Y-%m-%d %H:%M").to_string())
                    })
                    .unwrap_or_default();

                let is_current = {
                    let current_profile = profiles_dir.join("system");
                    if let Ok(link_target) = std::fs::read_link(&current_profile) {
                        // Resolve relative symlink target
                        let resolved = if link_target.is_relative() {
                            profiles_dir.join(&link_target)
                        } else {
                            link_target
                        };
                        resolved == entry.path()
                    } else {
                        false
                    }
                };

                generations.push(Generation {
                    id,
                    date,
                    nixos_version,
                    kernel,
                    config_rev,
                    is_current,
                });
            }
        }
    }

    generations.sort_by_key(|g| g.id);
    Ok(generations)
}

fn read_file_content(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

#[derive(Debug, Clone)]
pub struct Generation {
    pub id: u64,
    pub date: String,
    pub nixos_version: String,
    pub kernel: String,
    pub config_rev: String,
    pub is_current: bool,
}
