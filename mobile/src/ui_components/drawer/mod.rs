//! Shared drawer components for Platform, SSH, and future tabs

mod logs_renderer;
mod operations_renderer;
mod status_utils;
mod tab_bar;

pub use logs_renderer::LogsRenderer;
pub use operations_renderer::OperationsRenderer;
pub use status_utils::{format_elapsed, staleness_color, StatusLine};
pub use tab_bar::{DrawerTabTrait, TabBar};
