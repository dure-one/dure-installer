//! Platform VM command implementation using ViewModel
//!
//! Example CLI commands demonstrating MVVM pattern for async operations.

use crate::viewmodel::{ViewModel, ViewModelEvent, platform::PlatformEvent};
use std::time::Duration;
use anyhow::Result;

/// Create a VM using ViewModel (async with progress display)
pub async fn create_vm(
    platform_name: String,
    vm_name: String,
    zone: String,
    machine_type: String,
) -> Result<()> {
    let mut vm = ViewModel::new_headless();

    vm.create_vm(platform_name.clone(), vm_name.clone(), zone, machine_type)?;

    println!("Creating VM '{}'...", vm_name);

    // Poll for completion
    loop {
        let events = vm.poll_events_headless();
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::Progress { progress, status, .. }) => {
                    print!("\r[{:>3.0}%] {}", progress * 100.0, status);
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
                ViewModelEvent::Platform(PlatformEvent::VMCreated { vm_name, external_ip, .. }) => {
                    println!("\n✓ VM created: {} at {}", vm_name, external_ip);
                    return Ok(());
                }
                ViewModelEvent::Platform(PlatformEvent::Error { error, .. }) => {
                    eprintln!("\n✗ Failed: {}", error);
                    return Err(anyhow::anyhow!(error));
                }
                _ => {}
            }
        }
        smol::Timer::after(Duration::from_millis(100)).await;
    }
}

/// List VMs using ViewModel
pub async fn list_vms(platform_name: String) -> Result<()> {
    let mut vm = ViewModel::new_headless();

    vm.list_vms(platform_name.clone())?;

    println!("Listing VMs for platform '{}'...", platform_name);

    // Poll for completion
    loop {
        let events = vm.poll_events_headless();
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::VMsListed { vms, .. }) => {
                    println!("\nVMs:");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("{:<20} {:<20} {:<15} {:<10}", "Name", "Zone", "External IP", "Status");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    for vm in vms {
                        println!("{:<20} {:<20} {:<15} {:<10}",
                            vm.name,
                            vm.zone,
                            vm.external_ip.unwrap_or_else(|| "-".to_string()),
                            vm.status
                        );
                    }
                    return Ok(());
                }
                ViewModelEvent::Platform(PlatformEvent::Error { error, .. }) => {
                    eprintln!("✗ Failed: {}", error);
                    return Err(anyhow::anyhow!(error));
                }
                _ => {}
            }
        }
        smol::Timer::after(Duration::from_millis(100)).await;
    }
}

/// Delete a VM using ViewModel
pub async fn delete_vm(
    platform_name: String,
    vm_name: String,
    zone: String,
) -> Result<()> {
    let mut vm = ViewModel::new_headless();

    vm.delete_vm(platform_name.clone(), vm_name.clone(), zone)?;

    println!("Deleting VM '{}'...", vm_name);

    // Poll for completion
    loop {
        let events = vm.poll_events_headless();
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::Progress { progress, status, .. }) => {
                    print!("\r[{:>3.0}%] {}", progress * 100.0, status);
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
                ViewModelEvent::Platform(PlatformEvent::VMDeleted { vm_name, .. }) => {
                    println!("\n✓ VM deleted: {}", vm_name);
                    return Ok(());
                }
                ViewModelEvent::Platform(PlatformEvent::Error { error, .. }) => {
                    eprintln!("\n✗ Failed: {}", error);
                    return Err(anyhow::anyhow!(error));
                }
                _ => {}
            }
        }
        smol::Timer::after(Duration::from_millis(100)).await;
    }
}
