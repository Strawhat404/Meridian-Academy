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

/// Predefined submission templates that guide authors through type-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionTemplate {
    pub id: String,
    pub name: String,
    pub submission_type: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub description: String,
}

/// Returns the built-in set of guided submission templates.
pub fn get_submission_templates() -> Vec<SubmissionTemplate> {
    vec![
        SubmissionTemplate {
            id: "tpl-journal".to_string(),
            name: "Journal Article".to_string(),
            submission_type: "journal_article".to_string(),
            required_fields: vec!["title".into(), "abstract".into(), "keywords".into(), "methodology".into()],
            optional_fields: vec!["acknowledgments".into(), "funding_source".into()],
            description: "Standard template for peer-reviewed journal article submissions.".to_string(),
        },
        SubmissionTemplate {
            id: "tpl-conference".to_string(),
            name: "Conference Paper".to_string(),
            submission_type: "conference_paper".to_string(),
            required_fields: vec!["title".into(), "abstract".into(), "keywords".into(), "conference_name".into()],
            optional_fields: vec!["presentation_type".into(), "co_authors".into()],
            description: "Template for conference paper submissions with session details.".to_string(),
        },
        SubmissionTemplate {
            id: "tpl-thesis".to_string(),
            name: "Thesis".to_string(),
            submission_type: "thesis".to_string(),
            required_fields: vec!["title".into(), "abstract".into(), "department".into(), "advisor".into(), "degree_type".into()],
            optional_fields: vec!["committee_members".into(), "defense_date".into()],
            description: "Template for thesis/dissertation submissions with committee details.".to_string(),
        },
        SubmissionTemplate {
            id: "tpl-book-chapter".to_string(),
            name: "Book Chapter".to_string(),
            submission_type: "book_chapter".to_string(),
            required_fields: vec!["title".into(), "abstract".into(), "book_title".into(), "editor".into()],
            optional_fields: vec!["chapter_number".into(), "isbn".into()],
            description: "Template for contributed book chapter submissions.".to_string(),
        },
    ]
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub title: String,
    pub summary: Option<String>,
    pub submission_type: String,
    pub template_id: Option<String>,
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
