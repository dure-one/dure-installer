#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;

    #[test]
    fn test_ssh_actor_list_hosts() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            let actor = SshActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();

            cmd_tx.send(SshCommand::ListHosts).await.unwrap();

            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);

            smol::select! {
                event = event_rx.recv() => {
                    match event.unwrap() {
                        ViewModelEvent::Ssh(SshEvent::HostsListed { .. }) |
                        ViewModelEvent::Ssh(SshEvent::Error { .. }) => {}
                        _ => panic!("Unexpected event"),
                    }
                }
                _ = &mut timeout => panic!("Timeout"),
            }
        });
    }
}
