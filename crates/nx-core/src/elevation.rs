use std::process::Command;
use color_eyre::eyre::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    Sudo,
    Doas,
    Run0,
    Pkexec,
    None,
}

impl Elevation {
    pub fn detect() -> Self {
        if nix::unistd::Uid::effective().is_root() {
            return Self::None;
        }

        // Check in priority order
        if which::which("doas").is_ok() {
            return Self::Doas;
        }
        if which::which("run0").is_ok() {
            return Self::Run0;
        }
        if which::which("sudo").is_ok() {
            return Self::Sudo;
        }
        if which::which("pkexec").is_ok() {
            return Self::Pkexec;
        }

        Self::None
    }

    pub fn elevate(&self, cmd: &mut Command) {
        match self {
            Self::Sudo => {
                let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
                let program = cmd.get_program().to_string_lossy().to_string();
                cmd.arg(&program);
                cmd.args(&args);
                // Actually we need to wrap the entire command
                // Let's rebuild it
            }
            _ => {}
        }
    }

    pub fn is_available(&self) -> bool {
        *self != Self::None
    }
}

pub fn run_root_command(program: &str, args: &[&str]) -> Result<()> {
    let elev = Elevation::detect();

    let status = match elev {
        Elevation::Sudo => {
            let mut cmd = Command::new("sudo");
            cmd.arg(program).args(args);
            cmd.status()?
        }
        Elevation::Doas => {
            let mut cmd = Command::new("doas");
            cmd.arg(program).args(args);
            cmd.status()?
        }
        Elevation::Run0 => {
            let mut cmd = Command::new("run0");
            cmd.arg(program).args(args);
            cmd.status()?
        }
        Elevation::Pkexec => {
            let mut cmd = Command::new("pkexec");
            cmd.arg(program).args(args);
            cmd.status()?
        }
        Elevation::None => {
            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd.status()?
        }
    };

    if !status.success() {
        color_eyre::eyre::bail!("Command failed with exit code: {}", status);
    }
    Ok(())
}
