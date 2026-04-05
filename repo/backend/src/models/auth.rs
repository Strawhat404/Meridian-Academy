use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: super::user::UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResetTokenRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct ResetTokenResponse {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UseResetTokenRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestAccountDeletionRequest {}

#[derive(Debug, Deserialize)]
pub struct CancelDeletionRequest {}

#[derive(Debug, Serialize)]
pub struct ExportDataResponse {
    pub user_profile: serde_json::Value,
    pub addresses: Vec<serde_json::Value>,
    pub submissions: Vec<serde_json::Value>,
    pub orders: Vec<serde_json::Value>,
    pub reviews: Vec<serde_json::Value>,
    pub cases: Vec<serde_json::Value>,
    pub exported_at: String,
}
