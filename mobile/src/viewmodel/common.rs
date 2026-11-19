//! Common types shared across ViewModel and actors

use crate::viewmodel::{logs, mycart, ns, platform, ssh, wss};
use crate::{dure_debug, dure_error, dure_info, dure_warn};

/// Unified event type from all actors
#[derive(Clone, Debug)]
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Drawer(platform::DrawerEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
    Logs(logs::LogEvent),
    Mycart(mycart::MycartEvent),
}
