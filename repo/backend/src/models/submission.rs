use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub submission_type: String,
    pub status: String,
    pub deadline: Option<NaiveDateTime>,
    pub current_version: i32,
    pub max_versions: i32,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<String>,
    pub keywords: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionVersion {
    pub id: String,
    pub submission_id: String,
    pub version_number: i32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_type: String,
    pub file_hash: String,
    pub magic_bytes: Option<String>,
    pub form_data: Option<String>,
    pub submitted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub title: String,
    pub summary: Option<String>,
    pub submission_type: String,
    pub deadline: Option<String>,
    pub tags: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubmissionRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitVersionRequest {
    pub file_name: String,
    pub file_data: String, // base64 encoded
    pub form_data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionVersionResponse {
    pub id: String,
    pub version_number: i32,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub file_hash: String,
    pub submitted_at: Option<String>, // MM/DD/YYYY 12-hour format
}

impl SubmissionVersion {
    pub fn to_response(&self) -> SubmissionVersionResponse {
        let submitted_str = self.submitted_at.map(|dt| {
            dt.format("%m/%d/%Y, %I:%M:%S %p").to_string()
        });
        SubmissionVersionResponse {
            id: self.id.clone(),
            version_number: self.version_number,
            file_name: self.file_name.clone(),
            file_size: self.file_size,
            file_type: self.file_type.clone(),
            file_hash: self.file_hash.clone(),
            submitted_at: submitted_str,
        }
    }
}
