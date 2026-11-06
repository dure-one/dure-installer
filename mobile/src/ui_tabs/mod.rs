//! Tab modules for the main application UI

// Infrastructure management tabs (active on all platforms)
pub mod ns; // Keep existing NS tab code, but hidden from main tabs
pub mod dns; // New DNS tab with Cloudflare OAuth
pub mod dns_drawer; // DNS drawer with tabs
pub mod platform;
pub mod platform_drawer;
pub mod site;
pub mod ssh;
pub mod ssh_drawer;

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
    Dns, // New DNS tab with Cloudflare OAuth workflow
    Site,
    // Ns tab kept in code but hidden from main tab list
}

impl Tab {
    /// Get the display name for the tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Platform => "Platform",
            Tab::Ssh => "SSH",
            Tab::Dns => "DNS",
            Tab::Site => "Site",
        }
    }

    /// Get all tabs in order (NS tab excluded - kept in code for reference)
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Platform,
            Tab::Ssh,
            Tab::Dns,
            Tab::Site,
        ]
    }
}
