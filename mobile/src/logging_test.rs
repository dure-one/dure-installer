//! Manual test for logging macros

#[allow(unused_imports)]
use crate::{dure_info, dure_debug, dure_warn, dure_error};

#[test]
pub fn test_logging() {
    dure_info!("Test info message");
    dure_debug!("Test debug message with arg: {}", 42);
    dure_warn!("Test warning");
    dure_error!("Test error: {}", "something went wrong");
}
