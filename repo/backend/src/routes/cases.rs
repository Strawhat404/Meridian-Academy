use crate::middleware::AuthenticatedUser;
use crate::models::case::*;
use crate::models;
use crate::DbPool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

// Use a reduced query to stay within sqlx 16-field tuple limit
const CASE_SELECT: &str = "SELECT id, order_id, reporter_id, assigned_to, case_type, subject, description, status, priority, submitted_at, first_response_at, first_response_due, resolution_target, resolved_at, closed_at, created_at FROM after_sales_cases";

type CaseRow = (String, String, String, Option<String>, String, String, String, String, String, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>);

fn row_to_case(r: CaseRow) -> AfterSalesCase {
    AfterSalesCase {
        id: r.0, order_id: r.1, reporter_id: r.2, assigned_to: r.3,
        case_type: r.4, subject: r.5, description: r.6, status: r.7, priority: r.8,
        submitted_at: r.9, first_response_at: r.10, first_response_due: r.11,
        resolution_target: r.12, resolved_at: r.13, closed_at: r.14,
        created_at: r.15, updated_at: None,
    }
}

fn compute_sla(case: &AfterSalesCase) -> CaseWithSla {
    let now = chrono::Utc::now().naive_utc();

    let first_response_overdue = match (case.first_response_at, case.first_response_due) {
        (None, Some(due)) => now > due,
        _ => false,
    };

    let resolution_overdue = match (case.resolved_at, case.resolution_target) {
        (None, Some(target)) => now > target,
        _ => false,
    };

    let hours_until_first_response = match (case.first_response_at, case.first_response_due) {
        (None, Some(due)) => Some((due - now).num_minutes() as f64 / 60.0),
        _ => None,
    };

    let hours_until_resolution = match (case.resolved_at, case.resolution_target) {
        (None, Some(target)) => Some((target - now).num_minutes() as f64 / 60.0),
        _ => None,
    };

    CaseWithSla {
        case: case.clone(),
        first_response_overdue,
        resolution_overdue,
        hours_until_first_response,
        hours_until_resolution,
    }
}

#[post("/", data = "<req>")]
pub async fn create_case(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateCaseRequest>) -> Result<Json<CaseWithSla>, Status> {
    user.require_permission("cases.create")?;

    let valid_types = ["return", "refund", "exchange"];
    if !valid_types.contains(&req.case_type.as_str()) {
        return Err(Status::BadRequest);
    }

    // Verify the referenced order exists and belongs to the requesting user
    let order_owner = sqlx::query_scalar::<_, String>("SELECT user_id FROM orders WHERE id = ?")
        .bind(&req.order_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| { log::error!("create_case: select order owner failed: {}", e); Status::InternalServerError })?;

    match order_owner {
        Some(owner_id) => {
            if owner_id != user.user_id && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }

    let id = Uuid::new_v4().to_string();
    let priority = req.priority.clone().unwrap_or_else(|| "medium".to_string());
    let now = chrono::Utc::now().naive_utc();
    let first_response_due = models::add_business_days(now, 2);
    let resolution_target = models::add_business_days(now, 7);

    sqlx::query(
        "INSERT INTO after_sales_cases (id, order_id, reporter_id, case_type, subject, description, status, priority, submitted_at, first_response_due, resolution_target, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'submitted', ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(&id).bind(&req.order_id).bind(&user.user_id).bind(&req.case_type)
    .bind(&req.subject).bind(&req.description).bind(&priority)
    .bind(&now).bind(&first_response_due).bind(&resolution_target)
    .execute(pool.inner()).await.map_err(|e| { log::error!("create_case: insert case failed: {}", e); Status::InternalServerError })?;

    crate::notifications::create_notification(
        pool.inner(),
        &user.user_id,
        crate::notifications::PREF_CASES,
        "Case opened",
        &format!("Your {} case '{}' has been opened.", req.case_type, req.subject),
    ).await;

    let case = AfterSalesCase {
        id, order_id: req.order_id.clone(), reporter_id: user.user_id,
        assigned_to: None, case_type: req.case_type.clone(), subject: req.subject.clone(),
        description: req.description.clone(), status: "submitted".to_string(), priority,
        submitted_at: Some(now), first_response_at: None, first_response_due: Some(first_response_due),
        resolution_target: Some(resolution_target), resolved_at: None, closed_at: None,
        created_at: Some(now), updated_at: Some(now),
    };

    Ok(Json(compute_sla(&case)))
}

#[get("/")]
pub async fn list_cases(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<CaseWithSla>>, Status> {
    let rows = if user.is_privileged() {
        sqlx::query_as::<_, CaseRow>(
            &format!("{} ORDER BY created_at DESC", CASE_SELECT)
        )
        .fetch_all(pool.inner()).await
    } else {
        sqlx::query_as::<_, CaseRow>(
            &format!("{} WHERE reporter_id = ? ORDER BY created_at DESC", CASE_SELECT)
        )
        .bind(&user.user_id)
        .fetch_all(pool.inner()).await
    }.map_err(|e| { log::error!("list_cases: select cases query failed: {}", e); Status::InternalServerError })?;

    let cases: Vec<CaseWithSla> = rows.into_iter().map(|r| {
        let case = row_to_case(r);
        compute_sla(&case)
    }).collect();

    Ok(Json(cases))
}

#[get("/<case_id>")]
pub async fn get_case(pool: &State<DbPool>, user: AuthenticatedUser, case_id: String) -> Result<Json<CaseWithSla>, Status> {
    let query = format!("{} WHERE id = ?", CASE_SELECT);
    let row = sqlx::query_as::<_, CaseRow>(&query)
        .bind(&case_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_case: select case query failed: {}", e); Status::InternalServerError })?;

    match row {
        Some(r) => {
            let case = row_to_case(r);
            if case.reporter_id != user.user_id && !user.is_privileged() { return Err(Status::Forbidden); }
            Ok(Json(compute_sla(&case)))
        }
        None => Err(Status::NotFound),
    }
}

#[put("/<case_id>/status", data = "<req>")]
pub async fn update_case_status(pool: &State<DbPool>, user: AuthenticatedUser, case_id: String, req: Json<UpdateCaseStatusRequest>) -> Result<Status, Status> {
    user.require_privileged()?;

    let current = sqlx::query_scalar::<_, String>("SELECT status FROM after_sales_cases WHERE id = ?")
        .bind(&case_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("update_case_status: select current status failed: {}", e); Status::InternalServerError })?;

    match current {
        Some(cur) => {
            if !models::valid_case_transition(&cur, &req.status) {
                return Err(Status::BadRequest);
            }

            sqlx::query("UPDATE after_sales_cases SET status = ?, updated_at = NOW() WHERE id = ?")
                .bind(&req.status).bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("update_case_status: update status failed: {}", e); Status::InternalServerError })?;

            if cur == "submitted" && req.status == "in_review" {
                sqlx::query("UPDATE after_sales_cases SET first_response_at = COALESCE(first_response_at, NOW()) WHERE id = ?")
                    .bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("update_case_status: update first_response_at failed: {}", e); Status::InternalServerError })?;
            }
            if req.status == "approved" || req.status == "denied" {
                sqlx::query("UPDATE after_sales_cases SET resolved_at = NOW() WHERE id = ?")
                    .bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("update_case_status: update resolved_at failed: {}", e); Status::InternalServerError })?;
            }
            if req.status == "closed" {
                sqlx::query("UPDATE after_sales_cases SET closed_at = NOW() WHERE id = ?")
                    .bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("update_case_status: update closed_at failed: {}", e); Status::InternalServerError })?;
            }

            Ok(Status::Ok)
        }
        None => Err(Status::NotFound),
    }
}

#[put("/<case_id>/assign", data = "<req>")]
pub async fn assign_case(pool: &State<DbPool>, user: AuthenticatedUser, case_id: String, req: Json<AssignCaseRequest>) -> Result<Status, Status> {
    user.require_privileged()?;
    sqlx::query("UPDATE after_sales_cases SET assigned_to = ?, updated_at = NOW() WHERE id = ?")
        .bind(&req.assigned_to).bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("assign_case: update assigned_to failed: {}", e); Status::InternalServerError })?;
    Ok(Status::Ok)
}

#[post("/<case_id>/comments", data = "<req>")]
pub async fn add_comment(pool: &State<DbPool>, user: AuthenticatedUser, case_id: String, req: Json<CreateCaseCommentRequest>) -> Result<Json<CaseComment>, Status> {
    // IDOR: verify caller is the case reporter, the assigned staff, or privileged
    let case_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT reporter_id, assigned_to FROM after_sales_cases WHERE id = ?"
    ).bind(&case_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("add_comment: select case info failed: {}", e); Status::InternalServerError })?;

    match case_info {
        Some((reporter_id, assigned_to)) => {
            let is_reporter = reporter_id == user.user_id;
            let is_assigned = assigned_to.as_deref() == Some(&user.user_id);
            if !is_reporter && !is_assigned && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO case_comments (id, case_id, author_id, content, created_at) VALUES (?, ?, ?, ?, NOW())")
        .bind(&id).bind(&case_id).bind(&user.user_id).bind(&req.content)
        .execute(pool.inner()).await.map_err(|e| { log::error!("add_comment: insert comment failed: {}", e); Status::InternalServerError })?;

    if user.is_privileged() {
        sqlx::query("UPDATE after_sales_cases SET first_response_at = COALESCE(first_response_at, NOW()) WHERE id = ?")
            .bind(&case_id).execute(pool.inner()).await.map_err(|e| { log::error!("add_comment: update first_response_at failed: {}", e); Status::InternalServerError })?;
    }

    Ok(Json(CaseComment { id, case_id, author_id: user.user_id, content: req.content.clone(), created_at: None }))
}

#[get("/<case_id>/comments")]
pub async fn get_comments(pool: &State<DbPool>, user: AuthenticatedUser, case_id: String) -> Result<Json<Vec<CaseComment>>, Status> {
    // IDOR: verify caller is the reporter, the assigned handler, or privileged
    let case_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT reporter_id, assigned_to FROM after_sales_cases WHERE id = ?"
    ).bind(&case_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_comments: select case info failed: {}", e); Status::InternalServerError })?;
    match case_info {
        Some((rid, assigned_to)) => {
            let is_reporter = rid == user.user_id;
            let is_assigned = assigned_to.as_deref() == Some(&user.user_id);
            if !is_reporter && !is_assigned && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<chrono::NaiveDateTime>)>(
        "SELECT id, case_id, author_id, content, created_at FROM case_comments WHERE case_id = ? ORDER BY created_at ASC"
    )
    .bind(&case_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("get_comments: select comments query failed: {}", e); Status::InternalServerError })?;

    let comments: Vec<CaseComment> = rows.into_iter().map(|(id, cid, aid, content, ca)| {
        CaseComment { id, case_id: cid, author_id: aid, content, created_at: ca }
    }).collect();

    Ok(Json(comments))
}

#[get("/my")]
pub async fn my_cases(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<CaseWithSla>>, Status> {
    let query = format!("{} WHERE reporter_id = ? ORDER BY created_at DESC", CASE_SELECT);
    let rows = sqlx::query_as::<_, CaseRow>(&query)
        .bind(&user.user_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("my_cases: select cases query failed: {}", e); Status::InternalServerError })?;

    let cases: Vec<CaseWithSla> = rows.into_iter().map(|r| {
        let case = row_to_case(r);
        compute_sla(&case)
    }).collect();

    Ok(Json(cases))
}
