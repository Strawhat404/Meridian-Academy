use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub order_number: String,
    pub subscription_period: String,
    pub shipping_address_id: Option<String>,
    pub status: String,
    pub payment_status: String,
    pub total_amount: f64,
    pub parent_order_id: Option<String>,
    pub is_flagged: bool,
    pub flag_reason: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLineItem {
    pub id: String,
    pub order_id: String,
    pub publication_title: String,
    pub series_name: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub subscription_period: String,
    pub shipping_address_id: Option<String>,
    pub line_items: Vec<CreateLineItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLineItemRequest {
    pub publication_title: String,
    pub series_name: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SplitOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MergeOrdersRequest {
    pub order_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentEvent {
    pub id: String,
    pub order_id: String,
    pub line_item_id: Option<String>,
    pub event_type: String,
    pub issue_identifier: Option<String>,
    pub reason: String,
    pub expected_date: Option<String>,
    pub actual_date: Option<String>,
    pub logged_by: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFulfillmentEventRequest {
    pub order_id: String,
    pub line_item_id: Option<String>,
    pub event_type: String,
    pub issue_identifier: Option<String>,
    pub reason: String,
    pub expected_date: Option<String>,
    pub actual_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationRecord {
    pub id: String,
    pub order_id: String,
    pub line_item_id: Option<String>,
    pub issue_identifier: String,
    pub expected_qty: i32,
    pub received_qty: i32,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReconciliationRequest {
    pub received_qty: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub line_items: Vec<OrderLineItem>,
}

#[derive(Debug, Deserialize)]
pub struct ClearFlagRequest {
    pub order_id: String,
}
