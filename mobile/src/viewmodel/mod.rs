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

impl ViewModel {
    /// Create ViewModel for GUI mode (Desktop/Android)
    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    pub fn new(ctx: egui::Context) -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Spawn background thread with smol executor
        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started");

                // Create actors
                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());

                // Run all actors concurrently
                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();

                // Keep thread alive
                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            egui_ctx: Some(ctx),
        }
    }

    /// Create ViewModel for CLI mode (headless)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_headless() -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started (headless)");

                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());

                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();

                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            #[cfg(feature = "gui")]
            egui_ctx: None,
        }
    }

    /// Poll for events and update state (GUI mode)
    #[cfg(feature = "gui")]
    pub fn poll_events(&mut self, ctx: &egui::Context) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(&event, Some(ctx));
            events.push(event);
        }

        if !events.is_empty() {
            ctx.request_repaint();
        }

        events
    }

    /// Poll events without egui context (CLI mode)
    pub fn poll_events_headless(&mut self) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            #[cfg(feature = "gui")]
            self.apply_event(&event, None);
            #[cfg(not(feature = "gui"))]
            {
                // Minimal event processing for headless mode
                let _ = event; // Suppress unused variable warning
            }
            events.push(event);
        }

        events
    }

    #[cfg(feature = "gui")]
    fn apply_event(&mut self, _event: &ViewModelEvent, _ctx: Option<&egui::Context>) {
        // TODO: implement in Week 2
    }

    // State accessors
    pub fn active_operations(&self) -> &HashMap<String, OperationProgress> {
        &self.state.active_operations
    }

    pub fn recent_errors(&self) -> &VecDeque<ErrorRecord> {
        &self.state.recent_errors
    }
}
