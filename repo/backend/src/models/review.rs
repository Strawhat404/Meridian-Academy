use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub order_id: String,
    pub line_item_id: Option<String>,
    pub user_id: String,
    pub rating: i32,
    pub title: String,
    pub body: String,
    pub is_followup: bool,
    pub parent_review_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewImage {
    pub id: String,
    pub review_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub order_id: String,
    pub line_item_id: Option<String>,
    pub rating: i32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowupRequest {
    pub parent_review_id: String,
    pub rating: i32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct AddReviewImageRequest {
    pub file_name: String,
    pub file_data: String, // base64
}
