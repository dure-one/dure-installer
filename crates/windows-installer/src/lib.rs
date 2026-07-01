//! Windows installer utilities for Dure
//!
//! Provides Windows-specific installation operations that require COM APIs.
//! This crate uses unsafe code for COM interop.

use std::path::PathBuf;

/// Create a Windows shortcut (.lnk file)
///
/// # Arguments
/// * `target` - Path to the executable
/// * `shortcut_path` - Path where the shortcut should be created
/// * `args` - Command-line arguments for the shortcut
/// * `description` - Description text for the shortcut
///
/// # Platform
/// This function is only available on Windows and will panic on other platforms.
#[cfg(target_os = "windows")]
pub fn create_shortcut(
    target: &PathBuf,
    shortcut_path: &PathBuf,
    args: &str,
    description: &str,
) -> Result<(), String> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize, IPersistFile,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{Interface, PCWSTR};

    log::debug!("Creating Windows shortcut:");
    log::debug!("  Target: {}", target.display());
    log::debug!("  Shortcut path: {}", shortcut_path.display());

    // Check if target exists
    if !target.exists() {
        let err_msg = format!("Target binary does not exist: {}", target.display());
        log::error!("{}", err_msg);
        return Err(err_msg);
    }

    let working_dir = target
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;

    log::debug!("  Working directory: {}", working_dir.display());

    // Convert paths to wide strings
    let target_wide: Vec<u16> = target
        .display()
        .to_string()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let shortcut_wide: Vec<u16> = shortcut_path
        .display()
        .to_string()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let workdir_wide: Vec<u16> = working_dir
        .display()
        .to_string()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let args_wide: Vec<u16> = args.encode_utf16().chain(Some(0)).collect();
    let desc_wide: Vec<u16> = description.encode_utf16().chain(Some(0)).collect();

    unsafe {
        // Initialize COM
        log::debug!("Initializing COM...");
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        // Check if initialization failed (but allow RPC_E_CHANGED_MODE which means COM already initialized)
        if hr.is_err() {
            let hr_code = hr.0;
            // 0x80010106 = RPC_E_CHANGED_MODE, means COM already initialized with different mode (okay)
            if hr_code != 0x80010106u32 as i32 {
                let err_msg = format!("Failed to initialize COM: HRESULT 0x{:08X}", hr_code as u32);
                log::error!("{}", err_msg);
                return Err(err_msg);
            }
            log::debug!("COM already initialized (RPC_E_CHANGED_MODE)");
        }

        let result = (|| -> Result<(), String> {
            // Create ShellLink object
            log::debug!("Creating IShellLink instance...");
            let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("Failed to create IShellLink: {:?}", e))?;

            // Set target path
            log::debug!("Setting target path...");
            shell_link
                .SetPath(PCWSTR(target_wide.as_ptr()))
                .map_err(|e| format!("Failed to set target path: {:?}", e))?;

            // Set arguments
            log::debug!("Setting arguments...");
            shell_link
                .SetArguments(PCWSTR(args_wide.as_ptr()))
                .map_err(|e| format!("Failed to set arguments: {:?}", e))?;

            // Set working directory
            log::debug!("Setting working directory...");
            shell_link
                .SetWorkingDirectory(PCWSTR(workdir_wide.as_ptr()))
                .map_err(|e| format!("Failed to set working directory: {:?}", e))?;

            // Set description
            log::debug!("Setting description...");
            shell_link
                .SetDescription(PCWSTR(desc_wide.as_ptr()))
                .map_err(|e| format!("Failed to set description: {:?}", e))?;

            // Save the shortcut
            log::debug!("Saving shortcut...");
            let persist_file: IPersistFile = shell_link
                .cast()
                .map_err(|e| format!("Failed to get IPersistFile interface: {:?}", e))?;

            persist_file
                .Save(PCWSTR(shortcut_wide.as_ptr()), true)
                .map_err(|e| format!("Failed to save shortcut: {:?}", e))?;

            log::info!("Shortcut created successfully");
            Ok(())
        })();

        // Uninitialize COM
        CoUninitialize();

        result
    }
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn create_shortcut(
    _target: &PathBuf,
    _shortcut_path: &PathBuf,
    _args: &str,
    _description: &str,
) -> Result<(), String> {
    Err("Shortcut creation is only supported on Windows".to_string())
}
