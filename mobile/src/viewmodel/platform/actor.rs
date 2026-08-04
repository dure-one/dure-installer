//! Platform actor implementation

use crate::{dure_info, dure_debug, dure_warn, dure_error};
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
        dure_info!("PlatformActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        dure_error!("PlatformActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    dure_info!("PlatformActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: PlatformCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);

        let result = match cmd {
            PlatformCommand::ListVMs { platform_name } => self.list_vms(platform_name).await,
            PlatformCommand::ScanExistingVMs { platform_name } => self.scan_existing_vms(platform_name).await,
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
                platform_type,
                oauth_access_token,
                oauth_refresh_token,
                oauth_token_expiry,
                connected_email,
                selected_project_id,
            } => {
                self.add_platform(
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
            PlatformCommand::RefreshPlatform { platform_name } => {
                self.refresh_platform(platform_name).await
            }
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
        self.send_progress("list_vms", 0.1, "Checking authentication...")
            .await;

        // Get valid access token (refreshes if expired)
        let (access_token, _) = Self::get_valid_access_token(&platform_name).await?;

        self.send_progress("list_vms", 0.2, "Loading platform config...")
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
                        Err(e) => dure_warn!("Failed to list instances in zone {}: {}", zone, e),
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

    async fn scan_existing_vms(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("scan_existing_vms", 0.1, "Checking authentication...")
            .await;

        // Get valid access token (refreshes if expired)
        let (access_token, _) = Self::get_valid_access_token(&platform_name).await?;

        self.send_progress("scan_existing_vms", 0.2, "Loading platform config...")
            .await;

        // Load platform config
        let (config_path, project_id) = runtime::unblock({
            let platform_name = platform_name.clone();
            move || -> anyhow::Result<(std::path::PathBuf, String)> {
                let (platform, config_path) = Self::load_platform_config(&platform_name)?;
                let project_id = platform
                    .gcp_selected_project_id
                    .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
                Ok((config_path, project_id))
            }
        })
        .await?;

        self.send_progress("scan_existing_vms", 0.5, "Scanning VMs from all zones...")
            .await;

        // List instances from ALL zones in one API call (fast!)
        let all_instances = runtime::unblock({
            let project_id = project_id.clone();
            let access_token = access_token.clone();
            move || -> anyhow::Result<Vec<crate::api::gcp::compute::Instance>> {
                let client = GcpRestClient::new(access_token);
                client.list_instances_aggregated(&project_id)
            }
        })
        .await?;

        self.send_progress("scan_existing_vms", 0.7, "Converting VM data...")
            .await;

        // Convert to VmInstance and save to config
        let vm_count = runtime::unblock({
            let platform_name = platform_name.clone();
            move || -> anyhow::Result<usize> {
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Find the platform
                let platform = app_config
                    .platforms
                    .iter_mut()
                    .find(|p| p.gcp_selected_project_id.as_deref() == Some(&platform_name))
                    .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;

                // Clear existing VMs
                platform.vms.clear();

                // Convert GCP Instances to VmInstance
                for instance in all_instances {
                    // Extract zone name from full zone URL
                    let zone = instance.zone.split('/').last().unwrap_or(&instance.zone).to_string();

                    // Extract region from zone (e.g., "us-central1-a" -> "us-central1")
                    let region = zone.rsplitn(2, '-').nth(1).unwrap_or(&zone).to_string();

                    // Extract machine type from full URL
                    let machine_type = instance.machine_type.split('/').last().unwrap_or(&instance.machine_type).to_string();

                    let vm = crate::config::VmInstance {
                        name: instance.name.clone(),
                        instance_id: instance.id.clone(),
                        zone: zone.clone(),
                        gcp_region: region,
                        machine_type,
                        status: instance.status.clone(),
                        external_ip: instance.external_ip(),
                        internal_ip: instance.internal_ip(),
                        gcp_project_id: project_id.clone(),
                        gcp_billing_account: None, // Not available from instance data
                        created_at: chrono::Utc::now().timestamp(),
                        ssh_key_name: Some(format!("gcp.{}.{}", platform_name, instance.name)),
                    };

                    platform.vms.push(vm);
                }

                let vm_count = platform.vms.len();

                // Save config
                app_config.save(&config_path)?;

                dure_info!("Scanned and saved {} VMs to config", vm_count);

                Ok(vm_count)
            }
        })
        .await?;

        self.send_progress("scan_existing_vms", 1.0, &format!("Scanned {} VMs", vm_count))
            .await;

        self.send_event(PlatformEvent::VMsScanned {
            platform_name,
            vm_count,
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
        self.send_progress("update_firewall", 0.3, "Checking authentication...")
            .await;

        // Get valid access token (refreshes if expired)
        let (access_token, _) = Self::get_valid_access_token(&platform_name).await?;

        self.send_progress("update_firewall", 0.5, "Updating firewall rules...")
            .await;

        // Get project ID
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || Self::load_platform_config(&platform_name).map(|(p, _)| p)
        })
        .await?;

        let project_id = platform
            .gcp_selected_project_id
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;

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
        self.send_progress("fetch_billing", 0.3, "Checking authentication...")
            .await;

        // Get valid access token (refreshes if expired)
        let (access_token, _) = Self::get_valid_access_token(&platform_name).await?;

        self.send_progress("fetch_billing", 0.5, "Fetching billing data...")
            .await;

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
        self.send_progress("list_projects", 0.2, "Checking authentication...")
            .await;

        // Get valid access token (refreshes if expired)
        let (access_token, _) = Self::get_valid_access_token(&platform_name).await?;

        self.send_progress("list_projects", 0.4, "Fetching GCP projects...")
            .await;

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
            let platform_type = platform_type.clone();
            let selected_project_id = selected_project_id.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Check if platform already exists (by project_id)
                if let Some(ref project_id) = selected_project_id {
                    if app_config.platforms.iter().any(|p| p.gcp_selected_project_id.as_ref() == Some(project_id)) {
                        anyhow::bail!("Platform with project '{}' already exists", project_id);
                    }
                }

                // Get project_id for audit before moving selected_project_id
                let project_id_for_audit = selected_project_id.as_deref().unwrap_or("unknown").to_string();

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
                let _ = crate::calc::audit::push_gui("system", "desktop", "platform add", &project_id_for_audit);

                Ok(())
            }
        })
        .await?;

        self.send_progress("add_platform", 1.0, "Platform added")
            .await;

        self.send_event(PlatformEvent::PlatformAdded {
            platform_name: selected_project_id.unwrap_or_default(),
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
        // Only get access token if we need to delete cloud resources
        let needs_cloud_access = delete_options.delete_vms || delete_options.delete_project;

        let access_token = if needs_cloud_access {
            self.send_progress("delete_platform", 0.2, "Checking authentication...")
                .await;

            // Get valid access token (refreshes if expired)
            match Self::get_valid_access_token(&platform_name).await {
                Ok((token, _)) => Some(token),
                Err(e) => {
                    // If token refresh fails (e.g., expired refresh token), we can still
                    // delete the local config, but can't delete cloud resources
                    if needs_cloud_access {
                        return Err(anyhow::anyhow!(
                            "Cannot delete cloud resources without valid authentication.\n\
                             Your tokens have expired. To delete VMs/project from GCP:\n\
                             1. Reconnect your Google account first\n\
                             2. Then delete the platform\n\n\
                             Or uncheck 'Delete VMs' and 'Delete Project' to only remove local config.\n\n\
                             Error: {}", e
                        ));
                    }
                    None
                }
            }
        } else {
            None
        };

        self.send_progress("delete_platform", 0.25, "Preparing deletion...")
            .await;

        // Get platform data before deletion
        let (project_id, vms) = runtime::unblock({
            let platform_name = platform_name.clone();
            move || -> anyhow::Result<(String, Vec<crate::config::VmInstance>)> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Find platform
                let platform = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;

                let project_id = platform.gcp_selected_project_id.clone()
                    .ok_or_else(|| anyhow::anyhow!("No project selected for platform"))?;
                let vms = platform.vms.clone();

                Ok((project_id, vms))
            }
        })
        .await?;

        // Delete VMs from GCP if requested
        if delete_options.delete_vms && !vms.is_empty() {
            self.send_progress("delete_platform", 0.5, "Deleting VMs from GCP...")
                .await;

            let token = access_token.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Access token required but not available"))?;

            for vm in &vms {
                let vm_name = vm.name.clone();
                let vm_name_for_error = vm_name.clone();
                let zone = vm.zone.clone();
                let token = token.clone();
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

            let token = access_token.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Access token required but not available"))?;
            let token = token.clone();
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

    /// Helper to get a valid access token, refreshing if expired
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_valid_access_token(
        platform_name: &str,
    ) -> anyhow::Result<(String, PathBuf)> {
        runtime::unblock({
            let platform_name = platform_name.to_string();
            move || -> anyhow::Result<(String, PathBuf)> {
                let config_path = Self::get_config_path()?;
                let mut config = AppConfig::load_or_default(&config_path);

                let platform = config
                    .platforms
                    .iter_mut()
                    .find(|p| p.gcp_selected_project_id.as_deref() == Some(&platform_name))
                    .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", platform_name))?;

                // Check if we have tokens
                let access_token = platform
                    .gcp_oauth_access_token
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Not authenticated with GCP"))?;
                let refresh_token = platform
                    .gcp_oauth_refresh_token
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;

                // Check if token is expired (with 5 minute buffer)
                let now = chrono::Utc::now().timestamp();
                let expiry = platform.gcp_oauth_token_expiry.unwrap_or(0);
                let needs_refresh = expiry <= now + 300; // Refresh if expires in < 5 minutes

                if needs_refresh {
                    dure_info!("Access token expired or expiring soon, refreshing...");

                    // Get OAuth credentials
                    let oauth_handler = crate::api::gcp::oauth::OAuthHandler::default();

                    // Refresh the token
                    match crate::api::gcp::oauth::refresh_access_token(
                        oauth_handler.client_id(),
                        oauth_handler.client_secret(),
                        refresh_token,
                    ) {
                        Ok(oauth_result) => {
                            // Update platform config with new token
                            platform.gcp_oauth_access_token = Some(oauth_result.access_token.clone());
                            platform.gcp_oauth_token_expiry = Some(oauth_result.expires_at as i64);
                            // Keep the existing refresh token (it doesn't change)

                            // Save config
                            config.save(&config_path)?;

                            dure_info!("Access token refreshed successfully");
                            Ok((oauth_result.access_token, config_path))
                        }
                        Err(e) => {
                            // Check if the error is due to expired/revoked refresh token
                            let error_msg = e.to_string();
                            if error_msg.contains("invalid_grant") || error_msg.contains("Token has been expired or revoked") {
                                dure_error!("Refresh token has expired or been revoked. Clearing tokens...");

                                // Clear the invalid tokens
                                platform.gcp_oauth_access_token = None;
                                platform.gcp_oauth_refresh_token = None;
                                platform.gcp_oauth_token_expiry = None;

                                // Save config with cleared tokens
                                config.save(&config_path)?;

                                return Err(anyhow::anyhow!(
                                    "Your Google Cloud authentication has expired. Please reconnect your Google account:\n\
                                     1. Click the 'Connect' button in the Platform tab\n\
                                     2. Sign in with Google again\n\
                                     3. Grant permissions to Dure\n\n\
                                     This happens when refresh tokens expire after 6 months of inactivity or are revoked."
                                ));
                            }

                            // Other refresh errors - propagate as-is
                            Err(e)
                        }
                    }
                } else {
                    Ok((access_token.clone(), config_path))
                }
            }
        })
        .await
    }

    async fn refresh_platform(&mut self, platform_name: String) -> anyhow::Result<()> {
        dure_info!("🔄 Refreshing platform: {}", platform_name);

        // Load platform config
        #[cfg(not(target_arch = "wasm32"))]
        let (platform, _) = Self::load_platform_config(&platform_name)?;

        #[cfg(target_arch = "wasm32")]
        return Err(anyhow::anyhow!("Refresh not supported on WASM"));

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Step 1: Check VM status
            let vm_status = self.check_vm_status(&platform).await;

            // Step 2: Check firewall status
            let firewall_status = self.check_firewall_status(&platform).await;

            // Step 3: Test SSH connection
            let ssh_status = self.test_ssh_connection(&platform).await;

            // Send RefreshCompleted event
            self.send_event(PlatformEvent::RefreshCompleted {
                platform_name: platform_name.clone(),
                vm_status,
                firewall_status,
                ssh_status,
            })
            .await;

            Ok(())
        }
    }

    async fn check_vm_status(
        &self,
        platform: &crate::config::CloudPlatformConfig,
    ) -> super::VmStatus {
        use super::VmStatus;

        // Get project ID
        let project_id = match &platform.gcp_selected_project_id {
            Some(id) => id,
            None => {
                return VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                };
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                dure_warn!("No valid access token for VM check");
                return VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                };
            }
        };

        // Create GCP client
        let client = crate::api::gcp::GcpRestClient::new(access_token);

        // List VMs (use aggregated to check all zones)
        match client.list_instances_aggregated(project_id) {
            Ok(instances) => {
                if let Some(vm) = instances.first() {
                    // Extract external IP from network interfaces
                    let external_ip = vm
                        .network_interfaces
                        .first()
                        .and_then(|ni| ni.access_configs.first())
                        .and_then(|ac| ac.nat_ip.clone());

                    // Extract zone from full zone path (e.g., "zones/us-central1-a" -> "us-central1-a")
                    let zone = vm.zone.split('/').last().map(|s| s.to_string());

                    VmStatus {
                        exists: true,
                        name: Some(vm.name.clone()),
                        zone,
                        external_ip,
                        status: Some(vm.status.clone()),
                    }
                } else {
                    // No VMs found
                    VmStatus {
                        exists: false,
                        name: None,
                        zone: None,
                        external_ip: None,
                        status: None,
                    }
                }
            }
            Err(e) => {
                dure_error!("Failed to list VMs: {}", e);
                VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                }
            }
        }
    }

    async fn check_firewall_status(
        &self,
        platform: &crate::config::CloudPlatformConfig,
    ) -> super::FirewallStatus {
        use super::FirewallStatus;

        // Get project ID
        let project_id = match &platform.gcp_selected_project_id {
            Some(id) => id,
            None => {
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                dure_warn!("No valid access token for firewall check");
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Get current external IP
        let current_ip = match crate::api::gcp::get_current_ip() {
            Ok(ip) => ip,
            Err(e) => {
                dure_warn!("Failed to get current IP: {}", e);
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Create GCP client
        let client = crate::api::gcp::GcpRestClient::new(access_token);

        // Check if current IP is whitelisted
        match client.check_ip_whitelisted(project_id, &current_ip) {
            Ok(whitelisted) => FirewallStatus {
                whitelisted,
                current_ip: Some(current_ip),
            },
            Err(e) => {
                dure_error!("Failed to check firewall: {}", e);
                FirewallStatus {
                    whitelisted: false,
                    current_ip: Some(current_ip),
                }
            }
        }
    }

    async fn test_ssh_connection(
        &self,
        platform: &crate::config::CloudPlatformConfig,
    ) -> super::SshStatus {
        use super::SshStatus;

        // Get VM info
        let (external_ip, keyring_domain) = match platform.vms.first() {
            Some(vm) => {
                let ip = match &vm.external_ip {
                    Some(ip) => ip.clone(),
                    None => {
                        return SshStatus {
                            connected: false,
                            error: Some("No external IP configured".to_string()),
                        };
                    }
                };
                (ip, vm.ssh_key_name.clone())
            }
            None => {
                return SshStatus {
                    connected: false,
                    error: Some("No VM configured".to_string()),
                };
            }
        };

        // Test SSH connection
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Build SSH host config
            let host_config = crate::config::SshHostConfig {
                host: format!("root@{}", external_ip),
                password: None,
                private_key_path: None,
                keyring_domain,
                port: 22,
                initialized: false,
                last_status: None,
                platform_name: None,
                docker_containers: Vec::new(),
                ansible_roles: Vec::new(),
                dure_wss_config: None,
            };

            // Run test connection
            match runtime::unblock(move || {
                smol::block_on(async {
                    async_compat::Compat::new(crate::calc::ssh::test_connection(&host_config)).await
                })
            })
            .await
            {
                Ok(_) => SshStatus {
                    connected: true,
                    error: None,
                },
                Err(e) => SshStatus {
                    connected: false,
                    error: Some(format!("Connection failed: {}", e)),
                },
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            SshStatus {
                connected: false,
                error: Some("SSH test not supported on WASM".to_string()),
            }
        }
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
