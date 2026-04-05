use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use super::api;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub is_active: bool,
    pub invoice_title: Option<String>,
    pub notify_submissions: bool,
    pub notify_orders: bool,
    pub notify_reviews: bool,
    pub notify_cases: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

pub async fn login(username: &str, password: &str) -> Result<UserInfo, String> {
    let req = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let resp: LoginResponse = api::post("/api/auth/login", &req).await?;

    LocalStorage::set("auth_token", &resp.token).map_err(|e| e.to_string())?;
    LocalStorage::set("current_user", &resp.user).map_err(|e| e.to_string())?;

    Ok(resp.user)
}

/// Admin-provisioned account creation.
pub async fn register(data: &RegisterData) -> Result<UserInfo, String> {
    let resp: LoginResponse = api::post("/api/auth/provision", data).await?;

    LocalStorage::set("auth_token", &resp.token).map_err(|e| e.to_string())?;
    LocalStorage::set("current_user", &resp.user).map_err(|e| e.to_string())?;

    Ok(resp.user)
}

pub fn get_current_user() -> Option<UserInfo> {
    LocalStorage::get::<UserInfo>("current_user").ok()
}

pub fn get_token() -> Option<String> {
    LocalStorage::get::<String>("auth_token").ok()
}

pub fn logout() {
    let _ = LocalStorage::delete("auth_token");
    let _ = LocalStorage::delete("current_user");
}

pub fn is_authenticated() -> bool {
    get_token().is_some()
}
