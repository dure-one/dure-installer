//! Mycart actor commands

#[derive(Debug, Clone)]
pub enum MycartCommand {
    // Store Health
    CheckStoreHealth {
        store_id: String,
    },

    // Product Operations
    ListProducts {
        store_id: String,
        limit: Option<usize>,
        offset: Option<usize>,
    },

    GetProduct {
        product_id: String,
    },

    CreateProduct {
        store_id: String,
        name: String,
        price: i64,
        description: Option<String>,
    },

    UpdateProduct {
        product_id: String,
        name: Option<String>,
        price: Option<i64>,
    },

    DeleteProduct {
        product_id: String,
    },

    // Order Operations
    ListOrders {
        store_id: String,
        status: Option<String>,
        limit: Option<usize>,
    },

    GetOrder {
        order_id: String,
    },

    UpdateOrderStatus {
        order_id: String,
        status: String,
    },

    // SSH Log Operations
    FetchLogs {
        store_id: String,
        vm_name: String,
        log_type: LogType,
        lines: usize,
    },

    StreamLogs {
        store_id: String,
        vm_name: String,
        log_type: LogType,
    },

    StopLogStream {
        store_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogType {
    Application,
    Nginx,
    System,
}

impl LogType {
    pub fn log_path(&self) -> &'static str {
        match self {
            LogType::Application => "/var/log/mycart/app.log",
            LogType::Nginx => "/var/log/nginx/access.log",
            LogType::System => "/var/log/syslog",
        }
    }
}
