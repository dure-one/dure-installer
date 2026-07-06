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
}
