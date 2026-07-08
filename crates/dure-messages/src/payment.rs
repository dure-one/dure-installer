//! Payment processing message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::product::ProductPrice;

/// Create payment request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentCreateRequest {
    /// Server ID
    pub server_id: String,
    /// Order ID to pay for
    pub order_id: String,
    /// Payment method
    pub payment_method: PaymentMethod,
    /// Return URL after payment
    pub return_url: Option<String>,
}

/// Payment method
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// Portone payment gateway
    Portone {
        /// Portone merchant ID
        merchant_id: String,
    },
    /// KakaoPay
    KakaoPay {
        /// KakaoPay merchant ID
        merchant_id: String,
    },
    /// Credit card
    CreditCard {
        /// Card token (PCI compliant)
        card_token: String,
    },
    /// Bank transfer
    BankTransfer {
        /// Bank code
        bank_code: String,
    },
}

/// Payment created response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentCreatedResponse {
    /// Whether creation was successful
    pub success: bool,
    /// Payment ID
    pub payment_id: Option<String>,
    /// Payment gateway URL (for redirect)
    pub payment_url: Option<String>,
    /// Payment details
    pub payment: Option<PaymentData>,
    /// Error if failed
    pub error: Option<String>,
}

/// Payment data
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentData {
    /// Payment ID
    pub payment_id: String,
    /// Order ID
    pub order_id: String,
    /// Server ID
    pub server_id: String,
    /// Customer ID
    pub customer_id: String,
    /// Payment amount
    pub amount: ProductPrice,
    /// Payment method
    pub payment_method: PaymentMethod,
    /// Payment status
    pub status: PaymentStatus,
    /// Payment gateway transaction ID
    pub transaction_id: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Payment status
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Payment pending
    Pending,
    /// Payment processing
    Processing,
    /// Payment completed
    Completed,
    /// Payment failed
    Failed,
    /// Payment cancelled
    Cancelled,
    /// Payment refunded
    Refunded,
}

/// Verify payment request (from payment gateway webhook)
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentVerifyRequest {
    /// Server ID
    pub server_id: String,
    /// Payment ID
    pub payment_id: String,
    /// Payment gateway verification data (JSON)
    pub verification_data: serde_json::Value,
}

/// Payment verified response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentVerifiedResponse {
    /// Whether verification was successful
    pub success: bool,
    /// Payment ID
    pub payment_id: Option<String>,
    /// Payment details
    pub payment: Option<PaymentData>,
    /// Error if failed
    pub error: Option<String>,
}

/// List payments request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentListRequest {
    /// Server ID (optional)
    pub server_id: Option<String>,
    /// Filter by order ID (optional)
    pub order_id: Option<String>,
    /// Filter by status (optional)
    pub status: Option<PaymentStatus>,
    /// Limit
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// List payments response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PaymentListResponse {
    /// List of payments
    pub payments: Vec<PaymentData>,
    /// Total count
    pub total: usize,
    /// Whether there are more payments
    pub has_more: bool,
}
