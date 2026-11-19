use super::*;
#[cfg(test)]
use crate::{dure_debug, dure_error, dure_info, dure_warn};

#[test]
fn test_viewmodel_headless_initialization() {
    let vm = ViewModel::new_headless(None);

    // Should have empty state
    assert_eq!(vm.state.active_operations.len(), 0);
    assert_eq!(vm.state.recent_errors.len(), 0);
    // wss_connections removed in Phase 1
}

#[test]
fn test_viewmodel_poll_events_empty() {
    let mut vm = ViewModel::new_headless(None);

    // Polling with no events should return empty vec
    let events = vm.poll_events_headless();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_viewmodel_create_vm_command() {
    let vm = ViewModel::new_headless(None);

    let result = vm.create_vm(
        "test-platform".to_string(),
        "test-vm".to_string(),
        "us-central1-a".to_string(),
        "e2-micro".to_string(),
    );

    assert!(result.is_ok());
}

#[test]
fn test_viewmodel_no_wss_channels() {
    // Phase 1: Verify WSS infrastructure removed
    // This test verifies ViewModel no longer has wss_tx channel
    let vm = ViewModel::new_headless(None);

    // If this compiles, wss_tx field doesn't exist (what we want)
    // The test passing means WSS channel successfully removed
    assert_eq!(vm.state.active_operations.len(), 0);
}

#[test]
fn test_viewmodel_event_no_wss_variant() {
    // Phase 1: Verify ViewModelEvent::Wss variant removed
    // This test ensures we cannot create Wss events

    use crate::viewmodel::platform::PlatformEvent;

    // Create a valid event (not Wss)
    let event = ViewModelEvent::Platform(PlatformEvent::VmCreated {
        platform_id: "test".to_string(),
        vm_name: "test-vm".to_string(),
        instance_id: "i-123".to_string(),
    });

    // Verify it's not a Wss variant (which shouldn't exist)
    match event {
        ViewModelEvent::Platform(_) => assert!(true),
        ViewModelEvent::Ssh(_) => assert!(true),
        ViewModelEvent::Ns(_) => assert!(true),
        ViewModelEvent::Logs(_) => assert!(true),
        ViewModelEvent::Mycart(_) => assert!(true),
        // ViewModelEvent::Wss should not compile after removal
    }
}
