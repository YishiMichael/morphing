use std::time::SystemTime;

#[derive(Debug)]
struct LogRecord {
    timestamp: SystemTime,
    level: LogLevel,
    message: String,
}

// https://docs.rs/log/latest/src/log/lib.rs.html#484-508
#[derive(Debug)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    // https://docs.rs/env_logger/latest/src/env_logger/fmt/mod.rs.html#159
    fn color(&self) -> iced::Color {
        match self {
            Self::Error => iced::Color::from_rgb8(0xFF, 0x55, 0x55), // Red
            Self::Warn => iced::Color::from_rgb8(0xFF, 0xFF, 0x55),  // Yellow
            Self::Info => iced::Color::from_rgb8(0x55, 0xFF, 0x55),  // Green
            Self::Debug => iced::Color::from_rgb8(0x55, 0x55, 0xFF), // Blue
            Self::Trace => iced::Color::from_rgb8(0x55, 0xFF, 0xFF), // Cyan
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Logger(Vec<LogRecord>);

impl Logger {
    pub(crate) fn log<S>(&mut self, level: LogLevel, message: S)
    where
        S: AsRef<str>,
    {
        self.0.push(LogRecord {
            timestamp: SystemTime::now(),
            level,
            message: message.as_ref().to_string(),
        });
    }
}

// Use humantime::format_rfc3339_seconds
// [2025-01-01T12:34:56Z WARN ] message
