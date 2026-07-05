#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;

    #[test]
    fn test_platform_actor_list_vms_sends_event() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = PlatformActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            // Send command
            cmd_tx.send(PlatformCommand::ListVMs {
                platform_name: "test-platform".to_string()
            }).await.unwrap();

            // Should receive event (or error)
            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            smol::select! {
                event = event_rx.recv() => {
                    let event = event.unwrap();
                    match event {
                        ViewModelEvent::Platform(PlatformEvent::VMsListed { .. }) |
                        ViewModelEvent::Platform(PlatformEvent::Error { .. }) => {
                            // Success - received expected event type
                        }
                        _ => panic!("Unexpected event: {:?}", event),
                    }
                }
                _ = &mut timeout => {
                    panic!("Test timed out waiting for event");
                }
            }
        });
    }
}
