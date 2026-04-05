pub mod auth;
pub mod case;
pub mod content;
pub mod order;
pub mod payment;
pub mod review;
pub mod submission;
pub mod user;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_token_expiry")]
    pub token_expiry_hours: u64,
    #[serde(default = "default_session_timeout")]
    pub session_timeout_minutes: u64,
}

fn default_jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set")
}

fn default_token_expiry() -> u64 {
    std::env::var("TOKEN_EXPIRY_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24)
}

fn default_session_timeout() -> u64 {
    std::env::var("SESSION_TIMEOUT_MINUTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

/// Generates SEO metadata from title and summary deterministically
pub fn generate_seo(title: &str, summary: Option<&str>) -> (String, String, String) {
    let meta_title = if title.len() > 120 {
        title[..120].to_string()
    } else {
        title.to_string()
    };

    let meta_description = match summary {
        Some(s) if s.len() > 155 => s[..155].to_string(),
        Some(s) => s.to_string(),
        None if title.len() > 155 => title[..155].to_string(),
        None => title.to_string(),
    };

    let slug_val = slug::slugify(title);

    (meta_title, meta_description, slug_val)
}

/// Validates content metadata lengths
pub fn validate_metadata(title: &str, summary: Option<&str>, tags: Option<&str>, keywords: Option<&str>) -> Result<(), String> {
    if title.len() > 120 {
        return Err("Title must be 120 characters or fewer".to_string());
    }
    if let Some(s) = summary {
        if s.len() > 500 {
            return Err("Summary must be 500 characters or fewer".to_string());
        }
    }
    if let Some(t) = tags {
        if t.len() > 1000 {
            return Err("Tags field exceeds maximum length of 1000 characters".to_string());
        }
        for tag in t.split(',') {
            if tag.trim().len() > 50 {
                return Err("Individual tag must be 50 characters or fewer".to_string());
            }
        }
    }
    if let Some(k) = keywords {
        if k.len() > 1000 {
            return Err("Keywords field exceeds maximum length of 1000 characters".to_string());
        }
        for kw in k.split(',') {
            if kw.trim().len() > 50 {
                return Err("Individual keyword must be 50 characters or fewer".to_string());
            }
        }
    }
    Ok(())
}

/// File-type allowlist check
pub fn validate_file_type(filename: &str, magic_bytes: &[u8]) -> Result<String, String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    let allowed = match ext.as_str() {
        "pdf" => magic_bytes.starts_with(b"%PDF"),
        "docx" => magic_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "png" => magic_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        "jpg" | "jpeg" => magic_bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        _ => return Err(format!("File type '{}' not allowed. Allowed: PDF, DOCX, PNG, JPG", ext)),
    };

    if !allowed {
        return Err(format!("File signature does not match extension '{}'", ext));
    }

    Ok(ext)
}

pub const MAX_FILE_SIZE: u64 = 25 * 1024 * 1024; // 25 MB
pub const MAX_REVIEW_IMAGE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
pub const MAX_REVIEW_IMAGES: usize = 6;
pub const FOLLOWUP_WINDOW_DAYS: i64 = 14;
pub const SLA_FIRST_RESPONSE_HOURS: i64 = 48; // 2 business days
pub const SLA_RESOLUTION_HOURS: i64 = 168; // 7 business days
pub const SOFT_DELETE_HOLD_DAYS: i64 = 30;
pub const PASSWORD_RESET_EXPIRY_MINUTES: i64 = 60;
pub const SESSION_IDLE_TIMEOUT_MINUTES: i64 = 30;
pub const MAX_SUBMISSION_VERSIONS: i32 = 10;
pub const HIGH_QUANTITY_THRESHOLD: i32 = 50;
pub const REFUND_COUNT_THRESHOLD: i64 = 3;

/// Add N business days (Mon–Fri) to a datetime, skipping weekends.
pub fn add_business_days(from: chrono::NaiveDateTime, days: u32) -> chrono::NaiveDateTime {
    use chrono::Datelike;
    let mut result = from;
    let mut remaining = days;
    while remaining > 0 {
        result += chrono::Duration::days(1);
        match result.weekday() {
            chrono::Weekday::Sat | chrono::Weekday::Sun => {}
            _ => remaining -= 1,
        }
    }
    result
}

/// Case status transition validation
pub fn valid_case_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("submitted", "in_review")
            | ("in_review", "awaiting_evidence")
            | ("in_review", "arbitrated")
            | ("awaiting_evidence", "in_review")
            | ("awaiting_evidence", "arbitrated")
            | ("arbitrated", "approved")
            | ("arbitrated", "denied")
            | ("approved", "closed")
            | ("denied", "closed")
    )
}
