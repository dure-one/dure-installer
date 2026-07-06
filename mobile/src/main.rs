//! Desktop entry point for Dure
//!
//! Mode selection logic:
//! - `--gui` flag: Runs GUI mode
//! - `--tray` flag: Runs tray mode
//! - `--silent` flag: Performs silent installation and exits (desktop only)
//! - `--uninstall` flag: Performs uninstallation and exits (desktop only)
//! - Terminal detected: Runs CLI mode
//! - Default (double-click): Runs tray mode
//!
//! Logging options:
//! - `--log [FILE]`: Enable file logging (defaults to dure.log if FILE not specified)

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![cfg(not(target_os = "android"))]

// On WASM the lib entry point (wasm_start) is used; provide a no-op main so the
// bin target still compiles.
#[cfg(target_arch = "wasm32")]
fn main() {
    dure::main_wasm::wasm_start();
}

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(all(not(target_arch = "wasm32"), feature = "gui"))]
use dure::dure::DureApp;
#[cfg(not(target_arch = "wasm32"))]
use std::io::IsTerminal;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    // Parse command-line arguments FIRST to check for explicit mode flags and log configuration
    let args: Vec<String> = std::env::args().collect();

    // Check for --log argument
    let log_file = {
        let mut log_file_opt: Option<String> = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--log" {
                // Check if there's a value after --log
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    log_file_opt = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    // --log flag present but no value, use default
                    log_file_opt = Some("dure.log".to_string());
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        log_file_opt
    };

    // Initialize logger
    if let Some(log_path) = &log_file {
        // File-based logging
        let log_path = if log_path.is_empty() {
            "dure.log"
        } else {
            log_path.as_str()
        };

        let target = Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to open log file '{}': {}", log_path, e);
                    std::process::exit(1);
                }),
        );

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .target(env_logger::Target::Pipe(target))
            .init();
    } else {
        // Console logging (default)
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    log::info!("Dure v{} starting...", env!("CARGO_PKG_VERSION"));

    // Initialize application configuration and database
    let config = dure::Config::new().unwrap_or_else(|e| {
        log::error!("Failed to initialize application config: {}", e);
        std::process::exit(1);
    });
    let db_path = config.data_dir.join("dure.db");
    dure::calc::db::set_db_path(db_path.to_string_lossy().to_string());
    log::info!("Database path set to: {}", db_path.display());

    // Verify binary attestation (if not disabled)
    // if !args.iter().any(|arg| arg == "--skip-attestation") {
    //     match std::env::current_exe() {
    //         Ok(exe_path) => {
    //             log::info!("Verifying binary attestation for: {}", exe_path.display());
    //             match dure::attestation::verify_current_binary(
    //                 exe_path.to_str().unwrap_or(""),
    //                 "nikescar",  // TODO: Replace with actual org name
    //                 "dure",
    //             ) {
    //                 Ok(result) => {
    //                     log::info!("✓ Binary attestation verified successfully");
    //                     log::info!("  Digest: {}", result.digest);
    //                     log::info!("  Repository: {}", result.repository);
    //                     log::info!("  Attestations: {}", result.attestation_count);
    //                 }
    //                 Err(e) => {
    //                     log::warn!("⚠ Binary attestation verification failed: {}", e);
    //                     log::warn!("  This may indicate the binary was not released through official channels");
    //                     // Continue execution but log the warning
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             log::warn!("Failed to get current executable path: {}", e);
    //         }
    //     }
    // }

    // Initialize i18n EARLY (before any mode starts)
    if let Err(e) = dure::i18n::init_i18n("Auto") {
        log::error!("Failed to initialize i18n: {}", e);
    }

    // Initialize global tray event handlers (one-time setup)
    #[cfg(all(
        feature = "gui",
        not(any(target_os = "android", target_arch = "wasm32")),
        not(target_os = "openbsd")
    ))]
    dure::tray::init_tray_event_handlers();

    let force_gui = args.iter().any(|arg| arg == "--gui");
    let force_tray = args.iter().any(|arg| arg == "--tray");
    let silent_install = args.iter().any(|arg| arg == "--silent");
    let uninstall = args.iter().any(|arg| arg == "--uninstall");

    // Check if CLI arguments are provided (dns, info, init, etc.)
    let has_cli_command =
        args.len() > 1 && !force_gui && !force_tray && !silent_install && !uninstall;

    // Handle uninstall mode (desktop only)
    if uninstall {
        log::info!("Uninstall mode not implemented yet");
        println!("Uninstall mode not implemented yet");
        std::process::exit(0);
    }

    // Handle silent install mode (desktop only)
    if silent_install {
        log::info!("Silent install mode not implemented yet");
        println!("Silent install mode not implemented yet");
        std::process::exit(0);
    }

    if has_cli_command {
        // CLI mode (arguments provided)
        log::info!("Running in CLI mode (arguments detected)");
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        dure::cli::run_cli_mode()?;
    } else if force_gui {
        #[cfg(feature = "gui")]
        {
            // GUI mode (explicitly requested via --gui flag)
            log::info!("Running in GUI mode (--gui flag)");

            // Hide console window on Windows
            #[cfg(target_os = "windows")]
            windows_installer::hide_console();

            run_gui_mode()?;
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("Error: GUI support not compiled in this build.");
            eprintln!("Rebuild with: cargo build --features gui");
            std::process::exit(1);
        }
    } else if force_tray {
        #[cfg(all(
            feature = "gui",
            not(any(target_os = "android", target_arch = "wasm32")),
            not(target_os = "openbsd")
        ))]
        {
            // Tray mode (explicitly requested via --tray flag)
            log::info!("Running in tray mode (--tray flag)");

            // Hide console window on Windows
            #[cfg(target_os = "windows")]
            windows_installer::hide_console();

            // Start tray mode on separate thread
            log::info!("*** Starting tray mode on separate thread ***");
            let tray_handle = dure::tray::run_tray_mode()?;

            // Wait for tray actions
            loop {
                log::info!("*** Waiting for tray action ***");
                match tray_handle.recv_action() {
                    Some(dure::tray::TrayExitAction::Quit) => {
                        log::info!("*** Received Quit action, exiting application ***");
                        break;
                    }
                    Some(dure::tray::TrayExitAction::OpenGui) => {
                        log::info!("*** Received OpenGui action, opening GUI window ***");
                        log::info!("*** (Tray will continue running in background) ***");
                        run_gui_mode()?;
                        log::info!("*** GUI closed ***");
                    }
                    None => {
                        log::warn!("*** Tray thread ended unexpectedly ***");
                        break;
                    }
                }
            }

            log::info!("*** Joining tray thread ***");
            tray_handle.join()?;
        }
        #[cfg(all(feature = "gui", target_os = "openbsd"))]
        {
            // Tray mode not available on OpenBSD - run GUI mode instead
            log::info!("Tray mode not available on OpenBSD (--tray flag), running GUI mode");
            run_gui_mode()?;
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("Error: GUI/Tray support not compiled in this build.");
            eprintln!("Rebuild with: cargo build --features gui");
            std::process::exit(1);
        }
    } else if std::io::stdout().is_terminal() {
        // Terminal mode - run CLI interface
        log::info!("Running in CLI mode (terminal detected)");

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        dure::cli::run_cli_mode()?;
    } else {
        #[cfg(all(
            feature = "gui",
            not(any(target_os = "android", target_arch = "wasm32")),
            not(target_os = "openbsd")
        ))]
        {
            // Tray mode (default for double-click on Windows - no terminal, no flags)
            log::info!("Running in tray mode (default - no terminal detected)");

            // Hide console window on Windows
            #[cfg(target_os = "windows")]
            windows_installer::hide_console();

            // Start tray mode on separate thread
            log::info!("*** Starting tray mode on separate thread ***");
            let tray_handle = dure::tray::run_tray_mode()?;

            // Wait for tray actions
            loop {
                log::info!("*** Waiting for tray action ***");
                match tray_handle.recv_action() {
                    Some(dure::tray::TrayExitAction::Quit) => {
                        log::info!("*** Received Quit action, exiting application ***");
                        break;
                    }
                    Some(dure::tray::TrayExitAction::OpenGui) => {
                        log::info!("*** Received OpenGui action, opening GUI window ***");
                        log::info!("*** (Tray will continue running in background) ***");
                        run_gui_mode()?;
                        log::info!("*** GUI closed ***");
                    }
                    None => {
                        log::warn!("*** Tray thread ended unexpectedly ***");
                        break;
                    }
                }
            }

            log::info!("*** Joining tray thread ***");
            tray_handle.join()?;
        }
        #[cfg(all(feature = "gui", target_os = "openbsd"))]
        {
            // Tray mode not available on OpenBSD - run GUI mode instead
            log::info!("Tray mode not available on OpenBSD, running GUI mode");
            run_gui_mode()?;
        }
        #[cfg(not(feature = "gui"))]
        {
            // No terminal, no GUI - fall back to CLI mode
            log::warn!("No terminal detected and GUI not compiled in - falling back to CLI mode");
            eprintln!("Warning: Running in CLI mode (GUI not available)");
            eprintln!("For GUI support, rebuild with: cargo build --features gui");
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            dure::cli::run_cli_mode()?;
        }
    }

    Ok(())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "gui"))]
fn run_gui_mode() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Dure")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../app/src/main/play_store_512.png"
                ))
                .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Dure",
        options,
        Box::new(|cc| {
            log::info!("Creating Dure app instance");

            // Load Material3 theme system
            use egui_material3::theme::{
                load_fonts, load_themes, setup_local_fonts_from_bytes, setup_local_theme,
            };

            // Setup theme from file FIRST (before fonts)
            setup_local_theme(Some("resources/material-theme-lightblue.json"));

            // Prepare local fonts including Material Symbols (using include_bytes!)
            setup_local_fonts_from_bytes(
                "MaterialSymbolsOutlined",
                include_bytes!("../resources/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf"),
            );
            setup_local_fonts_from_bytes(
                "NotoSansKr",
                include_bytes!("../resources/noto-sans-kr.ttf"),
            );

            // Install image loaders
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // Load fonts and themes
            load_fonts(&cc.egui_ctx);
            load_themes();

            // Update window background with theme colors
            use egui_material3::theme::update_window_background;
            update_window_background(&cc.egui_ctx);

            Ok(Box::<DureApp>::default())
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}
