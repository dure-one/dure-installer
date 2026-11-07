use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Log settings for UI display
pub struct LogSettings {
    pub show_logs: bool,
    pub log_level: crate::LogLevel,
}

static LOG_BUFFER: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
static LOG_SETTINGS: OnceLock<Arc<Mutex<LogSettings>>> = OnceLock::new();

/// Get or initialize the log buffer
fn get_log_buffer() -> &'static Arc<Mutex<String>> {
    LOG_BUFFER.get_or_init(|| Arc::new(Mutex::new(String::new())))
}

/// Get or initialize log settings
fn get_log_settings() -> &'static Arc<Mutex<LogSettings>> {
    LOG_SETTINGS.get_or_init(|| {
        Arc::new(Mutex::new(LogSettings {
            show_logs: false,
            log_level: crate::LogLevel::Info,
        }))
    })
}

/// Update log settings (called from settings dialog)
pub fn update_log_settings(show_logs: bool, log_level: crate::LogLevel) {
    if let Ok(mut settings) = get_log_settings().lock() {
        settings.show_logs = show_logs;
        settings.log_level = log_level;
    }
}

/// Get log level priority (lower = higher priority)
fn level_priority(lvl: crate::LogLevel) -> i32 {
    match lvl {
        crate::LogLevel::Error => 0,
        crate::LogLevel::Warn => 1,
        crate::LogLevel::Info => 2,
        crate::LogLevel::Debug => 3,
        crate::LogLevel::Trace => 4,
    }
}

/// Append log message to buffer with level filtering
pub fn append_log(level: &str, message: String) {
    // Check if this log level should be captured
    let should_log = if let Ok(settings) = get_log_settings().lock() {
        let message_level = match level {
            "ERROR" => crate::LogLevel::Error,
            "WARN" => crate::LogLevel::Warn,
            "INFO" => crate::LogLevel::Info,
            "DEBUG" => crate::LogLevel::Debug,
            "TRACE" => crate::LogLevel::Trace,
            _ => return,
        };

        level_priority(message_level) <= level_priority(settings.log_level)
    } else {
        false
    };

    if !should_log {
        return;
    }

    if let Ok(mut buffer) = get_log_buffer().lock() {
        buffer.push_str(&message);
        buffer.push('\n');

        // Keep only last 10000 characters to prevent memory issues
        if buffer.len() > 10000 {
            *buffer = buffer.chars().skip(buffer.len() - 10000).collect();
        }
    }
}

/// Get current logs for display
pub fn get_logs() -> String {
    get_log_buffer()
        .lock()
        .map(|b| b.clone())
        .unwrap_or_default()
}

/// Combined logger that sends logs to both logcat (on Android) and in-app UI capture
struct CombinedLogger {
    level_filter: Arc<RwLock<log::LevelFilter>>,
}

impl log::Log for CombinedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if let Ok(filter) = self.level_filter.read() {
            metadata.level() <= *filter
        } else {
            false
        }
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Format the log message
        let level = record.level();
        let target = record.target();
        let message = format!("{}", record.args());

        // Skip empty messages
        if message.trim().is_empty() {
            return;
        }

        // Format the complete log line
        let log_line = format!("[{}] {}: {}", level, target, message);

        // Convert level to string
        let level_str = match level {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };

        // Append to global log buffer with level filtering
        append_log(level_str, log_line.clone());

        // On Android, also send to logcat via android_logger
        #[cfg(target_os = "android")]
        {
            // Use println/eprintln as backup - android_logger redirects these to logcat
            match level {
                log::Level::Error => eprintln!("[{}] {}", target, message),
                _ => println!("[{}] {}", target, message),
            }
        }

        // On non-Android, just print to stdout/stderr
        #[cfg(not(target_os = "android"))]
        {
            match level {
                log::Level::Error => eprintln!("{}", log_line),
                _ => println!("{}", log_line),
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<CombinedLogger> = OnceLock::new();

/// Initialize the combined logger that writes to both logcat and in-app log capture
pub fn init_combined_logger(level_filter: log::LevelFilter) {
    let logger = LOGGER.get_or_init(|| CombinedLogger {
        level_filter: Arc::new(RwLock::new(level_filter)),
    });

    // Set as the global logger
    if log::set_logger(logger).is_ok() {
        log::set_max_level(level_filter);
    }

    // On Android, also initialize android_logger as a backup
    #[cfg(target_os = "android")]
    {
        let _ = android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(level_filter)
                .with_tag("Dure"),
        );
    }
}

/// Update the log level at runtime
pub fn update_log_level(level: &str) {
    let level_filter = match level.to_uppercase().as_str() {
        "TRACE" => log::LevelFilter::Trace,
        "DEBUG" => log::LevelFilter::Debug,
        "INFO" => log::LevelFilter::Info,
        "WARN" => log::LevelFilter::Warn,
        "ERROR" => log::LevelFilter::Error,
        _ => log::LevelFilter::Error,
    };

    if let Some(logger) = LOGGER.get() {
        if let Ok(mut filter) = logger.level_filter.write() {
            *filter = level_filter;
            log::set_max_level(level_filter);
        }
    }
}
