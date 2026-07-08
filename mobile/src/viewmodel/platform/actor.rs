//! Platform actor implementation

use super::{PlatformCommand, PlatformEvent, VmInfo};
use crate::api::gcp::GcpRestClient;
use crate::config::AppConfig;
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};
use std::path::PathBuf;

pub struct PlatformActor {
    command_rx: Receiver<PlatformCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl PlatformActor {
    pub fn new(command_rx: Receiver<PlatformCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
        }
    }

    pub async fn run(mut self) {
        log::info!("PlatformActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("PlatformActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    log::info!("PlatformActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: PlatformCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);

        let result = match cmd {
            PlatformCommand::ListVMs { platform_name } => self.list_vms(platform_name).await,
            PlatformCommand::CreateVM {
                platform_name,
                vm_name,
                zone,
                machine_type,
            } => {
                self.create_vm(platform_name, vm_name, zone, machine_type)
                    .await
            }
            PlatformCommand::DeleteVM {
                platform_name,
                vm_name,
                zone,
            } => self.delete_vm(platform_name, vm_name, zone).await,
            PlatformCommand::RestartVM {
                platform_name,
                vm_name,
                zone,
            } => self.restart_vm(platform_name, vm_name, zone).await,
            PlatformCommand::RegenerateVM {
                platform_name,
                vm_name,
                zone,
            } => self.regenerate_vm(platform_name, vm_name, zone).await,
            PlatformCommand::UpdateFirewall {
                platform_name,
                allow_ip,
            } => self.update_firewall(platform_name, allow_ip).await,
            PlatformCommand::FetchBilling {
                platform_name,
                project_id,
                dataset,
                table,
            } => {
                self.fetch_billing(platform_name, project_id, dataset, table)
                    .await
            }
            PlatformCommand::ListProjects { platform_name } => {
                self.list_projects(platform_name).await
            }
            PlatformCommand::SelectProject {
                platform_name,
                project_id,
            } => self.select_project(platform_name, project_id).await,
            PlatformCommand::StartOAuth { platform_name } => self.start_oauth(platform_name).await,
            PlatformCommand::CompleteOAuth {
                platform_name,
                auth_code,
            } => self.complete_oauth(platform_name, auth_code).await,
            PlatformCommand::AddPlatform {
                name,
                platform_type,
                oauth_access_token,
                oauth_refresh_token,
                oauth_token_expiry,
                connected_email,
                selected_project_id,
            } => {
                self.add_platform(
                    name,
                    platform_type,
                    oauth_access_token,
                    oauth_refresh_token,
                    oauth_token_expiry,
                    connected_email,
                    selected_project_id,
                )
                .await
            }
            PlatformCommand::DeletePlatform {
                platform_name,
                delete_options,
            } => self.delete_platform(platform_name, delete_options).await,
            _ => {
                // Unimplemented commands
                Err(anyhow::anyhow!("Command not implemented: {:?}", cmd))
            }
        };

        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }

        Ok(())
    }

    /// Helper to get config file path
    #[cfg(not(target_arch = "wasm32"))]
    fn get_config_path() -> anyhow::Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        Ok(proj_dirs.config_dir().join("config.yml"))
    }

    /// Helper to load platform config by name
    #[cfg(not(target_arch = "wasm32"))]
    fn load_platform_config(
        platform_name: &str,
    ) -> anyhow::Result<(crate::config::CloudPlatformConfig, PathBuf)> {
        let config_path = Self::get_config_path()?;
        let config = AppConfig::load_or_default(&config_path);
        let platform = config
            .platforms
            .into_iter()
            .find(|p| p.gcp_selected_project_id.as_deref() == Some(platform_name))
            .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;
        Ok((platform, config_path))
    }

    async fn list_vms(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("list_vms", 0.1, "Loading platform config...")
            .await;

        // Load platform config
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        self.send_progress("list_vms", 0.3, "Fetching zones...")
            .await;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        // Get zones to query (from existing VMs or list all zones)
        let zones = runtime::unblock({
            let project_id = project_id.clone();
            let access_token = access_token.clone();
            let vms = platform.vms.clone();
            move || -> anyhow::Result<Vec<String>> {
                let client = GcpRestClient::new(access_token.clone());
                if !vms.is_empty() {
                    // Use zones from existing VMs
                    let zones: std::collections::HashSet<String> =
                        vms.iter().map(|vm| vm.zone.clone()).collect();
                    Ok(zones.into_iter().collect())
                } else {
                    // List all zones
                    let zone_list = client.list_zones(&project_id)?;
                    Ok(zone_list.items.into_iter().map(|z| z.name).collect())
                }
            }
        })
        .await?;

        self.send_progress("list_vms", 0.5, "Fetching VMs from GCP...")
            .await;

        // List instances from all zones
        let all_instances = runtime::unblock({
            let project_id = project_id.clone();
            move || -> anyhow::Result<Vec<crate::api::gcp::compute::Instance>> {
                let client = GcpRestClient::new(access_token);
                let mut all_vms = Vec::new();
                for zone in zones {
                    match client.list_instances(&project_id, &zone) {
                        Ok(list) => all_vms.extend(list.items),
                        Err(e) => log::warn!("Failed to list instances in zone {}: {}", zone, e),
                    }
                }
                Ok(all_vms)
            }
        })
        .await?;

        // Convert to VmInfo
        let vm_infos: Vec<VmInfo> = all_instances
            .into_iter()
            .map(|vm| {
                let external_ip = vm.external_ip();
                VmInfo {
                    name: vm.name,
                    zone: vm.zone,
                    external_ip,
                    status: vm.status,
                }
            })
            .collect();

        self.send_event(PlatformEvent::VMsListed {
            platform_name,
            vms: vm_infos,
        })
        .await;

        Ok(())
    }

    async fn create_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
        machine_type: String,
    ) -> anyhow::Result<()> {
        self.send_progress("create_vm", 0.0, "Starting VM creation...")
            .await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        self.send_progress("create_vm", 0.3, "Creating VM instance...")
            .await;

        let external_ip = runtime::unblock({
            let vm_name_clone = vm_name.clone();
            move || -> anyhow::Result<String> {
                let client = GcpRestClient::new(access_token);

                // Create instance request (debian-11 micro instance)
                let instance_req = crate::api::gcp::compute::InstanceRequest::debian_micro(
                    vm_name_clone.clone(),
                    zone.clone(),
                );

                let operation = client.create_instance(&project_id, &zone, &instance_req)?;

                // Wait for operation to complete
                let op_name = operation.name.split('/').last().unwrap_or(&operation.name);
                client.wait_for_operation(&project_id, &zone, op_name, 120)?;

                // Get instance details to fetch external IP
                let instance = client.get_instance(&project_id, &zone, &vm_name_clone)?;
                Ok(instance
                    .external_ip()
                    .unwrap_or_else(|| "pending".to_string()))
            }
        })
        .await?;

        self.send_event(PlatformEvent::VMCreated {
            platform_name,
            vm_name,
            external_ip,
        })
        .await;

        Ok(())
    }

    async fn delete_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.send_progress("delete_vm", 0.5, "Deleting VM...").await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        runtime::unblock({
            let vm_name_clone = vm_name.clone();
            move || -> anyhow::Result<()> {
                let client = GcpRestClient::new(access_token);
                let operation = client.delete_instance(&project_id, &zone, &vm_name_clone)?;

                // Wait for deletion to complete
                let op_name = operation.name.split('/').last().unwrap_or(&operation.name);
                client.wait_for_operation(&project_id, &zone, op_name, 120)?;
                Ok(())
            }
        })
        .await?;

        self.send_event(PlatformEvent::VMDeleted {
            platform_name,
            vm_name,
        })
        .await;

        Ok(())
    }

    async fn restart_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.send_progress("restart_vm", 0.5, "Restarting VM...")
            .await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        runtime::unblock({
            let vm_name_clone = vm_name.clone();
            move || -> anyhow::Result<()> {
                let client = GcpRestClient::new(access_token);
                let operation = client.reset_instance(&project_id, &zone, &vm_name_clone)?;

                // Wait for reset to complete
                let op_name = operation.name.split('/').last().unwrap_or(&operation.name);
                client.wait_for_operation(&project_id, &zone, op_name, 120)?;
                Ok(())
            }
        })
        .await?;

        self.send_event(PlatformEvent::VMRestarted {
            platform_name,
            vm_name,
        })
        .await;

        Ok(())
    }

    async fn regenerate_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.send_progress("regenerate_vm", 0.3, "Regenerating VM...")
            .await;

        // Load platform config
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        self.send_progress("regenerate_vm", 0.6, "Calling GCP API...")
            .await;

        // Regenerate VM
        let message = runtime::unblock({
            let mut platform = platform.clone();
            let zone = zone.clone();
            move || {
                let access_token = platform
                    .gcp_oauth_access_token
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;
                let client = GcpRestClient::new(access_token);
                crate::calc::hosting_gcp::regenerate_vm(&client, &mut platform, &zone)
            }
        })
        .await?;

        self.send_event(PlatformEvent::VMRegenerated {
            platform_name,
            vm_name,
            message,
        })
        .await;

        Ok(())
    }

    async fn update_firewall(
        &mut self,
        platform_name: String,
        allow_ip: String,
    ) -> anyhow::Result<()> {
        self.send_progress("update_firewall", 0.5, "Updating firewall rules...")
            .await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        runtime::unblock({
            let allow_ip = allow_ip.clone();
            move || -> anyhow::Result<()> {
                let client = GcpRestClient::new(access_token);
                client.add_ip_to_firewall(&project_id, &allow_ip)?;
                Ok(())
            }
        })
        .await?;

        self.send_event(PlatformEvent::FirewallUpdated {
            platform_name,
            whitelisted_ip: allow_ip,
        })
        .await;

        Ok(())
    }

    async fn fetch_billing(
        &mut self,
        platform_name: String,
        project_id: String,
        dataset: String,
        table: String,
    ) -> anyhow::Result<()> {
        self.send_progress("fetch_billing", 0.5, "Fetching billing data...")
            .await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        let records = runtime::unblock(
            move || -> anyhow::Result<Vec<crate::api::gcp::bigquery::BillingRecord>> {
                let client = GcpRestClient::new(access_token);
                client.get_current_month_billing(&project_id, &dataset, &table)
            },
        )
        .await?;

        self.send_event(PlatformEvent::BillingFetched {
            platform_name,
            records,
        })
        .await;

        Ok(())
    }

    async fn list_projects(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("list_projects", 0.3, "Fetching GCP projects...")
            .await;

        // Load platform to get access token
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let access_token = platform
            .gcp_oauth_access_token
            .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;

        self.send_progress("list_projects", 0.6, "Retrieving project list...")
            .await;

        // Fetch projects from GCP
        let project_list = runtime::unblock({
            move || {
                let client = GcpRestClient::new(access_token);
                client.list_projects(None)
            }
        })
        .await?;

        // Convert to (id, name) tuples
        let projects: Vec<(String, String)> = project_list
            .projects
            .into_iter()
            .map(|p| {
                let name = p.display_name().to_string();
                (p.id().to_string(), name)
            })
            .collect();

        self.send_event(PlatformEvent::ProjectsListed {
            platform_name,
            projects,
        })
        .await;

        Ok(())
    }

    async fn select_project(
        &mut self,
        platform_name: String,
        project_id: String,
    ) -> anyhow::Result<()> {
        self.send_progress("select_project", 0.5, "Updating project selection...")
            .await;

        // Update config
        runtime::unblock({
            let platform_name = platform_name.clone();
            let project_id = project_id.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut config = AppConfig::load_or_default(&config_path);

                if let Some(platform) = config
                    .platforms
                    .iter_mut()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                {
                    platform.gcp_selected_project_id = Some(project_id.clone());
                    config.save(&config_path)?;
                } else {
                    return Err(anyhow::anyhow!("Platform '{}' not found", platform_name));
                }

                Ok(())
            }
        })
        .await?;

        self.send_event(PlatformEvent::ProjectSelected {
            platform_name,
            project_id,
        })
        .await;

        Ok(())
    }

    async fn start_oauth(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("start_oauth", 0.2, "Starting OAuth flow...")
            .await;

        // TODO: OAuth flow requires browser interaction and is complex
        // The current implementation uses poll_promise with run_oauth_flow()
        // which handles: auth URL generation, browser launch, callback server,
        // token exchange, and user info fetching all in one blocking operation.
        //
        // For now, return a placeholder URL. The UI should continue using
        // poll_promise for OAuth until this is properly refactored.
        let auth_url = "https://accounts.google.com/o/oauth2/auth?...".to_string();

        self.send_event(PlatformEvent::OAuthStarted {
            platform_name,
            auth_url,
        })
        .await;

        Ok(())
    }

    async fn complete_oauth(
        &mut self,
        platform_name: String,
        _auth_code: String,
    ) -> anyhow::Result<()> {
        self.send_progress("complete_oauth", 0.3, "Completing OAuth...")
            .await;

        // TODO: OAuth completion is handled by run_oauth_flow() in the current implementation
        // which runs a callback server and exchanges the auth code for tokens.
        // This needs proper refactoring to work with the actor pattern.
        //
        // For now, return a placeholder. The UI should continue using poll_promise.
        let email = "user@example.com".to_string();

        self.send_event(PlatformEvent::OAuthCompleted {
            platform_name,
            email,
        })
        .await;

        Ok(())
    }

    async fn add_platform(
        &mut self,
        name: String,
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    ) -> anyhow::Result<()> {
        self.send_progress("add_platform", 0.5, "Adding platform...")
            .await;

        runtime::unblock({
            let name = name.clone();
            let platform_type = platform_type.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Check if platform already exists (by project_id)
                if let Some(ref project_id) = selected_project_id {
                    if app_config.platforms.iter().any(|p| p.gcp_selected_project_id.as_ref() == Some(project_id)) {
                        anyhow::bail!("Platform with project '{}' already exists", project_id);
                    }
                }

                // Create new platform
                let platform = crate::config::CloudPlatformConfig {
                    platform_type: platform_type.clone(),
                    gcp_oauth_access_token: oauth_access_token,
                    gcp_oauth_refresh_token: oauth_refresh_token,
                    gcp_oauth_token_expiry: oauth_token_expiry,
                    gcp_connected_email: connected_email,
                    gcp_selected_project_id: selected_project_id,
                    firebase_project_id: None,
                    firebase_api_key: None,
                    supabase_project_ref: None,
                    supabase_api_url: None,
                    supabase_anon_key: None,
                    api_token: None,
                    service_account_json: None,
                    vms: Vec::new(),
                    cached_total_project_count: None,
                    cached_vm_status: None,
                    cached_firewall_status: None,
                    cached_vm_external_ip: None,
                    last_status_refresh: None,
                };

                // Add to config
                app_config.platforms.push(platform);

                // Save config
                app_config.save(&config_path)?;

                // Record audit event
                let _ = crate::calc::audit::push_gui("system", "desktop", "platform add", &name);

                Ok(())
            }
        })
        .await?;

        self.send_progress("add_platform", 1.0, "Platform added")
            .await;

        self.send_event(PlatformEvent::PlatformAdded {
            platform_name: name,
            platform_type,
        })
        .await;

        Ok(())
    }

    async fn delete_platform(
        &mut self,
        platform_name: String,
        delete_options: crate::viewmodel::platform::DeleteOptions,
    ) -> anyhow::Result<()> {
        self.send_progress("delete_platform", 0.25, "Preparing deletion...")
            .await;

        // Get platform data before deletion
        let (access_token, project_id, vms) = runtime::unblock({
            let platform_name = platform_name.clone();
            move || -> anyhow::Result<(String, String, Vec<crate::config::VmInstance>)> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Find platform
                let platform = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;

                let access_token = platform.gcp_oauth_access_token.clone()
                    .ok_or_else(|| anyhow::anyhow!("No OAuth token for platform"))?;
                let project_id = platform.gcp_selected_project_id.clone()
                    .ok_or_else(|| anyhow::anyhow!("No project selected for platform"))?;
                let vms = platform.vms.clone();

                Ok((access_token, project_id, vms))
            }
        })
        .await?;

        // Delete VMs from GCP if requested
        if delete_options.delete_vms && !vms.is_empty() {
            self.send_progress("delete_platform", 0.5, "Deleting VMs from GCP...")
                .await;

            for vm in &vms {
                let vm_name = vm.name.clone();
                let vm_name_for_error = vm_name.clone();
                let zone = vm.zone.clone();
                let token = access_token.clone();
                let proj_id = project_id.clone();

                runtime::unblock(move || -> anyhow::Result<()> {
                    let client = crate::api::gcp::GcpRestClient::new(token);
                    client.delete_instance(&proj_id, &zone, &vm_name)?;
                    Ok(())
                })
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to delete VM '{}': {}", vm_name_for_error, e)
                })?;
            }
        }

        // Delete GCP project if requested
        if delete_options.delete_project {
            self.send_progress("delete_platform", 0.75, "Deleting GCP project...")
                .await;

            let token = access_token.clone();
            let proj_id = project_id.clone();

            runtime::unblock(move || -> anyhow::Result<()> {
                let client = crate::api::gcp::GcpRestClient::new(token);
                client.delete_project(&proj_id)?;
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete project '{}': {}", project_id, e))?;
        }

        // Remove from local config
        let vm_count = runtime::unblock({
            let platform_name = platform_name.clone();
            move || -> anyhow::Result<usize> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Find and remove platform
                let platform_idx = app_config
                    .platforms
                    .iter()
                    .position(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;

                let platform = app_config.platforms.remove(platform_idx);
                let vm_count = platform.vms.len();

                // Save config
                app_config.save(&config_path)?;

                // Record audit event
                let _ = crate::calc::audit::push_gui(
                    "system",
                    "desktop",
                    "platform delete",
                    &format!("{} ({} VMs)", platform_name, vm_count),
                );

                Ok(vm_count)
            }
        })
        .await?;

        self.send_progress("delete_platform", 1.0, "Platform deleted")
            .await;

        self.send_event(PlatformEvent::PlatformDeleted {
            platform_name,
            vm_count,
        })
        .await;

        Ok(())
    }

    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self
            .event_tx
            .send(ViewModelEvent::Platform(PlatformEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }))
            .await;
    }

    async fn send_event(&self, event: PlatformEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(event)).await;
    }

    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self
            .event_tx
            .send(ViewModelEvent::Platform(PlatformEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }))
            .await;
    }
}
