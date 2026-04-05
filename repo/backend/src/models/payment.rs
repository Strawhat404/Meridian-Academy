use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub order_id: String,
    pub idempotency_key: String,
    pub payment_method: String,
    pub amount: f64,
    pub transaction_type: String,
    pub reference_payment_id: Option<String>,
    pub status: String,
    pub check_number: Option<String>,
    pub notes: Option<String>,
    pub processed_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub order_id: String,
    pub idempotency_key: String,
    pub payment_method: String,
    pub amount: f64,
    pub transaction_type: String,
    pub reference_payment_id: Option<String>,
    pub check_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefundPaymentRequest {
    pub original_payment_id: String,
    pub idempotency_key: String,
    pub amount: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReconciliationReport {
    pub id: String,
    pub report_date: String,
    pub expected_balance: f64,
    pub actual_balance: f64,
    pub discrepancy: f64,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AbnormalOrderFlag {
    pub id: String,
    pub order_id: Option<String>,
    pub user_id: Option<String>,
    pub flag_type: String,
    pub reason: String,
    pub is_cleared: bool,
    pub cleared_by: Option<String>,
    pub cleared_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}
