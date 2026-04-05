use crate::middleware::AuthenticatedUser;
use crate::models;
use crate::DbPool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::json;

#[get("/dashboard")]
pub async fn dashboard_stats(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
    user.require_permission("admin.dashboard")?;

    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE soft_deleted_at IS NULL")
        .fetch_one(pool.inner()).await.unwrap_or(0);
    let submission_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM submissions")
        .fetch_one(pool.inner()).await.unwrap_or(0);
    let order_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(pool.inner()).await.unwrap_or(0);
    let pending_cases = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM after_sales_cases WHERE status IN ('submitted', 'in_review', 'awaiting_evidence')")
        .fetch_one(pool.inner()).await.unwrap_or(0);
    let flagged_orders = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders WHERE is_flagged = true")
        .fetch_one(pool.inner()).await.unwrap_or(0);
    let total_revenue: f64 = sqlx::query_scalar::<_, f64>("SELECT COALESCE(CAST(SUM(amount) AS DOUBLE), 0) FROM payments WHERE transaction_type = 'charge' AND status = 'completed'")
        .fetch_one(pool.inner()).await.unwrap_or(0.0);
    let blocked_content = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM submissions WHERE status = 'blocked'")
        .fetch_one(pool.inner()).await.unwrap_or(0);

    Ok(Json(json!({
        "total_users": user_count,
        "total_submissions": submission_count,
        "total_orders": order_count,
        "pending_cases": pending_cases,
        "flagged_orders": flagged_orders,
        "total_revenue": total_revenue,
        "blocked_content": blocked_content
    })))
}

#[get("/audit-log")]
pub async fn audit_log(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
    user.require_permission("admin.audit")?;

    let rows = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, user_id, action, target_type, target_id, details, created_at FROM audit_log ORDER BY created_at DESC LIMIT 200"
    )
    .fetch_all(pool.inner()).await.map_err(|_| Status::InternalServerError)?;

    let logs: Vec<serde_json::Value> = rows.into_iter().map(|(id, uid, action, tt, tid, details, ca)| {
        json!({ "id": id, "user_id": uid, "action": action, "target_type": tt, "target_id": tid, "details": details, "created_at": ca })
    }).collect();

    Ok(Json(json!({ "logs": logs })))
}

// Alias with plural name for frontend compatibility
#[get("/audit-logs")]
pub async fn audit_logs(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<serde_json::Value>>, Status> {
    user.require_permission("admin.audit")?;

    let rows = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, user_id, action, target_type, target_id, details, created_at FROM audit_log ORDER BY created_at DESC LIMIT 200"
    )
    .fetch_all(pool.inner()).await.map_err(|_| Status::InternalServerError)?;

    let logs: Vec<serde_json::Value> = rows.into_iter().map(|(id, uid, action, tt, tid, details, ca)| {
        json!({ "id": id, "user_id": uid, "action": action, "target_type": tt, "target_id": tid, "details": details, "created_at": ca })
    }).collect();

    Ok(Json(logs))
}

#[get("/settings")]
pub async fn system_settings(user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
    user.require_permission("admin.settings")?;

    Ok(Json(json!({
        "session_timeout_minutes": models::SESSION_IDLE_TIMEOUT_MINUTES,
        "password_reset_expiry_minutes": models::PASSWORD_RESET_EXPIRY_MINUTES,
        "soft_delete_hold_days": models::SOFT_DELETE_HOLD_DAYS,
        "max_submission_versions": models::MAX_SUBMISSION_VERSIONS,
        "max_file_size_mb": models::MAX_FILE_SIZE / (1024 * 1024),
        "max_review_images": models::MAX_REVIEW_IMAGES,
        "max_review_image_size_mb": models::MAX_REVIEW_IMAGE_SIZE / (1024 * 1024),
        "followup_window_days": models::FOLLOWUP_WINDOW_DAYS,
        "sla_first_response_hours": models::SLA_FIRST_RESPONSE_HOURS,
        "sla_resolution_hours": models::SLA_RESOLUTION_HOURS,
        "high_quantity_threshold": models::HIGH_QUANTITY_THRESHOLD,
        "refund_count_threshold": models::REFUND_COUNT_THRESHOLD,
        "allowed_file_types": ["pdf", "docx", "png", "jpg"],
        "allowed_submission_types": ["journal_article", "conference_paper", "thesis", "book_chapter"],
        "allowed_case_types": ["return", "refund", "exchange"],
        "allowed_payment_methods": ["cash", "check", "on_account"],
        "notification_channels": {
            "in_app": { "available": true, "description": "In-app inbox banners" },
            "email": { "available": false, "description": "Email notifications (unavailable offline)" },
            "sms": { "available": false, "description": "SMS notifications (unavailable offline)" }
        },
        "payment_gateways": "Third-party payment gateway integration is disabled unless an offline-compatible connector is installed and configured by an Administrator."
    })))
}

/// Permanently delete users past their 30-day hold
#[post("/cleanup-soft-deleted")]
pub async fn cleanup_soft_deleted(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
    user.require_permission("admin.dashboard")?;

    let result = sqlx::query("DELETE FROM users WHERE deletion_scheduled_at IS NOT NULL AND deletion_scheduled_at <= NOW()")
        .execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(json!({
        "deleted_count": result.rows_affected()
    })))
}
