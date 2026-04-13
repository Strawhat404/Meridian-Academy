use crate::middleware::AuthenticatedUser;
use crate::models::review::*;
use crate::models;
use crate::DbPool;
use chrono::Duration;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

#[post("/", data = "<req>")]
pub async fn create_review(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateReviewRequest>) -> Result<Json<Review>, Status> {
    user.require_permission("reviews.create")?;

    if req.rating < 1 || req.rating > 5 {
        return Err(Status::BadRequest);
    }
    if req.title.len() > 120 {
        return Err(Status::UnprocessableEntity);
    }

    let order_check = sqlx::query_as::<_, (String, String)>(
        "SELECT user_id, status FROM orders WHERE id = ?"
    )
    .bind(&req.order_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| { log::error!("create_review: select order query failed: {}", e); Status::InternalServerError })?;

    match order_check {
        Some((uid, status)) => {
            if uid != user.user_id { return Err(Status::Forbidden); }
            if status != "delivered" { return Err(Status::BadRequest); }
        }
        None => return Err(Status::NotFound),
    }

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reviews WHERE order_id = ? AND user_id = ? AND is_followup = false"
    )
    .bind(&req.order_id).bind(&user.user_id)
    .fetch_one(pool.inner()).await.unwrap_or(0);

    if existing > 0 {
        return Err(Status::Conflict);
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO reviews (id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, false, NULL, NOW(), NOW())"
    )
    .bind(&id).bind(&req.order_id).bind(&req.line_item_id).bind(&user.user_id)
    .bind(req.rating).bind(&req.title).bind(&req.body)
    .execute(pool.inner()).await.map_err(|e| { log::error!("create_review: insert review failed: {}", e); Status::InternalServerError })?;

    crate::notifications::create_notification(
        pool.inner(),
        &user.user_id,
        crate::notifications::PREF_REVIEWS,
        "Review posted",
        &format!("Your review '{}' has been posted.", req.title),
    ).await;

    Ok(Json(Review {
        id, order_id: req.order_id.clone(), line_item_id: req.line_item_id.clone(),
        user_id: user.user_id, rating: req.rating, title: req.title.clone(),
        body: req.body.clone(), is_followup: false, parent_review_id: None,
        created_at: None, updated_at: None,
    }))
}

#[post("/followup", data = "<req>")]
pub async fn create_followup(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateFollowupRequest>) -> Result<Json<Review>, Status> {
    if req.rating < 1 || req.rating > 5 {
        return Err(Status::BadRequest);
    }

    let parent = sqlx::query_as::<_, (String, String, String, bool, Option<chrono::NaiveDateTime>)>(
        "SELECT id, order_id, user_id, is_followup, created_at FROM reviews WHERE id = ?"
    )
    .bind(&req.parent_review_id)
    .fetch_optional(pool.inner()).await.map_err(|e| { log::error!("create_followup: select parent review failed: {}", e); Status::InternalServerError })?;

    match parent {
        Some((pid, order_id, parent_uid, is_fu, created_at)) => {
            if parent_uid != user.user_id { return Err(Status::Forbidden); }
            if is_fu { return Err(Status::BadRequest); }

            if let Some(ca) = created_at {
                let deadline = ca + Duration::days(models::FOLLOWUP_WINDOW_DAYS);
                if chrono::Utc::now().naive_utc() > deadline {
                    return Err(Status::Gone);
                }
            }

            let existing_fu = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM reviews WHERE parent_review_id = ? AND is_followup = true"
            )
            .bind(&pid).fetch_one(pool.inner()).await.unwrap_or(0);

            if existing_fu > 0 {
                return Err(Status::Conflict);
            }

            let id = Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO reviews (id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at) VALUES (?, ?, NULL, ?, ?, ?, ?, true, ?, NOW(), NOW())"
            )
            .bind(&id).bind(&order_id).bind(&user.user_id)
            .bind(req.rating).bind(&req.title).bind(&req.body).bind(&pid)
            .execute(pool.inner()).await.map_err(|e| { log::error!("create_followup: insert followup review failed: {}", e); Status::InternalServerError })?;

            crate::notifications::create_notification(
                pool.inner(),
                &user.user_id,
                crate::notifications::PREF_REVIEWS,
                "Follow-up review posted",
                &format!("Your follow-up review '{}' has been posted.", req.title),
            ).await;

            Ok(Json(Review {
                id, order_id, line_item_id: None, user_id: user.user_id,
                rating: req.rating, title: req.title.clone(), body: req.body.clone(),
                is_followup: true, parent_review_id: Some(pid),
                created_at: None, updated_at: None,
            }))
        }
        None => Err(Status::NotFound),
    }
}

/// List reviews — IDOR enforced: users see only their own; staff/admin see all.
#[get("/")]
pub async fn list_reviews(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Review>>, Status> {
    let rows = if user.is_privileged() {
        sqlx::query_as::<_, (String, String, Option<String>, String, i32, String, String, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            "SELECT id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at FROM reviews ORDER BY created_at DESC LIMIT 200"
        )
        .fetch_all(pool.inner()).await
    } else {
        sqlx::query_as::<_, (String, String, Option<String>, String, i32, String, String, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            "SELECT id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at FROM reviews WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(&user.user_id)
        .fetch_all(pool.inner()).await
    }.map_err(|e| { log::error!("list_reviews: select reviews query failed: {}", e); Status::InternalServerError })?;

    let reviews: Vec<Review> = rows.into_iter().map(|(id, oid, lid, uid, rating, title, body, fu, prid, ca, ua)| {
        Review { id, order_id: oid, line_item_id: lid, user_id: uid, rating, title, body, is_followup: fu, parent_review_id: prid, created_at: ca, updated_at: ua }
    }).collect();

    Ok(Json(reviews))
}

/// Get single review — IDOR enforced: owner, or order owner, or staff/admin.
#[get("/<review_id>")]
pub async fn get_review(pool: &State<DbPool>, user: AuthenticatedUser, review_id: String) -> Result<Json<Review>, Status> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, String, i32, String, String, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at FROM reviews WHERE id = ?"
    )
    .bind(&review_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_review: select review query failed: {}", e); Status::InternalServerError })?;

    match row {
        Some((id, oid, lid, uid, rating, title, body, fu, prid, ca, ua)) => {
            // IDOR: must be the review author, the order owner, or privileged
            let is_privileged = user.is_privileged();
            let is_review_owner = uid == user.user_id;
            if !is_review_owner && !is_privileged {
                // Check if requester owns the order
                let order_owner = sqlx::query_scalar::<_, String>("SELECT user_id FROM orders WHERE id = ?")
                    .bind(&oid).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_review: select order owner failed: {}", e); Status::InternalServerError })?;
                if order_owner.as_deref() != Some(&user.user_id) {
                    return Err(Status::Forbidden);
                }
            }

            Ok(Json(Review { id, order_id: oid, line_item_id: lid, user_id: uid, rating, title, body, is_followup: fu, parent_review_id: prid, created_at: ca, updated_at: ua }))
        }
        None => Err(Status::NotFound),
    }
}

#[post("/<review_id>/images", data = "<req>")]
pub async fn add_review_image(pool: &State<DbPool>, user: AuthenticatedUser, review_id: String, req: Json<AddReviewImageRequest>) -> Result<Status, Status> {
    let owner = sqlx::query_scalar::<_, String>("SELECT user_id FROM reviews WHERE id = ?")
        .bind(&review_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("add_review_image: select review owner failed: {}", e); Status::InternalServerError })?;

    match owner {
        Some(uid) if uid == user.user_id => {}
        Some(_) => return Err(Status::Forbidden),
        None => return Err(Status::NotFound),
    }

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM review_images WHERE review_id = ?")
        .bind(&review_id).fetch_one(pool.inner()).await.unwrap_or(0);

    if count >= models::MAX_REVIEW_IMAGES as i64 {
        return Err(Status::UnprocessableEntity);
    }

    let file_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &req.file_data)
        .map_err(|e| { log::warn!("add_review_image: base64 decode failed: {}", e); Status::BadRequest })?;

    if file_data.len() as u64 > models::MAX_REVIEW_IMAGE_SIZE {
        return Err(Status::PayloadTooLarge);
    }

    // Validate file type and magic bytes — only PNG and JPG allowed for review images
    let magic = if file_data.len() >= 8 { &file_data[..8] } else { &file_data };
    let ext = req.file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    let valid_image = match ext.as_str() {
        "png" => magic.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        "jpg" | "jpeg" => magic.starts_with(&[0xFF, 0xD8, 0xFF]),
        _ => false,
    };
    if !valid_image {
        return Err(Status::UnsupportedMediaType);
    }

    // Compute SHA-256 hash for integrity
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let file_hash = hex::encode(hasher.finalize());

    let id = Uuid::new_v4().to_string();
    let file_path = format!("uploads/reviews/{}/{}", review_id, req.file_name);

    sqlx::query(
        "INSERT INTO review_images (id, review_id, file_name, file_path, file_size, file_type, file_hash, image_data, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())"
    )
    .bind(&id).bind(&review_id).bind(&req.file_name).bind(&file_path)
    .bind(file_data.len() as i64).bind(&ext).bind(&file_hash).bind(&file_data)
    .execute(pool.inner()).await.map_err(|e| { log::error!("add_review_image: insert review_images failed: {}", e); Status::InternalServerError })?;

    Ok(Status::Created)
}

#[get("/my")]
pub async fn my_reviews(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Review>>, Status> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String, i32, String, String, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, order_id, line_item_id, user_id, rating, title, body, is_followup, parent_review_id, created_at, updated_at FROM reviews WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user.user_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("my_reviews: select reviews query failed: {}", e); Status::InternalServerError })?;

    let reviews: Vec<Review> = rows.into_iter().map(|(id, oid, lid, uid, rating, title, body, fu, prid, ca, ua)| {
        Review { id, order_id: oid, line_item_id: lid, user_id: uid, rating, title, body, is_followup: fu, parent_review_id: prid, created_at: ca, updated_at: ua }
    }).collect();

    Ok(Json(reviews))
}
