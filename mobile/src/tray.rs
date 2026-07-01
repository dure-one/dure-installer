//! System tray interface for Dure (Desktop only)
//!
//! Provides a system tray icon with menu
//!
//! The tray icon runs on a separate thread to avoid blocking the main thread.
//! When "Show App" is clicked, it sends TrayExitAction::OpenGui via channel.
//!

// Allow unsafe code for platform-specific FFI (macOS CFRunLoop)
#![allow(unsafe_code)]

use anyhow::Result;
use crossbeam_queue::SegQueue;
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};

// Platform-specific imports for creating event loop on any thread
#[cfg(target_os = "linux")]
use tao::platform::unix::EventLoopBuilderExtUnix;

use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

/// Global queue for tray icon events (set up once at startup)
static TRAY_ICON_EVENTS: OnceLock<Arc<SegQueue<TrayIconEvent>>> = OnceLock::new();
/// Global queue for menu events (set up once at startup)
static MENU_EVENTS: OnceLock<Arc<SegQueue<MenuEvent>>> = OnceLock::new();

/// Action to take after tray mode exits
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayExitAction {
    Quit,
    OpenGui,
}

/// User events for the event loop
#[derive(Debug)]
enum UserEvent {
    #[allow(dead_code)]
    TrayIconEvent(TrayIconEvent),
    #[allow(dead_code)]
    MenuEvent(MenuEvent),
}

/// Initialize global event handlers (call once at startup)
pub fn init_tray_event_handlers() {
    log::info!("Initializing global tray event handlers (one-time setup)");

    // Initialize queues
    let tray_queue = Arc::new(SegQueue::new());
    let menu_queue = Arc::new(SegQueue::new());

    TRAY_ICON_EVENTS.get_or_init(|| tray_queue.clone());
    MENU_EVENTS.get_or_init(|| menu_queue.clone());

    // Set up global event handlers that push to queues
    let tray_queue_for_handler = tray_queue.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        log::debug!(
            "TrayIconEvent received, pushing to global queue: {:?}",
            event
        );
        tray_queue_for_handler.push(event);
    }));

    let menu_queue_for_handler = menu_queue.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        log::info!(
            ">>> MenuEvent handler fired, pushing to global queue: {:?}",
            event
        );
        menu_queue_for_handler.push(event);
    }));

    log::info!("Global tray event handlers initialized successfully");
}

/// Menu item identifiers
struct MenuItems {
    show_app: MenuId,
    quit: MenuId,
}

/// Load the embedded icon
fn load_icon() -> Icon {
    let start_time = std::time::Instant::now();
    log::debug!("Loading tray icon...");

    let icon_bytes = include_bytes!("../app/src/main/play_store_512.png");

    let t0 = std::time::Instant::now();
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon");
    log::debug!("  image::load_from_memory(): {:?}", t0.elapsed());

    let t1 = std::time::Instant::now();
    let rgba = image.to_rgba8();
    log::debug!("  to_rgba8(): {:?}", t1.elapsed());

    let t2 = std::time::Instant::now();
    let icon = Icon::from_rgba(rgba.to_vec(), image.width(), image.height())
        .expect("Failed to create icon");
    log::debug!("  Icon::from_rgba(): {:?}", t2.elapsed());

    log::debug!("Icon loaded in {:?}", start_time.elapsed());
    icon
}

/// Create the tray menu
fn create_tray_menu() -> (Menu, MenuItems) {
    let start_time = std::time::Instant::now();
    log::debug!("=== Creating tray menu ===");

    let menu = Menu::new();
    let show_app = MenuItem::new("Show App", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let menu_items = MenuItems {
        show_app: show_app.id().clone(),
        quit: quit.id().clone(),
    };

    menu.append(&show_app).ok();
    menu.append(&quit).ok();

    log::debug!("Menu created in {:?}", start_time.elapsed());
    (menu, menu_items)
}

/// Handle for the tray thread
pub struct TrayHandle {
    thread: Option<JoinHandle<()>>,
    action_receiver: std::sync::mpsc::Receiver<TrayExitAction>,
}

impl TrayHandle {
    /// Check if a tray action has been received (non-blocking)
    pub fn try_recv_action(&self) -> Option<TrayExitAction> {
        self.action_receiver.try_recv().ok()
    }

    /// Wait for a tray action (blocking)
    pub fn recv_action(&self) -> Option<TrayExitAction> {
        self.action_receiver.recv().ok()
    }

    /// Stop the tray and wait for the thread to finish
    pub fn join(mut self) -> Result<()> {
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| anyhow::anyhow!("Tray thread panicked"))?;
        }
        Ok(())
    }
}

/// Run the system tray mode on a separate thread
/// Returns a handle that can be used to check for actions or stop the tray
pub fn run_tray_mode() -> Result<TrayHandle> {
    log::info!("=== Starting tray mode on separate thread ===");

    let (action_sender, action_receiver) = std::sync::mpsc::channel();

    let thread = thread::Builder::new()
        .name("tray-event-loop".to_string())
        .spawn(move || {
            tray_thread_main(action_sender);
        })?;

    Ok(TrayHandle {
        thread: Some(thread),
        action_receiver,
    })
}

/// Main function for the tray thread
fn tray_thread_main(action_sender: std::sync::mpsc::Sender<TrayExitAction>) {
    log::info!("=== Tray thread started ===");

    // Create event loop
    log::info!("Creating new event loop on tray thread");

    // On Linux, we need to use new_any_thread to create the event loop on a non-main thread
    #[cfg(target_os = "linux")]
    let event_loop = {
        let mut builder = EventLoopBuilder::<UserEvent>::with_user_event();
        builder.with_any_thread(true);
        builder.build()
    };

    // On other platforms, use the standard builder
    #[cfg(not(target_os = "linux"))]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    log::info!("Event loop created successfully");

    // Get references to global event queues
    let tray_queue = TRAY_ICON_EVENTS
        .get()
        .expect("Tray event handlers not initialized! Call init_tray_event_handlers() first");
    let menu_queue = MENU_EVENTS
        .get()
        .expect("Menu event handlers not initialized! Call init_tray_event_handlers() first");
    log::info!("Got references to global event queues");

    // Variables to be captured by the event loop
    let mut tray_icon: Option<TrayIcon> = None;
    let mut menu_items: Option<MenuItems> = None;

    // Clone Arc references for the closure
    let tray_queue = tray_queue.clone();
    let menu_queue = menu_queue.clone();

    // Run event loop
    log::info!("Starting tray event loop...");
    event_loop.run(move |event, _, control_flow| {
        // Use Poll mode to ensure system events are processed immediately
        // We'll manually sleep if there are no events to keep CPU usage low
        *control_flow = ControlFlow::Poll;

        // Process events from global queues
        while let Some(tray_event) = tray_queue.pop() {
            log::debug!("Processing TrayIconEvent from queue: {:?}", tray_event);
        }

        while let Some(menu_event) = menu_queue.pop() {
            log::info!(">>> Menu event received from queue: {:?}", menu_event);

            if let Some(ref items) = menu_items {
                log::debug!("Menu items available, checking which item was clicked");
                if menu_event.id == items.show_app {
                    log::info!(">>> 'Show App' menu item clicked!");
                    log::info!("Sending OpenGui action to main thread");
                    // Send action but keep tray running
                    if let Err(e) = action_sender.send(TrayExitAction::OpenGui) {
                        log::error!("Failed to send OpenGui action: {}", e);
                    }
                    log::info!("OpenGui action sent, tray continues running");
                    continue; // Skip further processing
                } else if menu_event.id == items.quit {
                    // Quit application - exit the event loop
                    log::info!("Quitting application");
                    tray_icon.take(); // Drop tray icon
                    // Send Quit action before exiting
                    if let Err(e) = action_sender.send(TrayExitAction::Quit) {
                        log::error!("Failed to send Quit action: {}", e);
                    }
                    *control_flow = ControlFlow::Exit;
                }
            }
        }

        if let Event::NewEvents(tao::event::StartCause::Init) = event {
            let init_start = std::time::Instant::now();
            log::info!("=== Event loop Init event - creating tray icon ===");

            let t0 = std::time::Instant::now();
            let icon = load_icon();
            log::debug!("  load_icon(): {:?}", t0.elapsed());

            let t1 = std::time::Instant::now();
            let (tray_menu, items) = create_tray_menu();
            log::debug!("  create_tray_menu(): {:?}", t1.elapsed());

            let t2 = std::time::Instant::now();
            tray_icon = Some(
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_tooltip("Dure")
                    .with_icon(icon)
                    .build()
                    .expect("Failed to create tray icon"),
            );
            log::debug!("  TrayIconBuilder.build(): {:?}", t2.elapsed());

            menu_items = Some(items);
            log::info!("=== Tray icon created in {:?} ===", init_start.elapsed());

            // Wake up macOS run loop if needed
            #[cfg(target_os = "macos")]
            unsafe {
                use objc2_core_foundation::CFRunLoop;
                if let Some(rl) = CFRunLoop::main() {
                    rl.wake_up();
                }
            }
        }

        // In Poll mode, sleep briefly if we're not exiting to avoid high CPU usage
        // This gives time for system events to be delivered while keeping responsiveness
        if *control_flow != ControlFlow::Exit {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Note: event_loop.run() never returns - it runs until the process exits.
    // When Quit is clicked, the event loop exits via ControlFlow::Exit,
    // which terminates the thread. The action_sender is dropped, closing the channel.
}
