//! NS actor for DNS management

mod actor;
mod commands;
mod events;

#[cfg(test)]
mod tests;

pub use actor::NsActor;
pub use commands::NsCommand;
pub use events::{DnsDomain, DnsProvider, DnsRecord, NsEvent};
