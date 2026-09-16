use std::fmt;
use yansi::{Color, Paint};

pub enum LogLevel {
    Info,
    Done,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (label, color) = match self {
            LogLevel::Info => ("INFO", Color::Cyan),
            LogLevel::Done => ("DONE", Color::Green),
            LogLevel::Warn => ("WARN", Color::Yellow),
            LogLevel::Error => ("ERROR", Color::Red),
        };
        write!(f, "{}", label.fg(color).bold())
    }
}

pub fn log(level: LogLevel, msg: &str) {
    eprintln!("  [{}] {}", level, msg);
}

pub fn info(msg: &str) {
    log(LogLevel::Info, msg);
}

pub fn done(msg: &str) {
    log(LogLevel::Done, msg);
}

pub fn warn(msg: &str) {
    log(LogLevel::Warn, msg);
}

pub fn error(msg: &str) {
    log(LogLevel::Error, msg);
}
