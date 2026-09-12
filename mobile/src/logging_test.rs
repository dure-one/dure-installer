//! Manual test for logging macros

#[allow(unused_imports)]
use crate::{dure_info, dure_debug, dure_warn, dure_error};

#[test]
pub fn test_logging() {
    dure_info!("Test info message");
    log::debug!("Test debug message with arg: {}", 42);
    log::warn!("Test warning");
    log::error!("Test error: {}", "something went wrong");
}
