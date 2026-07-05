//! NS actor for DNS management

mod commands;
mod events;
mod actor;

#[cfg(test)]
mod tests;

pub use commands::NsCommand;
pub use events::{NsEvent, DnsProvider, DnsDomain, DnsRecord};
pub use actor::NsActor;
