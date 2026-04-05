use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub contact_info: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub soft_deleted_at: Option<NaiveDateTime>,
    pub deletion_scheduled_at: Option<NaiveDateTime>,
    pub invoice_title: Option<String>,
    pub notify_submissions: bool,
    pub notify_orders: bool,
    pub notify_reviews: bool,
    pub notify_cases: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub contact_info: Option<String>,
    pub invoice_title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPrefsRequest {
    pub notify_submissions: Option<bool>,
    pub notify_orders: Option<bool>,
    pub notify_reviews: Option<bool>,
    pub notify_cases: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub contact_info: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub invoice_title: Option<String>,
    pub notify_submissions: bool,
    pub notify_orders: bool,
    pub notify_reviews: bool,
    pub notify_cases: bool,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAddress {
    pub id: String,
    pub user_id: String,
    pub label: String,
    pub street_line1: String,
    pub street_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateAddressRequest {
    pub label: String,
    pub street_line1: String,
    pub street_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultAddressRequest {
    pub address_id: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationItem {
    pub id: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: Option<NaiveDateTime>,
}
