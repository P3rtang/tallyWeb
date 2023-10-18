#![allow(dead_code)]
use chrono::{DateTime, Local};
use termion::{
    color::{self, Fg},
    style,
};
use thiserror::Error;

pub static mut LOG_LEVEL: LogLevel = LogLevel::Info;

#[macro_export]
macro_rules! server_log {
    ($level: ident,  $($args: tt)*) => {{
        pub use crate::log::{LogError, LogLevel, LogMessage};
        pub use crate::log::LOG_LEVEL;
        unsafe {
            match stringify!($level) {
                "debug" if LOG_LEVEL == LogLevel::Debug => {
                    LogMessage::new(LogLevel::Debug, format_args!($($args)*).to_string()).print()
                }
                "info" if LOG_LEVEL <= LogLevel::Info => {
                    LogMessage::new(LogLevel::Info, format_args!($($args)*).to_string()).print()
                }
                "warn" if LOG_LEVEL <= LogLevel::Warn => {
                    LogMessage::new(LogLevel::Warn, format_args!($($args)*).to_string()).print()
                }
                "fatal" if LOG_LEVEL <= LogLevel::Fatal => {
                    LogMessage::new(LogLevel::Fatal, format_args!($($args)*).to_string()).print()
                }
                "debug" | "info" | "warn" | "fatal" => {}
                expr => eprintln!("{:?}", LogError::InvalidLogLevel(expr.to_string())),
            }
        }
    }};
}

#[macro_export]
macro_rules! errplace {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

#[derive(Debug, Error)]
pub enum LogError {
    #[error("{0} is not a valid logging level")]
    InvalidLogLevel(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Fatal,
}

impl TryFrom<String> for LogLevel {
    type Error = LogError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "fatal" => Ok(Self::Fatal),
            level => Err(LogError::InvalidLogLevel(level.to_string())),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "{}[DEBUG]{}", Fg(color::Green), Fg(color::Reset)),
            LogLevel::Info => write!(f, "{}[INFO]{}", Fg(color::Yellow), Fg(color::Reset)),
            LogLevel::Warn => write!(f, "{}[WARN]{}", Fg(color::Red), Fg(color::Reset)),
            LogLevel::Fatal => write!(
                f,
                "{}{}[FATAL]{}{}",
                Fg(color::Red),
                style::Bold,
                Fg(color::Reset),
                style::Reset
            ),
        }
    }
}

#[derive(Debug, Default)]
enum TimeInfoFormat {
    #[default]
    Full,
    NoDate,
    None,
}

struct TimeInfo {
    time: DateTime<Local>,
    format: TimeInfoFormat,
}

impl TimeInfo {
    fn now() -> Self {
        return Self {
            time: Local::now(),
            format: TimeInfoFormat::default(),
        };
    }
}

impl std::fmt::Display for TimeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            TimeInfoFormat::Full => write!(
                f,
                "{}{}{}",
                Fg(color::Blue),
                self.time.format("[%d/%m - %H:%M:%S]"),
                Fg(color::Reset)
            ),
            TimeInfoFormat::NoDate => todo!(),
            TimeInfoFormat::None => todo!(),
        }
    }
}

pub struct LogMessage {
    level: LogLevel,
    time_info: TimeInfo,
    message: String,
}

impl LogMessage {
    pub fn new(level: LogLevel, msg: impl Into<String>) -> Self {
        return Self {
            level,
            time_info: TimeInfo::now(),
            message: msg.into(),
        };
    }

    pub fn print(&self) {
        match self.level {
            LogLevel::Debug => println!("{} {}:  {}", self.time_info, self.level, self.message),
            LogLevel::Info => println!("{} {}:   {}", self.time_info, self.level, self.message),
            LogLevel::Warn => eprintln!("{} {}:   {}", self.time_info, self.level, self.message),
            LogLevel::Fatal => eprintln!("{} {}:  {}", self.time_info, self.level, self.message),
        }
    }
}

#[cfg(test)]
mod test_logging {
    use super::*;

    #[test]
    fn formatting() -> Result<(), LogError> {
        server_log!(warn, "testing");
        Ok(())
    }
}
