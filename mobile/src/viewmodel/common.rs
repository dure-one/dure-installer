//! Common types shared across ViewModel and actors

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::viewmodel::{ns, platform, ssh, wss};

/// Unified event type from all actors
#[derive(Clone, Debug)]
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
}
