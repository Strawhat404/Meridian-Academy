use crate::DbPool;
use uuid::Uuid;

/// Notification preference column names corresponding to user-controllable toggles.
pub const PREF_SUBMISSIONS: &str = "notify_submissions";
pub const PREF_ORDERS: &str = "notify_orders";
pub const PREF_REVIEWS: &str = "notify_reviews";
pub const PREF_CASES: &str = "notify_cases";

/// Insert a notification row for `user_id` if the user's `pref_column` preference is enabled.
///
/// Failures are logged and swallowed: notification delivery is best-effort and must
/// never cause the originating business event (submission/order/review/case creation)
/// to fail.
pub async fn create_notification(
    pool: &DbPool,
    user_id: &str,
    pref_column: &str,
    title: &str,
    message: &str,
) {
    // Whitelist the preference column to prevent any chance of SQL injection
    // through the inlined column name.
    let column = match pref_column {
        PREF_SUBMISSIONS | PREF_ORDERS | PREF_REVIEWS | PREF_CASES => pref_column,
        _ => {
            log::error!("create_notification: unknown preference column '{}'", pref_column);
            return;
        }
    };

    let query = format!("SELECT {} FROM users WHERE id = ?", column);
    let enabled = match sqlx::query_scalar::<_, bool>(&query)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            log::warn!("create_notification: user {} not found", user_id);
            return;
        }
        Err(e) => {
            log::error!("create_notification: read pref {} failed: {}", column, e);
            return;
        }
    };

    if !enabled {
        return;
    }

    let id = Uuid::new_v4().to_string();
    if let Err(e) = sqlx::query(
        "INSERT INTO notifications (id, user_id, title, message, is_read, created_at) VALUES (?, ?, ?, ?, false, NOW())"
    )
    .bind(&id)
    .bind(user_id)
    .bind(title)
    .bind(message)
    .execute(pool)
    .await
    {
        log::error!("create_notification: insert notification failed: {}", e);
    }
}
