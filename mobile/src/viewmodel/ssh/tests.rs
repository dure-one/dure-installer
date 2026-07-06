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
}
