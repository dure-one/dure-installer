//! Tab modules for the main application UI

// Infrastructure management tabs (active on all platforms)
pub mod ns;
pub mod platform;
pub mod site;
pub mod ssh;

// E-commerce tabs (disabled)
// pub mod channel;
// pub mod client;
// pub mod dm;
// pub mod email;
// pub mod members;
// pub mod orders;
// pub mod products;
// pub mod roles;

/// Enum representing all available tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Tab {
    Platform,
    Ssh,
    Ns,
    Site,
}

impl Tab {
    /// Get the display name for the tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Platform => "Platform",
            Tab::Ssh => "SSH",
            Tab::Ns => "Nameserver",
            Tab::Site => "Site",
        }
    }

    /// Get all tabs in order
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Platform,
            Tab::Ssh,
            Tab::Ns,
            Tab::Site,
        ]
    }
}
