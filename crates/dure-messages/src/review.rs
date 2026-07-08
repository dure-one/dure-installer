//! Review management message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Create review request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewCreateRequest {
    /// Server/store ID being reviewed
    pub server_id: String,
    /// Order ID this review is for
    pub order_id: String,
    /// Product ID being reviewed (optional, if specific product)
    pub product_id: Option<String>,
    /// Rating (1-5 stars)
    pub rating: u8,
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Optional images (base64 encoded)
    pub images: Option<Vec<ReviewImage>>,
}

/// Review image
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewImage {
    /// Image filename
    pub filename: String,
    /// Image data (base64 encoded)
    pub data: String,
    /// Image MIME type
    pub content_type: String,
}

/// Review created response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewCreatedResponse {
    /// Whether creation was successful
    pub success: bool,
    /// Created review ID
    pub review_id: Option<String>,
    /// Review details
    pub review: Option<ReviewData>,
    /// Error if failed
    pub error: Option<String>,
}

/// Review data
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewData {
    /// Review ID
    pub review_id: String,
    /// Server ID
    pub server_id: String,
    /// Order ID
    pub order_id: String,
    /// Product ID (optional)
    pub product_id: Option<String>,
    /// Reviewer member ID
    pub reviewer_id: String,
    /// Reviewer display name
    pub reviewer_name: Option<String>,
    /// Rating (1-5)
    pub rating: u8,
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Image URLs
    pub image_urls: Option<Vec<String>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Store owner reply
    pub reply: Option<ReviewReply>,
    /// Whether review is verified purchase
    pub is_verified_purchase: bool,
    /// Helpfulness count
    pub helpful_count: u32,
}

/// Store owner reply to review
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewReply {
    /// Reply content
    pub content: String,
    /// Who replied
    pub replied_by: String,
    /// Reply timestamp
    pub replied_at: DateTime<Utc>,
}

/// List reviews request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewListRequest {
    /// Server ID to list reviews for
    pub server_id: String,
    /// Filter by product ID (optional)
    pub product_id: Option<String>,
    /// Filter by minimum rating (optional)
    pub min_rating: Option<u8>,
    /// Sort order
    pub sort_by: Option<ReviewSortBy>,
    /// Limit
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Review sort options
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSortBy {
    /// Most recent first
    Recent,
    /// Highest rated first
    HighestRated,
    /// Lowest rated first
    LowestRated,
    /// Most helpful first
    MostHelpful,
}

/// List reviews response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ReviewListResponse {
    /// List of reviews
    pub reviews: Vec<ReviewData>,
    /// Total count
    pub total: usize,
    /// Average rating
    pub average_rating: Option<f32>,
    /// Rating distribution (1-5 stars)
    pub rating_distribution: Option<RatingDistribution>,
    /// Whether there are more reviews
    pub has_more: bool,
}

/// Rating distribution
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct RatingDistribution {
    /// Number of 5-star reviews
    pub five_stars: u32,
    /// Number of 4-star reviews
    pub four_stars: u32,
    /// Number of 3-star reviews
    pub three_stars: u32,
    /// Number of 2-star reviews
    pub two_stars: u32,
    /// Number of 1-star reviews
    pub one_star: u32,
}
