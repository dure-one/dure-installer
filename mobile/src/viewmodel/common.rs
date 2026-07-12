//! Common types shared across ViewModel and actors

use crate::viewmodel::{ns, platform, ssh, wss};
#[cfg(not(target_os = "openbsd"))]
use crate::viewmodel::deltachat;

/// Unified event type from all actors
#[derive(Clone, Debug)]
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
    #[cfg(not(target_os = "openbsd"))]
    DeltaChat(deltachat::DeltaChatEvent),
}
