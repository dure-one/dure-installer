//! Common types shared across ViewModel and actors

use crate::viewmodel::{deltachat, ns, platform, ssh, wss};

/// Unified event type from all actors
#[derive(Clone, Debug)]
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
    DeltaChat(deltachat::DeltaChatEvent),
}
