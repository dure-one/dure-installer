#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;

    #[test]
    fn test_ns_actor_list_providers() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = NsActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            cmd_tx.send(NsCommand::ListProviders).await.unwrap();

            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            smol::select! {
                event = event_rx.recv() => {
                    match event.unwrap() {
                        ViewModelEvent::Ns(NsEvent::ProvidersListed { .. }) |
                        ViewModelEvent::Ns(NsEvent::Error { .. }) => {}
                        _ => panic!("Unexpected event"),
                    }
                }
                _ = &mut timeout => panic!("Timeout"),
            }
        });
    }
}
