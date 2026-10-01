//! Shared drawer components for Platform, SSH, and future tabs

mod tab_bar;
mod status_utils;

pub use tab_bar::{DrawerTabTrait, TabBar};
pub use status_utils::{StatusLine, format_elapsed, staleness_color};
