use nx_core::nix::Generation;
use yansi::Paint;

pub fn print_generations(generations: &[Generation]) {
    if generations.is_empty() {
        nx_core::log::warn("No generations found");
        return;
    }

    eprintln!();
    eprintln!("  {:<6} {:<18} {:<12} {:<16} {}", "ID", "Date", "NixOS", "Kernel", "Config");
    eprintln!("  {:<6} {:<18} {:<12} {:<16} {}", "---", "----", "-----", "------", "------");

    for entry in generations {
        let current = if entry.is_current { " *" } else { "" };
        let config_rev = if entry.config_rev.is_empty() {
            "-".to_string()
        } else {
            entry.config_rev.clone()
        };

        eprintln!(
            "  {:<6} {:<18} {:<12} {:<16} {}{}",
            entry.id, entry.date, entry.nixos_version, entry.kernel, config_rev, current
        );
    }

    eprintln!();
    eprintln!("  {} = current generation", "*".green());
}

pub fn select_generations_interactive(generations: &[Generation]) -> Option<Vec<u64>> {
    if generations.is_empty() {
        nx_core::log::warn("No generations to select");
        return None;
    }

    eprintln!();
    eprintln!("  Available generations (current marked with *):");
    eprintln!();

    for entry in generations {
        let current = if entry.is_current { " *" } else { "  " };
        eprintln!("  {:>4}{}  {}", entry.id, current.red(), entry.date);
    }

    eprintln!();
    eprintln!("  Enter generation IDs to delete (space-separated), or 'q' to quit:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok()?;

    let input = input.trim();
    if input == "q" || input.is_empty() {
        return None;
    }

    let ids: Vec<u64> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if ids.is_empty() {
        nx_core::log::warn("No valid generation IDs entered");
        return None;
    }

    for id in &ids {
        if let Some(entry) = generations.iter().find(|g| g.id == *id) {
            if entry.is_current {
                nx_core::log::error(&format!("Cannot delete current generation {}", id));
                return None;
            }
        }
    }

    Some(ids)
}
