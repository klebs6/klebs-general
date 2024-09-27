crate::ix!();

// Enum to represent log levels
#[repr(usize)]
pub enum LogLevel {
    ERROR = 0,
    WARN  = 1,
    INFO  = 2,
    DEBUG = 3,
    TRACE = 4,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::ERROR => LogLevel::ERROR,
            Level::WARN => LogLevel::WARN,
            Level::INFO => LogLevel::INFO,
            Level::DEBUG => LogLevel::DEBUG,
            Level::TRACE => LogLevel::TRACE,
        }
    }
}

impl From<LogLevel> for Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::ERROR => Level::ERROR,
            LogLevel::WARN => Level::WARN,
            LogLevel::INFO => Level::INFO,
            LogLevel::DEBUG => Level::DEBUG,
            LogLevel::TRACE => Level::TRACE,
        }
    }
}
