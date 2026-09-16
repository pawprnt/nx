use std::process::{Command, ExitStatus, Stdio};
use std::io::{BufRead, BufReader};
use color_eyre::eyre::{Result, eyre};

pub struct RunOptions {
    pub show_output: bool,
    pub capture: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            show_output: true,
            capture: false,
        }
    }
}

pub fn run_command(cmd: &mut Command, opts: RunOptions) -> Result<(ExitStatus, String, String)> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    // Read stderr in real-time (build output goes to stderr from nix)
    for line in stderr_reader.lines() {
        let line = line?;
        if opts.show_output && !opts.capture {
            eprintln!("{}", line);
        }
        stderr_lines.push(line);
    }

    // Read stdout
    for line in stdout_reader.lines() {
        let line = line?;
        if opts.show_output && !opts.capture {
            println!("{}", line);
        }
        stdout_lines.push(line);
    }

    let status = child.wait()?;
    let stdout_str = stdout_lines.join("\n");
    let stderr_str = stderr_lines.join("\n");

    Ok((status, stdout_str, stderr_str))
}

pub fn run_simple(cmd: &mut Command) -> Result<String> {
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("Command failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn require_root() -> Result<()> {
    if !nix::unistd::Uid::effective().is_root() {
        Err(eyre!("This command requires root. Re-run with sudo."))
    } else {
        Ok(())
    }
}
