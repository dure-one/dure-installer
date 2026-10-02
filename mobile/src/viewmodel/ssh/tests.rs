#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;
    use futures::select;
    use futures::FutureExt;

    #[test]
    fn test_ssh_actor_list_hosts() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = super::super::SshActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            cmd_tx.send(super::super::SshCommand::ListHosts).await.unwrap();

            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            // Loop until we get a non-Progress event
            loop {
                select! {
                    event = event_rx.recv().fuse() => {
                        match event.unwrap() {
                            ViewModelEvent::Ssh(super::super::SshEvent::Progress { .. }) => {
                                // Skip progress events, keep waiting
                                continue;
                            }
                            ViewModelEvent::Ssh(super::super::SshEvent::HostsListed { .. }) |
                            ViewModelEvent::Ssh(super::super::SshEvent::Error { .. }) => {
                                // Test passed
                                break;
                            }
                            other => panic!("Unexpected event: {:?}", other),
                        }
                    }
                    _ = timeout.as_mut().fuse() => panic!("Timeout"),
                }
            }
        });
    }

    #[test]
    fn test_operation_commands_exist() {
        // Verify all operation commands compile and can be constructed
        let _cmd1 = super::super::SshCommand::Refresh {
            host: "test-host".to_string(),
        };
        let _cmd2 = super::super::SshCommand::SshCheck {
            host: "test-host".to_string(),
        };
        let _cmd3 = super::super::SshCommand::CheckBase {
            host: "test-host".to_string(),
        };
        let _cmd4 = super::super::SshCommand::InstallBase {
            host: "test-host".to_string(),
        };
        let _cmd5 = super::super::SshCommand::CheckDocker {
            host: "test-host".to_string(),
        };
        let _cmd6 = super::super::SshCommand::InstallDockerDaemon {
            host: "test-host".to_string(),
        };
        let _cmd7 = super::super::SshCommand::RemoveDocker {
            host: "test-host".to_string(),
        };
        let _cmd8 = super::super::SshCommand::CheckDure {
            host: "test-host".to_string(),
        };
        let _cmd9 = super::super::SshCommand::InstallDure {
            host: "test-host".to_string(),
            env_config: std::collections::HashMap::new(),
        };
        let _cmd10 = super::super::SshCommand::RemoveDure {
            host: "test-host".to_string(),
        };
    }

    #[test]
    fn test_operation_events_exist() {
        // Verify all operation events compile and can be constructed
        let _ev1 = super::super::SshEvent::RefreshCompleted {
            host: "test-host".to_string(),
            ssh_connected: false,
            base_installed: false,
            docker_installed: false,
            dure_installed: false,
        };
        let _ev2 = super::super::SshEvent::SshCheckCompleted {
            host: "test-host".to_string(),
            connected: false,
            error: None,
        };
        let _ev3 = super::super::SshEvent::BaseCheckCompleted {
            host: "test-host".to_string(),
            installed: false,
            missing_packages: vec![],
        };
        let _ev4 = super::super::SshEvent::BaseInstallCompleted {
            host: "test-host".to_string(),
            success: false,
            error: None,
        };
        let _ev5 = super::super::SshEvent::DockerCheckCompleted {
            host: "test-host".to_string(),
            installed: false,
            version: None,
        };
        let _ev6 = super::super::SshEvent::DockerInstallCompleted {
            host: "test-host".to_string(),
            success: false,
            error: None,
        };
        let _ev7 = super::super::SshEvent::DockerRemoveCompleted {
            host: "test-host".to_string(),
            success: false,
            error: None,
        };
        let _ev8 = super::super::SshEvent::DureCheckCompleted {
            host: "test-host".to_string(),
            installed: false,
            running: false,
            services: vec![],
        };
        let _ev9 = super::super::SshEvent::DureInstallCompleted {
            host: "test-host".to_string(),
            success: false,
            error: None,
        };
        let _ev10 = super::super::SshEvent::DureRemoveCompleted {
            host: "test-host".to_string(),
            success: false,
            error: None,
        };
        let _ev11 = super::super::SshEvent::OperationFailed {
            host: "test-host".to_string(),
            operation: "test-op".to_string(),
            error: "test-error".to_string(),
        };
    }

    #[test]
    fn test_operation_commands_debug() {
        let cmd = super::super::SshCommand::Refresh {
            host: "example.com".to_string(),
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Refresh"));
        assert!(debug_str.contains("example.com"));
    }

    #[test]
    fn test_operation_events_debug() {
        let event = super::super::SshEvent::RefreshCompleted {
            host: "example.com".to_string(),
            ssh_connected: true,
            base_installed: true,
            docker_installed: true,
            dure_installed: true,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RefreshCompleted"));
        assert!(debug_str.contains("example.com"));
    }
}
