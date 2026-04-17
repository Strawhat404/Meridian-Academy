use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfterSalesCase {
    pub id: String,
    pub order_id: String,
    pub reporter_id: String,
    pub assigned_to: Option<String>,
    pub case_type: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub submitted_at: Option<NaiveDateTime>,
    pub first_response_at: Option<NaiveDateTime>,
    pub first_response_due: Option<NaiveDateTime>,
    pub resolution_target: Option<NaiveDateTime>,
    pub resolved_at: Option<NaiveDateTime>,
    pub closed_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseWithSla {
    pub case: AfterSalesCase,
    pub first_response_overdue: bool,
    pub resolution_overdue: bool,
    pub hours_until_first_response: Option<f64>,
    pub hours_until_resolution: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseRequest {
    pub order_id: String,
    pub case_type: String,
    pub subject: String,
    pub description: String,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCaseStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignCaseRequest {
    pub assigned_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseComment {
    pub id: String,
    pub case_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseCommentRequest {
    pub content: String,
}
