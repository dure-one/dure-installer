#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;
    use futures::select;
    use futures::FutureExt;

    #[test]
    fn test_ns_actor_list_providers() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = super::super::NsActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            cmd_tx.send(super::super::NsCommand::ListProviders).await.unwrap();

            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            // Loop until we get a non-Progress event
            loop {
                select! {
                    event = event_rx.recv().fuse() => {
                        match event.unwrap() {
                            ViewModelEvent::Ns(super::super::NsEvent::Progress { .. }) => {
                                // Skip progress events, keep waiting
                                continue;
                            }
                            ViewModelEvent::Ns(super::super::NsEvent::ProvidersListed { .. }) |
                            ViewModelEvent::Ns(super::super::NsEvent::Error { .. }) => {
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
}
