use std::time::SystemTime;

#[derive(Debug)]
struct LogRecord {
    timestamp: SystemTime,
    level: log::Level,
    message: String,
}

#[derive(Debug, Default)]
pub(crate) struct Logger(Vec<LogRecord>);

impl Logger {
    pub(crate) fn log<S>(&mut self, level: log::Level, message: S)
    where
        S: ToString,
    {
        self.0.push(LogRecord {
            timestamp: SystemTime::now(),
            level,
            message: message.to_string(),
        });
    }
}

// Use humantime::format_rfc3339_seconds
// [2025-01-01T12:34:56Z WARN ] message

// Colors from https://docs.rs/env_logger/latest/src/env_logger/fmt/mod.rs.html#159
fn convert_color(level: log::Level) -> iced::Color {
    match level {
        log::Level::Error => iced::Color::from_rgb8(0xFF, 0x55, 0x55), // Red
        log::Level::Warn => iced::Color::from_rgb8(0xFF, 0xFF, 0x55),  // Yellow
        log::Level::Info => iced::Color::from_rgb8(0x55, 0xFF, 0x55),  // Green
        log::Level::Debug => iced::Color::from_rgb8(0x55, 0x55, 0xFF), // Blue
        log::Level::Trace => iced::Color::from_rgb8(0x55, 0xFF, 0xFF), // Cyan
    }
}
