//! ViewModel layer for actor-based MVVM architecture

pub mod common;
pub mod runtime;
pub mod io;
pub mod platform;
pub mod ssh;
pub mod ns;
pub mod wss;

#[cfg(test)]
mod tests;

pub use common::*;

use smol::channel::{Receiver, Sender};
use std::collections::{HashMap, VecDeque};

/// ViewModel coordinates actors and exposes unified API for UI/CLI
pub struct ViewModel {
    // Actor communication channels
    platform_tx: Sender<platform::PlatformCommand>,
    ssh_tx: Sender<ssh::SshCommand>,
    ns_tx: Sender<ns::NsCommand>,
    wss_tx: Sender<wss::WssCommand>,

    // Unified event receiver
    event_rx: Receiver<ViewModelEvent>,

    // Transient state
    state: ViewModelState,

    // Runtime handle
    runtime_handle: Option<RuntimeHandle>,

    // Optional egui context (for GUI mode)
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,
}

enum RuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::thread::JoinHandle<()>),

    #[cfg(target_arch = "wasm32")]
    Wasm(WasmExecutorHandle),
}

#[cfg(target_arch = "wasm32")]
struct WasmExecutorHandle;

#[derive(Default)]
pub struct ViewModelState {
    pub active_operations: HashMap<String, OperationProgress>,
    pub recent_errors: VecDeque<ErrorRecord>,
    pub wss_connections: HashMap<String, WssConnectionInfo>,
    #[cfg(feature = "gui")]
    pub textures: HashMap<String, egui::TextureHandle>,
}

pub struct OperationProgress {
    pub operation: String,
    pub progress: f32,
    pub status: String,
    pub started_at: std::time::Instant,
}

pub struct ErrorRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub error: String,
    pub actor: String,
}

#[derive(Clone, Debug)]
pub struct WssConnectionInfo {
    pub connection_id: String,
    pub url: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}
