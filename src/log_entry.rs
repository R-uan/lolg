use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

pub struct LogEntry {
    pub level: Level,
    /// Datetime that the entry was created (in unix time).
    pub timestamp: u64,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: Level, msg: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        return Self {
            level,
            timestamp,
            message: msg.to_string(),
        };
    }

    pub fn bytes(&self) -> Vec<u8> {
        let level = self.level.parse();
        let timestamp = self.timestamp.to_le_bytes();
        let message_bytes = self.message.as_bytes();
        let size = message_bytes.len().to_le_bytes();

        let mut packet = Vec::new();
        packet.extend(level);
        packet.extend(timestamp);
        packet.extend(size);
        packet.extend([0x0A]);
        packet.extend(message_bytes);
        return packet;
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.level {
            Level::Debug => "[DEBUG]",
            Level::Info => "[INFO]",
            Level::Warning => "[WARNING]",
            Level::Error => "[ERROR]",
        };

        let time = DateTime::from_timestamp(self.timestamp as i64, 0).unwrap_or_default();
        let time_fmt = time.format("[%d/%m/%Y - %H:%M:%S]");
        let output = format!("{level} {time_fmt} {}", self.message);
        write!(f, "{}", &output)
    }
}

/// Log Level
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

impl Level {
    /// Parses the Level enum into a 4 byte little endian array
    pub fn parse(&self) -> [u8; 4] {
        match &self {
            Self::Debug => [0x01, 0x00, 0x00, 0x00],
            Self::Info => [0x02, 0x00, 0x00, 0x00],
            Self::Error => [0x03, 0x00, 0x00, 0x00],
            Self::Warning => [0x04, 0x00, 0x00, 0x00],
        }
    }
}
