//! Product management message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Create product request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductCreateRequest {
    /// Server ID
    pub server_id: String,
    /// Product name
    pub name: String,
    /// Product category
    pub category: String,
    /// Product image URL or base64 data
    pub image: String,
    /// Product description/contents
    pub contents: String,
    /// Product price
    pub price: ProductPrice,
    /// Product stock quantity
    pub stock: u32,
    /// Product SKU
    pub sku: Option<String>,
}

/// Product price information
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductPrice {
    /// Price amount
    pub amount: f64,
    /// Currency code (USD, KRW, etc.)
    pub currency: String,
    /// Optional discount percentage
    pub discount_percent: Option<f32>,
}

/// Product created response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductCreatedResponse {
    /// Whether creation was successful
    pub success: bool,
    /// Created product ID
    pub product_id: Option<String>,
    /// Product details
    pub product: Option<ProductData>,
    /// Error if failed
    pub error: Option<String>,
}

/// Product data
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductData {
    /// Product ID
    pub product_id: String,
    /// Server ID
    pub server_id: String,
    /// Product name
    pub name: String,
    /// Product category
    pub category: String,
    /// Product image URL
    pub image_url: String,
    /// Product description
    pub description: String,
    /// Product price
    pub price: ProductPrice,
    /// Stock quantity
    pub stock: u32,
    /// SKU
    pub sku: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether product is available
    pub is_available: bool,
}

/// List products request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductListRequest {
    /// Server ID
    pub server_id: String,
    /// Optional category filter
    pub category: Option<String>,
    /// Optional search query
    pub search: Option<String>,
    /// Limit
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// List products response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductListResponse {
    /// List of products
    pub products: Vec<ProductData>,
    /// Total count
    pub total: usize,
    /// Whether there are more products
    pub has_more: bool,
}

/// Modify product request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductModifyRequest {
    /// Server ID
    pub server_id: String,
    /// Product ID to modify
    pub product_id: String,
    /// New name (optional)
    pub name: Option<String>,
    /// New category (optional)
    pub category: Option<String>,
    /// New image (optional)
    pub image: Option<String>,
    /// New description (optional)
    pub contents: Option<String>,
    /// New price (optional)
    pub price: Option<ProductPrice>,
    /// New stock (optional)
    pub stock: Option<u32>,
    /// New availability (optional)
    pub is_available: Option<bool>,
}

/// Product modified notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductModifiedNotification {
    /// Modified product data
    pub product: ProductData,
    /// Who modified the product
    pub modified_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Delete product request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductDeleteRequest {
    /// Server ID
    pub server_id: String,
    /// Product ID to delete
    pub product_id: String,
    /// Confirmation flag
    pub confirm: bool,
}

/// Product deleted notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ProductDeletedNotification {
    /// Deleted product ID
    pub product_id: String,
    /// Server ID
    pub server_id: String,
    /// Who deleted the product
    pub deleted_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
