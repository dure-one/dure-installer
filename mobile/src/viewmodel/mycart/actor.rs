//! Mycart actor implementation

use super::commands::{LogType, MycartCommand};
use super::events::{MycartEvent, Order, Product, StoreHealth};
use crate::viewmodel::common::ViewModelEvent;
use crate::viewmodel::runtime;
use crate::{dure_debug, dure_error, dure_info};
use smol::channel::{Receiver, Sender};

pub struct MycartActor {
    command_rx: Receiver<MycartCommand>,
    event_tx: Sender<ViewModelEvent>,
    base_url: String,
    auth_token: Option<String>,
}

impl MycartActor {
    pub fn new(command_rx: Receiver<MycartCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
            base_url: "https://mycart.dure.app/api/_".to_string(), // ponytail: placeholder URL
            auth_token: None,
        }
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
            MycartCommand::CheckStoreHealth { store_id } => {
                self.check_store_health(store_id).await
            }

            MycartCommand::ListProducts {
                store_id,
                limit,
                offset,
            } => self.list_products(store_id, limit, offset).await,

            MycartCommand::GetProduct { product_id } => self.get_product(product_id).await,

            MycartCommand::CreateProduct {
                store_id,
                name,
                price,
                description,
            } => {
                self.create_product(store_id, name, price, description)
                    .await
            }

            MycartCommand::UpdateProduct {
                product_id,
                name,
                price,
            } => self.update_product(product_id, name, price).await,

            MycartCommand::DeleteProduct { product_id } => self.delete_product(product_id).await,

            MycartCommand::ListOrders {
                store_id,
                status,
                limit,
            } => self.list_orders(store_id, status, limit).await,

            MycartCommand::GetOrder { order_id } => self.get_order(order_id).await,

            MycartCommand::UpdateOrderStatus { order_id, status } => {
                self.update_order_status(order_id, status).await
            }

            MycartCommand::FetchLogs {
                store_id,
                vm_name,
                log_type,
                lines,
            } => self.fetch_logs(store_id, vm_name, log_type, lines).await,

            MycartCommand::StreamLogs {
                store_id,
                vm_name,
                log_type,
            } => self.stream_logs(store_id, vm_name, log_type).await,

            MycartCommand::StopLogStream { store_id } => self.stop_log_stream(store_id).await,
        }
    }

    async fn send_event(&self, event: MycartEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Mycart(event)).await;
    }

    // ponytail: stub implementations, replace with actual API calls when backend ready
    async fn check_store_health(&mut self, store_id: String) -> anyhow::Result<()> {
        let health = StoreHealth {
            store_id: store_id.clone(),
            status: "healthy".to_string(),
            uptime_seconds: 0,
            version: "0.0.0".to_string(),
        };

        self.send_event(MycartEvent::StoreHealthChecked { store_id, health })
            .await;
        Ok(())
    }

    async fn list_products(
        &mut self,
        store_id: String,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> anyhow::Result<()> {
        // ponytail: placeholder, replace with actual HTTP call
        let products = Vec::new();

        self.send_event(MycartEvent::ProductsListed {
            store_id,
            total: products.len(),
            products,
        })
        .await;
        Ok(())
    }

    async fn get_product(&mut self, product_id: String) -> anyhow::Result<()> {
        // ponytail: placeholder
        let product = Product {
            id: product_id,
            store_id: "unknown".to_string(),
            name: "Unknown".to_string(),
            price: 0,
            description: None,
            created_at: 0,
        };

        self.send_event(MycartEvent::ProductRetrieved { product })
            .await;
        Ok(())
    }

    async fn create_product(
        &mut self,
        store_id: String,
        name: String,
        price: i64,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        // ponytail: placeholder
        let product = Product {
            id: "new-id".to_string(),
            store_id,
            name,
            price,
            description,
            created_at: chrono::Utc::now().timestamp(),
        };

        self.send_event(MycartEvent::ProductCreated { product })
            .await;
        Ok(())
    }

    async fn update_product(
        &mut self,
        product_id: String,
        _name: Option<String>,
        _price: Option<i64>,
    ) -> anyhow::Result<()> {
        // ponytail: placeholder
        self.send_event(MycartEvent::ProductUpdated { product_id })
            .await;
        Ok(())
    }

    async fn delete_product(&mut self, product_id: String) -> anyhow::Result<()> {
        // ponytail: placeholder
        self.send_event(MycartEvent::ProductDeleted { product_id })
            .await;
        Ok(())
    }

    async fn list_orders(
        &mut self,
        store_id: String,
        _status: Option<String>,
        _limit: Option<usize>,
    ) -> anyhow::Result<()> {
        // ponytail: placeholder
        let orders = Vec::new();

        self.send_event(MycartEvent::OrdersListed {
            store_id,
            total: orders.len(),
            orders,
        })
        .await;
        Ok(())
    }

    async fn get_order(&mut self, order_id: String) -> anyhow::Result<()> {
        // ponytail: placeholder
        let order = Order {
            id: order_id,
            store_id: "unknown".to_string(),
            customer_name: "Unknown".to_string(),
            status: "pending".to_string(),
            total_amount: 0,
            created_at: 0,
        };

        self.send_event(MycartEvent::OrderRetrieved { order }).await;
        Ok(())
    }

    async fn update_order_status(
        &mut self,
        order_id: String,
        status: String,
    ) -> anyhow::Result<()> {
        // ponytail: placeholder
        self.send_event(MycartEvent::OrderStatusUpdated { order_id, status })
            .await;
        Ok(())
    }

    async fn fetch_logs(
        &mut self,
        store_id: String,
        vm_name: String,
        log_type: LogType,
        lines: usize,
    ) -> anyhow::Result<()> {
        let store_id_clone = store_id.clone();
        let log_type_clone = log_type;

        let log_lines = runtime::unblock(move || -> anyhow::Result<Vec<String>> {
            // ponytail: SSH log fetching - requires calc::ssh integration
            let log_path = log_type_clone.log_path();
            let cmd = format!("tail -n {} {}", lines, log_path);

            // For now, return empty - actual implementation needs SSH connection
            dure_debug!("[MycartActor] Would fetch logs from {} via SSH: {}", vm_name, cmd);
            Ok(Vec::new())
        })
        .await?;

        self.send_event(MycartEvent::LogsFetched {
            store_id,
            log_type,
            lines: log_lines,
        })
        .await;
        Ok(())
    }

    async fn stream_logs(
        &mut self,
        store_id: String,
        vm_name: String,
        log_type: LogType,
    ) -> anyhow::Result<()> {
        // ponytail: log streaming - requires async SSH stream
        dure_debug!(
            "[MycartActor] Would stream logs from {} ({})",
            vm_name,
            log_type.log_path()
        );

        // For now, just send stopped event
        self.send_event(MycartEvent::LogStreamStopped { store_id })
            .await;
        Ok(())
    }

    async fn stop_log_stream(&mut self, store_id: String) -> anyhow::Result<()> {
        self.send_event(MycartEvent::LogStreamStopped { store_id })
            .await;
        Ok(())
    }
}
