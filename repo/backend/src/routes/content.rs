use crate::middleware::AuthenticatedUser;
use crate::models::content::*;
use crate::models;
use crate::DbPool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

// ===== SENSITIVE WORDS MANAGEMENT =====

#[get("/sensitive-words")]
pub async fn list_sensitive_words(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<SensitiveWord>>, Status> {
    user.require_permission("content.manage")?;

    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "SELECT id, word, action, replacement, added_by FROM sensitive_words ORDER BY word"
    )
    .fetch_all(pool.inner()).await.map_err(|e| { log::error!("list_sensitive_words: select words query failed: {}", e); Status::InternalServerError })?;

    let words: Vec<SensitiveWord> = rows.into_iter().map(|(id, word, action, replacement, added_by)| {
        SensitiveWord { id, word, action, replacement, added_by }
    }).collect();

    Ok(Json(words))
}

#[post("/sensitive-words", data = "<req>")]
pub async fn add_sensitive_word(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<AddSensitiveWordRequest>) -> Result<Json<SensitiveWord>, Status> {
    user.require_permission("content.manage")?;

    let valid_actions = ["replace", "block"];
    if !valid_actions.contains(&req.action.as_str()) {
        return Err(Status::BadRequest);
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO sensitive_words (id, word, action, replacement, added_by, created_at) VALUES (?, ?, ?, ?, ?, NOW())"
    )
    .bind(&id).bind(&req.word).bind(&req.action).bind(&req.replacement).bind(&user.user_id)
    .execute(pool.inner()).await.map_err(|e| { log::error!("add_sensitive_word: insert word failed: {}", e); Status::InternalServerError })?;

    let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'sensitive_word_added', 'sensitive_word', ?, ?, NOW())")
        .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&id)
        .bind(&format!("Word '{}' added with action '{}'", req.word, req.action))
        .execute(pool.inner()).await;

    Ok(Json(SensitiveWord {
        id, word: req.word.clone(), action: req.action.clone(),
        replacement: req.replacement.clone(), added_by: user.user_id,
    }))
}

#[delete("/sensitive-words/<word_id>")]
pub async fn remove_sensitive_word(pool: &State<DbPool>, user: AuthenticatedUser, word_id: String) -> Result<Status, Status> {
    user.require_permission("content.manage")?;

    sqlx::query("DELETE FROM sensitive_words WHERE id = ?")
        .bind(&word_id).execute(pool.inner()).await.map_err(|e| { log::error!("remove_sensitive_word: delete word failed: {}", e); Status::InternalServerError })?;

    let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'sensitive_word_removed', 'sensitive_word', ?, 'Sensitive word removed', NOW())")
        .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&word_id)
        .execute(pool.inner()).await;

    Ok(Status::NoContent)
}

#[post("/check", data = "<req>")]
pub async fn check_content(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<serde_json::Value>) -> Result<Json<ContentCheckResult>, Status> {
    user.require_privileged()?;

    let text = req.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let words = load_sensitive_words(pool.inner()).await;
    let result = check_sensitive_words(text, &words);
    Ok(Json(result))
}

// ===== CONTENT ITEM LIFECYCLE (submissions as content items) =====
// These endpoints manage the content governance lifecycle:
// submit → review → approve/reject → publish, with versioning and rollback.

/// Submit a draft submission for review (changes status draft → submitted)
#[post("/items/<item_id>/submit")]
pub async fn submit_item(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String) -> Result<Status, Status> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT author_id, status FROM submissions WHERE id = ?"
    ).bind(&item_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("submit_item: select submission failed: {}", e); Status::InternalServerError })?;

    match row {
        Some((author_id, status)) => {
            if author_id != user.user_id { return Err(Status::Forbidden); }
            if status != "draft" && status != "revision_requested" {
                return Err(Status::Conflict);
            }
            sqlx::query("UPDATE submissions SET status = 'submitted', updated_at = NOW() WHERE id = ?")
                .bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("submit_item: update status to submitted failed: {}", e); Status::InternalServerError })?;
            Ok(Status::Ok)
        }
        None => Err(Status::NotFound),
    }
}

/// Staff/admin approves content (submitted/in_review → accepted)
#[post("/items/<item_id>/approve")]
pub async fn approve_item(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String) -> Result<Status, Status> {
    user.require_permission("submissions.review")?;

    let status = sqlx::query_scalar::<_, String>("SELECT status FROM submissions WHERE id = ?")
        .bind(&item_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("approve_item: select status failed: {}", e); Status::InternalServerError })?;

    match status {
        Some(s) if s == "submitted" || s == "in_review" || s == "blocked" => {
            sqlx::query("UPDATE submissions SET status = 'accepted', updated_at = NOW() WHERE id = ?")
                .bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("approve_item: update status to accepted failed: {}", e); Status::InternalServerError })?;

            let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'content_approved', 'submission', ?, 'Content item approved', NOW())")
                .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&item_id)
                .execute(pool.inner()).await;
            Ok(Status::Ok)
        }
        Some(_) => Err(Status::Conflict),
        None => Err(Status::NotFound),
    }
}

/// Staff/admin rejects content (submitted/in_review → rejected)
#[post("/items/<item_id>/reject")]
pub async fn reject_item(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String) -> Result<Status, Status> {
    user.require_permission("submissions.review")?;

    let status = sqlx::query_scalar::<_, String>("SELECT status FROM submissions WHERE id = ?")
        .bind(&item_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("reject_item: select status failed: {}", e); Status::InternalServerError })?;

    match status {
        Some(s) if s == "submitted" || s == "in_review" => {
            sqlx::query("UPDATE submissions SET status = 'rejected', updated_at = NOW() WHERE id = ?")
                .bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("reject_item: update status to rejected failed: {}", e); Status::InternalServerError })?;

            let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'content_rejected', 'submission', ?, 'Content item rejected', NOW())")
                .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&item_id)
                .execute(pool.inner()).await;
            Ok(Status::Ok)
        }
        Some(_) => Err(Status::Conflict),
        None => Err(Status::NotFound),
    }
}

/// Staff/admin requests revision (in_review → revision_requested)
#[post("/items/<item_id>/request-revision")]
pub async fn request_revision(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String) -> Result<Status, Status> {
    user.require_permission("submissions.review")?;

    sqlx::query("UPDATE submissions SET status = 'revision_requested', updated_at = NOW() WHERE id = ? AND status IN ('submitted', 'in_review')")
        .bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("request_revision: update status to revision_requested failed: {}", e); Status::InternalServerError })?;
    Ok(Status::Ok)
}

/// Staff/admin publishes accepted content (accepted → published)
#[post("/items/<item_id>/publish")]
pub async fn publish_item(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String) -> Result<Status, Status> {
    user.require_permission("submissions.review")?;

    sqlx::query("UPDATE submissions SET status = 'published', updated_at = NOW() WHERE id = ? AND status = 'accepted'")
        .bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("publish_item: update status to published failed: {}", e); Status::InternalServerError })?;
    Ok(Status::Ok)
}

/// Rollback to a previous version (resets current_version)
#[post("/items/<item_id>/rollback/<version_number>")]
pub async fn rollback_version(pool: &State<DbPool>, user: AuthenticatedUser, item_id: String, version_number: i32) -> Result<Status, Status> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT author_id, status FROM submissions WHERE id = ?"
    ).bind(&item_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("rollback_version: select submission failed: {}", e); Status::InternalServerError })?;

    match row {
        Some((author_id, _status)) => {
            user.require_owner_or_privileged(&author_id)?;

            // Verify the target version exists
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM submission_versions WHERE submission_id = ? AND version_number = ?"
            ).bind(&item_id).bind(version_number).fetch_one(pool.inner()).await.unwrap_or(0);

            if exists == 0 { return Err(Status::NotFound); }

            sqlx::query("UPDATE submissions SET current_version = ?, status = 'draft', updated_at = NOW() WHERE id = ?")
                .bind(version_number).bind(&item_id).execute(pool.inner()).await.map_err(|e| { log::error!("rollback_version: update current_version failed: {}", e); Status::InternalServerError })?;

            let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'content_rollback', 'submission', ?, ?, NOW())")
                .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&item_id)
                .bind(&format!("Rolled back to version {}", version_number))
                .execute(pool.inner()).await;

            Ok(Status::Ok)
        }
        None => Err(Status::NotFound),
    }
}

async fn load_sensitive_words(pool: &DbPool) -> Vec<SensitiveWord> {
    sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "SELECT id, word, action, replacement, added_by FROM sensitive_words"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, word, action, replacement, added_by)| SensitiveWord { id, word, action, replacement, added_by })
    .collect()
}
