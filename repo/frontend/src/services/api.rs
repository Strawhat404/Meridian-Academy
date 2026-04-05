use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

const BACKEND_URL: &str = "http://localhost:8000";

fn auth_header() -> Option<String> {
    LocalStorage::get::<String>("auth_token")
        .ok()
        .map(|t| format!("Bearer {}", t))
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::get(&url);

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Request failed with status: {}", resp.status()))
    }
}

pub async fn post<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::post(&url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = req.body(body_str).unwrap().send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Request failed with status: {}", resp.status()))
    }
}

/// POST that expects an empty body (200/201/204 with no JSON).
pub async fn post_empty<B: Serialize>(path: &str, body: &B) -> Result<(), String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::post(&url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = req.body(body_str).unwrap().send().await.map_err(|e| e.to_string())?;

    if resp.ok() { Ok(()) } else { Err(format!("Request failed with status: {}", resp.status())) }
}

pub async fn put<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::put(&url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = req.body(body_str).unwrap().send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Request failed with status: {}", resp.status()))
    }
}

/// PUT that expects an empty body (200/204 with no JSON).
pub async fn put_empty<B: Serialize>(path: &str, body: &B) -> Result<(), String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::put(&url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = req.body(body_str).unwrap().send().await.map_err(|e| e.to_string())?;

    if resp.ok() { Ok(()) } else { Err(format!("Request failed with status: {}", resp.status())) }
}

pub async fn delete(path: &str) -> Result<(), String> {
    let url = format!("{}{}", BACKEND_URL, path);
    let mut req = Request::delete(&url);

    if let Some(token) = auth_header() {
        req = req.header("Authorization", &token);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Request failed with status: {}", resp.status()))
    }
}
