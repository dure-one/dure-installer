//! Mycart actor events

use super::commands::LogType;

#[derive(Debug, Clone)]
pub struct StoreHealth {
    pub store_id: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub price: i64,
    pub description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub store_id: String,
    pub customer_name: String,
    pub status: String,
    pub total_amount: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub enum MycartEvent {
    // Health
    StoreHealthChecked {
        store_id: String,
        health: StoreHealth,
    },

    // Product Events
    ProductsListed {
        store_id: String,
        products: Vec<Product>,
        total: usize,
    },

    ProductRetrieved {
        product: Product,
    },

    ProductCreated {
        product: Product,
    },

    ProductUpdated {
        product_id: String,
    },

    ProductDeleted {
        product_id: String,
    },

    // Order Events
    OrdersListed {
        store_id: String,
        orders: Vec<Order>,
        total: usize,
    },

    OrderRetrieved {
        order: Order,
    },

    OrderStatusUpdated {
        order_id: String,
        status: String,
    },

    // Log Events
    LogsFetched {
        store_id: String,
        log_type: LogType,
        lines: Vec<String>,
    },

    LogLine {
        store_id: String,
        line: String,
    },

    LogStreamStopped {
        store_id: String,
    },

    Error {
        operation: String,
        error: String,
    },
}
