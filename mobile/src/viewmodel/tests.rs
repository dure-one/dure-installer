#[cfg(test)]
use super::*;

#[test]
fn test_viewmodel_headless_initialization() {
    let vm = ViewModel::new_headless();

    // Should have empty state
    assert_eq!(vm.state.active_operations.len(), 0);
    assert_eq!(vm.state.recent_errors.len(), 0);
    assert_eq!(vm.state.wss_connections.len(), 0);
}

#[test]
fn test_viewmodel_poll_events_empty() {
    let mut vm = ViewModel::new_headless();

    // Polling with no events should return empty vec
    let events = vm.poll_events_headless();
    assert_eq!(events.len(), 0);
}
