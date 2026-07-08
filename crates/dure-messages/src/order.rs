//! Order management message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::product::ProductPrice;

/// Create order request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderCreateRequest {
    /// Server/store ID to order from
    pub server_id: String,
    /// List of product IDs
    pub product_ids: Vec<String>,
    /// Quantities for each product (parallel array)
    pub counts: Vec<u32>,
    /// Shipping address
    pub shipping_address: ShippingAddress,
    /// Optional order notes
    pub notes: Option<String>,
}

/// Shipping address
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ShippingAddress {
    /// Recipient name
    pub recipient_name: String,
    /// Phone number
    pub phone: String,
    /// Address line 1
    pub address_line1: String,
    /// Address line 2
    pub address_line2: Option<String>,
    /// City
    pub city: String,
    /// State/Province
    pub state: Option<String>,
    /// Postal/ZIP code
    pub postal_code: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
}

/// Order created response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderCreatedResponse {
    /// Whether creation was successful
    pub success: bool,
    /// Created order ID
    pub order_id: Option<String>,
    /// Order details
    pub order: Option<OrderData>,
    /// Error if failed
    pub error: Option<String>,
}

/// Order data
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderData {
    /// Order ID
    pub order_id: String,
    /// Server/store ID
    pub server_id: String,
    /// Customer member ID
    pub customer_id: String,
    /// Order items
    pub items: Vec<OrderItem>,
    /// Total price
    pub total_price: ProductPrice,
    /// Shipping address
    pub shipping_address: ShippingAddress,
    /// Order status
    pub status: OrderStatus,
    /// Order notes
    pub notes: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Channel ID for order communication
    pub channel_id: Option<String>,
}

/// Order item
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderItem {
    /// Product ID
    pub product_id: String,
    /// Product name (snapshot at order time)
    pub product_name: String,
    /// Quantity ordered
    pub quantity: u32,
    /// Unit price at order time
    pub unit_price: ProductPrice,
    /// Subtotal for this item
    pub subtotal: ProductPrice,
}

/// Order status
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order pending approval
    Pending,
    /// Order processing
    Processing,
    /// Payment completed
    Paid,
    /// Order shipped
    Shipped,
    /// Order delivered
    Delivered,
    /// Order cancelled
    Cancelled,
    /// Order refunded
    Refunded,
}

/// List orders request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderListRequest {
    /// Server ID (optional - if empty, lists all orders)
    pub server_id: Option<String>,
    /// Filter by status (optional)
    pub status: Option<OrderStatus>,
    /// Limit
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// List orders response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderListResponse {
    /// List of orders
    pub orders: Vec<OrderData>,
    /// Total count
    pub total: usize,
    /// Whether there are more orders
    pub has_more: bool,
}

/// Order status update notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct OrderStatusUpdateNotification {
    /// Order ID
    pub order_id: String,
    /// Server ID
    pub server_id: String,
    /// New status
    pub status: OrderStatus,
    /// Previous status
    pub previous_status: OrderStatus,
    /// Who updated the status
    pub updated_by: String,
    /// Optional message
    pub message: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Tracking number (if shipped)
    pub tracking_number: Option<String>,
}
