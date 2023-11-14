use std::{
    io::Write,
    sync::{Arc, RwLock},
};

#[derive(Copy, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl From<LogLevel> for mint::Vector4<f32> {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => [0.5, 0.5, 1.0, 1.0].into(),
            LogLevel::Info => [0.5, 0.5, 0.5, 1.0].into(),
            LogLevel::Warn => [1.0, 0.64, 0.0, 1.0].into(),
            LogLevel::Error => [1.0, 0.0, 0.0, 1.0].into(),
        }
    }
}

impl From<log::LevelFilter> for LogLevel {
    fn from(level: log::LevelFilter) -> Self {
        match level {
            log::LevelFilter::Debug => LogLevel::Debug,
            log::LevelFilter::Info => LogLevel::Info,
            log::LevelFilter::Warn => LogLevel::Warn,
            log::LevelFilter::Error => LogLevel::Error,
            _ => LogLevel::Debug,
        }
    }
}

pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
}

impl LogMessage {
    pub fn new(level: LogLevel, message: &str) -> Self {
        Self {
            level,
            message: message.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Log {
    pub history: Vec<LogMessage>,
}

impl Log {
    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn log(&mut self, message: &str, level: LogLevel) {
        self.history.push(LogMessage::new(level, message));
    }
}

#[derive(Clone)]
pub struct WritableLog {
    pub arc: Arc<RwLock<Log>>,
    buf: String,
}

impl Default for WritableLog {
    fn default() -> Self {
        Self {
            arc: Arc::new(RwLock::new(Log::default())),
            buf: String::with_capacity(1024),
        }
    }
}

impl Write for WritableLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.push_str(std::str::from_utf8(buf).unwrap());
        if buf.contains(&b'\n') {
            self.flush()?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // match the log level found at index 0 till space to LogLevel
        let level = match &self.buf[0..self.buf.find(' ').unwrap()] {
            "[ERROR]" => LogLevel::Error,
            "[WARN]" => LogLevel::Warn,
            "[INFO]" => LogLevel::Info,
            "[DEBUG]" => LogLevel::Debug,
            _ => LogLevel::Info,
        };

        let mut logger = self.arc.write().unwrap();
        logger.log(&self.buf, level);
        self.buf.clear();

        Ok(())
    }
}
