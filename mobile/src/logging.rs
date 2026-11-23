//! Dure logging infrastructure.
//!
//! Provides custom macros for consistent console and in-app log routing.

// Helper function for routing logs to actor
pub fn log_to_actor(
    host_id: Option<String>,
    level: crate::viewmodel::logs::LogLevel,
    message: String,
) {
    let id_str = host_id.as_ref().map(|s| s.as_str()).unwrap_or("<context>");
    log::trace!(
        "[LOG_ROUTE] Routing log to actor - host_id='{}', level={:?}, msg='{}'",
        id_str,
        level,
        message
    );
    crate::viewmodel::logs::append_log(host_id, level, message);
}

// Non-WASM platforms: use standard log crate
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_info {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            log::info!($($arg)*);
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Info, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            log::info!($($arg)*);
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Info, format!($($arg)*));
        }
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_trace {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            log::trace!($($arg)*);
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Trace, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            log::trace!($($arg)*);
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Trace, format!($($arg)*));
        }
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_debug {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            log::debug!($($arg)*);
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Debug, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            log::debug!($($arg)*);
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Debug, format!($($arg)*));
        }
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_warn {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            log::warn!($($arg)*);
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Warn, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            log::warn!($($arg)*);
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Warn, format!($($arg)*));
        }
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_error {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            log::error!($($arg)*);
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Error, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            log::error!($($arg)*);
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Error, format!($($arg)*));
        }
    };
}

// WASM platform: use web_sys::console
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_info {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            web_sys::console::log_1(&format!("[INFO] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Info, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            web_sys::console::log_1(&format!("[INFO] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Info, format!($($arg)*));
        }
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_trace {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Trace, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            web_sys::console::debug_1(&format!("[TRACE] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Trace, format!($($arg)*));
        }
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_debug {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            web_sys::console::debug_1(&format!("[DEBUG] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Debug, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            web_sys::console::debug_1(&format!("[DEBUG] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Debug, format!($($arg)*));
        }
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_warn {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            web_sys::console::warn_1(&format!("[WARN] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Warn, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            web_sys::console::warn_1(&format!("[WARN] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Warn, format!($($arg)*));
        }
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_error {
    (project_id = $project_id:expr, $($arg:tt)*) => {
        {
            web_sys::console::error_1(&format!("[ERROR] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(Some($project_id.to_string()), $crate::viewmodel::logs::LogLevel::Error, format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            web_sys::console::error_1(&format!("[ERROR] {}", format!($($arg)*)).into());
            $crate::logging::log_to_actor(None, $crate::viewmodel::logs::LogLevel::Error, format!($($arg)*));
        }
    };
}
