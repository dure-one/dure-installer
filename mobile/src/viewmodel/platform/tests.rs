#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;
    use futures::select;
    use futures::FutureExt;

    #[test]
    fn test_platform_actor_list_vms_sends_event() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = super::super::PlatformActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            // Send command
            cmd_tx
                .send(super::super::PlatformCommand::ListVMs {
                    platform_name: "test-platform".to_string(),
                })
                .await
                .unwrap();

            // Should receive event (or error), ignoring Progress events
            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            loop {
                select! {
                    event = event_rx.recv().fuse() => {
                        let event = event.unwrap();
                        match event {
                            ViewModelEvent::Platform(super::super::PlatformEvent::Progress { .. }) => {
                                // Skip progress events, keep waiting
                                continue;
                            }
                            ViewModelEvent::Platform(super::super::PlatformEvent::VMsListed { .. }) |
                            ViewModelEvent::Platform(super::super::PlatformEvent::Error { .. }) => {
                                // Success - received expected event type
                                break;
                            }
                            other => panic!("Unexpected event: {:?}", other),
                        }
                    }
                    _ = timeout.as_mut().fuse() => {
                        panic!("Test timed out waiting for event");
                    }
                }
            }
        });
    }

    #[test]
    fn test_vm_status_exists() {
        use super::super::VmStatus;

        let status = VmStatus {
            exists: true,
            name: Some("test-vm".to_string()),
            zone: Some("us-central1-a".to_string()),
            external_ip: Some("1.2.3.4".to_string()),
            status: Some("RUNNING".to_string()),
        };

        assert!(status.exists);
        assert_eq!(status.name, Some("test-vm".to_string()));
        assert_eq!(status.external_ip, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn test_vm_status_no_vms() {
        use super::super::VmStatus;

        let status = VmStatus {
            exists: false,
            name: None,
            zone: None,
            external_ip: None,
            status: None,
        };

        assert!(!status.exists);
        assert!(status.name.is_none());
    }

    #[test]
    fn test_firewall_status_whitelisted() {
        use super::super::FirewallStatus;

        let status = FirewallStatus {
            whitelisted: true,
            current_ip: Some("1.2.3.4".to_string()),
        };

        assert!(status.whitelisted);
        assert_eq!(status.current_ip, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn test_firewall_status_not_whitelisted() {
        use super::super::FirewallStatus;

        let status = FirewallStatus {
            whitelisted: false,
            current_ip: Some("5.6.7.8".to_string()),
        };

        assert!(!status.whitelisted);
        assert_eq!(status.current_ip, Some("5.6.7.8".to_string()));
    }

    #[test]
    fn test_ssh_status_no_external_ip() {
        use super::super::SshStatus;

        let status = SshStatus {
            connected: false,
            error: Some("No external IP configured".to_string()),
        };

        assert!(!status.connected);
        assert!(status.error.is_some());
        assert_eq!(status.error.unwrap(), "No external IP configured");
    }

    #[test]
    fn test_ssh_status_no_key() {
        use super::super::SshStatus;

        let status = SshStatus {
            connected: false,
            error: Some("SSH key not found in keyring".to_string()),
        };

        assert!(!status.connected);
        assert!(status.error.is_some());
    }

    #[test]
    fn test_ssh_status_connected() {
        use super::super::SshStatus;

        let status = SshStatus {
            connected: true,
            error: None,
        };

        assert!(status.connected);
        assert!(status.error.is_none());
    }
}
