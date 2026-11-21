//! Mycart actor implementation
//! Phase 4: Cleaned up - handles crontab commands only

use super::commands::MycartCommand;
use super::events::MycartEvent;
use crate::calc::mycart;
use crate::calc::profile::ProfileContext;
use crate::config::AppConfig;
use crate::viewmodel::common::ViewModelEvent;
use crate::{dure_debug, dure_error, dure_info};
use smol::channel::{Receiver, Sender};

pub struct MycartActor {
    command_rx: Receiver<MycartCommand>,
    event_tx: Sender<ViewModelEvent>,
    base_url: String,
    auth_token: Option<String>,
    profile: Option<ProfileContext>,
}

impl MycartActor {
    pub fn new(
        command_rx: Receiver<MycartCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: Option<ProfileContext>,
    ) -> Self {
        Self {
            command_rx,
            event_tx,
            base_url: "https://mycart.dure.app/api/_".to_string(), // ponytail: placeholder URL
            auth_token: None,
            profile,
        }
    }

    fn get_config_path(&self) -> anyhow::Result<std::path::PathBuf> {
        self.profile
            .as_ref()
            .map(|p| p.config_file.clone())
            .ok_or_else(|| anyhow::anyhow!("No profile context"))
    }

    pub async fn run(mut self) {
        dure_info!("[MycartActor] Starting");

        while let Ok(command) = self.command_rx.recv().await {
            dure_debug!("[MycartActor] Received command: {:?}", command);

            if let Err(e) = self.handle_command(command).await {
                dure_error!("[MycartActor] Command error: {}", e);
                self.send_event(MycartEvent::Error {
                    operation: "command".to_string(),
                    error: e.to_string(),
                })
                .await;
            }
        }

        dure_info!("[MycartActor] Stopped");
    }

    async fn handle_command(&mut self, command: MycartCommand) -> anyhow::Result<()> {
        match command {
            MycartCommand::LoadStats { site_address } => self.load_stats(site_address).await,

            MycartCommand::LoadLogs {
                site_address,
                limit,
            } => self.load_logs(site_address, limit).await,

            MycartCommand::LoadOperations {
                site_address,
                limit,
            } => self.load_operations(site_address, limit).await,

            MycartCommand::LoadMycartLogs {
                site_address,
                limit,
            } => self.load_mycart_logs(site_address, limit).await,

            MycartCommand::LoadXMPPProxyLogs {
                site_address,
                limit,
            } => self.load_xmppproxy_logs(site_address, limit).await,

            MycartCommand::LoadFail2BanRSLogs {
                site_address,
                limit,
            } => self.load_fail2banrs_logs(site_address, limit).await,

            MycartCommand::ListCrontab { site_address } => self.list_crontab(site_address).await,

            MycartCommand::EnableCronjob {
                site_address,
                job_id,
            } => self.enable_cronjob(site_address, job_id).await,

            MycartCommand::DisableCronjob {
                site_address,
                job_id,
            } => self.disable_cronjob(site_address, job_id).await,

            MycartCommand::SetInterval {
                site_address,
                job_id,
                interval,
            } => self.set_interval(site_address, job_id, interval).await,
        }
    }

    async fn send_event(&self, event: MycartEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Mycart(event)).await;
    }

    // Phase 4: Handler implementations for existing UI commands (stub for now)
    async fn load_stats(&mut self, site_address: String) -> anyhow::Result<()> {
        // ponytail: placeholder - actual implementation needs JWT token from config
        dure_debug!("[MycartActor] Would load stats for {}", site_address);
        self.send_event(MycartEvent::StatsLoaded {
            site_address,
            stats: None,
        })
        .await;
        Ok(())
    }

    // Phase 5: Installer logs handler (stdout filtered by site)
    async fn load_logs(&mut self, site_address: String, _limit: usize) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Loading installer logs for {}", site_address);
        // ponytail: needs LogActor integration - placeholder returns empty
        self.send_event(MycartEvent::LogsLoaded {
            site_address,
            logs: Vec::new(),
        })
        .await;
        Ok(())
    }

    // Phase 5: SSH server log handlers
    async fn load_mycart_logs(
        &mut self,
        site_address: String,
        _limit: usize,
    ) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Loading Mycart logs for {}", site_address);

        let config_path = self.get_config_path()?;
        let config = AppConfig::load_or_default(&config_path);

        let site = config
            .mycart_sites
            .iter()
            .find(|s| s.site_address == site_address)
            .ok_or_else(|| anyhow::anyhow!("Site not found: {}", site_address))?;

        dure_debug!(
            "[MycartActor] Site found: ssh_host={:?}, platform_id={:?}",
            site.ssh_host,
            site.platform_id
        );

        let host_config = mycart::resolve_ssh_host_for_site(&config, site)
            .ok_or_else(|| anyhow::anyhow!("No SSH host found for site: {}", site_address))?;

        dure_info!(
            "[MycartActor] Using SSH host: {}",
            host_config.host
        );

        // ponytail: keyring loading - pass None for now, SSH uses password from config
        let logs = mycart::get_duremycart_logs(host_config, None).await?;

        self.send_event(MycartEvent::MycartLogsLoaded { site_address, logs })
            .await;

        Ok(())
    }

    async fn load_xmppproxy_logs(
        &mut self,
        site_address: String,
        _limit: usize,
    ) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Loading XMPPProxy logs for {}", site_address);

        let config_path = self.get_config_path()?;
        let config = AppConfig::load_or_default(&config_path);

        let site = config
            .mycart_sites
            .iter()
            .find(|s| s.site_address == site_address)
            .ok_or_else(|| anyhow::anyhow!("Site not found: {}", site_address))?;

        let host_config = mycart::resolve_ssh_host_for_site(&config, site)
            .ok_or_else(|| anyhow::anyhow!("No SSH host found for site: {}", site_address))?;

        dure_info!(
            "[MycartActor] Using SSH host: {}",
            host_config.host
        );

        // ponytail: keyring loading - pass None for now, SSH uses password from config
        let logs = mycart::get_xmppproxy_logs(host_config, None).await?;

        self.send_event(MycartEvent::XMPPProxyLogsLoaded { site_address, logs })
            .await;

        Ok(())
    }

    async fn load_fail2banrs_logs(
        &mut self,
        site_address: String,
        _limit: usize,
    ) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Loading Fail2BanRS logs for {}", site_address);

        let config_path = self.get_config_path()?;
        let config = AppConfig::load_or_default(&config_path);

        let site = config
            .mycart_sites
            .iter()
            .find(|s| s.site_address == site_address)
            .ok_or_else(|| anyhow::anyhow!("Site not found: {}", site_address))?;

        let host_config = mycart::resolve_ssh_host_for_site(&config, site)
            .ok_or_else(|| anyhow::anyhow!("No SSH host found for site: {}", site_address))?;

        dure_info!(
            "[MycartActor] Using SSH host: {}",
            host_config.host
        );

        // ponytail: keyring loading - pass None for now, SSH uses password from config
        let logs = mycart::get_fail2banrs_logs(host_config, None).await?;

        self.send_event(MycartEvent::Fail2BanRSLogsLoaded { site_address, logs })
            .await;

        Ok(())
    }

    // Phase 5: Operations handler (SQLite-based)
    async fn load_operations(&mut self, site_address: String, limit: i64) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Loading operations for {}", site_address);

        match mycart::load_site_operations(&site_address, limit) {
            Ok(operations) => {
                dure_info!("[MycartActor] Loaded {} operations", operations.len());
                self.send_event(MycartEvent::OperationsLoaded {
                    site_address,
                    operations,
                })
                .await;
                Ok(())
            }
            Err(e) => {
                dure_error!("[MycartActor] Failed to load operations: {}", e);
                Err(e)
            }
        }
    }

    // Phase 4: Crontab management handlers
    async fn list_crontab(&mut self, site_address: String) -> anyhow::Result<()> {
        dure_info!("[MycartActor] Listing crontab for {}", site_address);

        // ponytail: needs JWT token from config - placeholder for now
        // Actual: get token from config, call mycart::list_crontab
        let jwt_token = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No auth token - site needs re-authentication"))?;

        match mycart::list_crontab(&site_address, jwt_token).await {
            Ok(jobs) => {
                dure_info!("[MycartActor] Loaded {} crontab jobs", jobs.len());
                self.send_event(MycartEvent::CrontabLoaded { site_address, jobs })
                    .await;
                Ok(())
            }
            Err(e) => {
                dure_error!("[MycartActor] Failed to load crontab: {}", e);
                Err(e)
            }
        }
    }

    async fn enable_cronjob(&mut self, site_address: String, job_id: String) -> anyhow::Result<()> {
        dure_info!(
            "[MycartActor] Enabling cronjob {} for {}",
            job_id,
            site_address
        );

        let jwt_token = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No auth token"))?;

        match mycart::set_crontab_enabled(&site_address, jwt_token, &job_id, true).await {
            Ok(()) => {
                dure_info!("[MycartActor] Enabled cronjob {}", job_id);
                // Job list will be reloaded by caller
                Ok(())
            }
            Err(e) => {
                dure_error!("[MycartActor] Failed to enable cronjob: {}", e);
                Err(e)
            }
        }
    }

    async fn disable_cronjob(
        &mut self,
        site_address: String,
        job_id: String,
    ) -> anyhow::Result<()> {
        dure_info!(
            "[MycartActor] Disabling cronjob {} for {}",
            job_id,
            site_address
        );

        let jwt_token = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No auth token"))?;

        match mycart::set_crontab_enabled(&site_address, jwt_token, &job_id, false).await {
            Ok(()) => {
                dure_info!("[MycartActor] Disabled cronjob {}", job_id);
                // Job list will be reloaded by caller
                Ok(())
            }
            Err(e) => {
                dure_error!("[MycartActor] Failed to disable cronjob: {}", e);
                Err(e)
            }
        }
    }

    async fn set_interval(
        &mut self,
        site_address: String,
        job_id: String,
        interval: String,
    ) -> anyhow::Result<()> {
        dure_info!(
            "[MycartActor] Setting cronjob {} interval to '{}' for {}",
            job_id,
            interval,
            site_address
        );

        let jwt_token = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No auth token"))?;

        match mycart::set_crontab_interval(&site_address, jwt_token, &job_id, &interval).await {
            Ok(()) => {
                dure_info!("[MycartActor] Updated cronjob {} interval", job_id);
                // Job list will be reloaded by caller
                Ok(())
            }
            Err(e) => {
                dure_error!("[MycartActor] Failed to update cronjob interval: {}", e);
                Err(e)
            }
        }
    }
}
