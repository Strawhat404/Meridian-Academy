use crate::middleware::AuthenticatedUser;
use crate::models::order::*;
use crate::models;
use crate::DbPool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

fn generate_order_number() -> String {
    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let rand_part: u32 = rand::random::<u32>() % 10000;
    format!("ORD-{}-{:04}", ts, rand_part)
}

use rand;

#[post("/", data = "<req>")]
pub async fn create_order(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateOrderRequest>) -> Result<Json<OrderWithItems>, Status> {
    user.require_permission("orders.create")?;

    let valid_periods = ["monthly", "quarterly", "annual"];
    if !valid_periods.contains(&req.subscription_period.as_str()) {
        return Err(Status::BadRequest);
    }
    if req.line_items.is_empty() {
        return Err(Status::BadRequest);
    }

    let order_id = Uuid::new_v4().to_string();
    let order_number = generate_order_number();

    // Validate shipping_address_id ownership if provided
    if let Some(ref addr_id) = req.shipping_address_id {
        let addr_owner = sqlx::query_scalar::<_, String>(
            "SELECT user_id FROM user_addresses WHERE id = ?"
        )
        .bind(addr_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| { log::error!("create_order: shipping address query failed: {}", e); Status::InternalServerError })?;

        match addr_owner {
            Some(owner_id) => {
                if owner_id != user.user_id {
                    return Err(Status::Forbidden);
                }
            }
            None => return Err(Status::NotFound),
        }
    }

    let mut total_amount: f64 = 0.0;
    let mut has_high_qty = false;
    let mut line_items_out = Vec::new();

    // Calculate totals first without inserting
    for item in &req.line_items {
        let line_total = item.quantity as f64 * item.unit_price;
        total_amount += line_total;
        if item.quantity > models::HIGH_QUANTITY_THRESHOLD {
            has_high_qty = true;
        }
    }

    // Check for repeated refunds
    let refund_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM payments WHERE order_id IN (SELECT id FROM orders WHERE user_id = ?) AND transaction_type = 'refund'"
    )
    .bind(&user.user_id)
    .fetch_one(pool.inner())
    .await
    .unwrap_or(0);

    let is_flagged = has_high_qty || refund_count >= models::REFUND_COUNT_THRESHOLD;
    let flag_reason = if has_high_qty && refund_count >= models::REFUND_COUNT_THRESHOLD {
        Some("High quantity order AND repeated refund history".to_string())
    } else if has_high_qty {
        Some("Unusually high quantity order".to_string())
    } else if refund_count >= models::REFUND_COUNT_THRESHOLD {
        Some("Account with repeated refund requests".to_string())
    } else {
        None
    };

    // Insert order FIRST so FK on line items is satisfied
    sqlx::query(
        "INSERT INTO orders (id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, total_amount, is_flagged, flag_reason, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', 'unpaid', ?, ?, ?, NOW(), NOW())"
    )
    .bind(&order_id).bind(&user.user_id).bind(&order_number).bind(&req.subscription_period)
    .bind(&req.shipping_address_id).bind(total_amount).bind(is_flagged).bind(&flag_reason)
    .execute(pool.inner())
    .await
    .map_err(|e| { log::error!("create_order: order insert failed: {}", e); Status::InternalServerError })?;

    // Now insert line items
    for item in &req.line_items {
        let line_total = item.quantity as f64 * item.unit_price;
        let li_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO order_line_items (id, order_id, publication_title, series_name, quantity, unit_price, line_total) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&li_id).bind(&order_id).bind(&item.publication_title).bind(&item.series_name)
        .bind(item.quantity).bind(item.unit_price).bind(line_total)
        .execute(pool.inner())
        .await
        .map_err(|e| { log::error!("create_order: line item insert failed: {}", e); Status::InternalServerError })?;

        line_items_out.push(OrderLineItem {
            id: li_id, order_id: order_id.clone(), publication_title: item.publication_title.clone(),
            series_name: item.series_name.clone(), quantity: item.quantity, unit_price: item.unit_price, line_total,
        });
    }

    if is_flagged {
        let _ = sqlx::query(
            "INSERT INTO abnormal_order_flags (id, order_id, user_id, flag_type, reason, created_at) VALUES (?, ?, ?, ?, ?, NOW())"
        )
        .bind(Uuid::new_v4().to_string()).bind(&order_id).bind(&user.user_id)
        .bind(if has_high_qty { "high_quantity" } else { "repeated_refunds" })
        .bind(flag_reason.as_deref().unwrap_or(""))
        .execute(pool.inner()).await;
    }

    // Generate initial reconciliation records for the new order
    generate_initial_reconciliation(pool.inner(), &order_id).await;

    let order = Order {
        id: order_id, user_id: user.user_id, order_number, subscription_period: req.subscription_period.clone(),
        shipping_address_id: req.shipping_address_id.clone(), status: "pending".to_string(),
        payment_status: "unpaid".to_string(), total_amount, parent_order_id: None,
        is_flagged, flag_reason, created_at: None, updated_at: None,
    };

    Ok(Json(OrderWithItems { order, line_items: line_items_out }))
}

#[get("/")]
pub async fn list_orders(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Order>>, Status> {
    let base = "SELECT id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, CAST(total_amount AS DOUBLE), parent_order_id, is_flagged, flag_reason, created_at, updated_at FROM orders";

    let rows = if user.is_privileged() {
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, f64, Option<String>, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            &format!("{} ORDER BY created_at DESC", base)
        )
        .fetch_all(pool.inner()).await
    } else {
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, f64, Option<String>, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            &format!("{} WHERE user_id = ? ORDER BY created_at DESC", base)
        )
        .bind(&user.user_id)
        .fetch_all(pool.inner()).await
    }.map_err(|e| { log::error!("list_orders: select orders query failed: {}", e); Status::InternalServerError })?;

    let orders: Vec<Order> = rows.into_iter().map(|(id, user_id, order_number, sp, said, status, ps, total, parent, flagged, fr, ca, ua)| {
        Order { id, user_id, order_number, subscription_period: sp, shipping_address_id: said, status, payment_status: ps, total_amount: total, parent_order_id: parent, is_flagged: flagged, flag_reason: fr, created_at: ca, updated_at: ua }
    }).collect();

    Ok(Json(orders))
}

#[get("/<order_id>")]
pub async fn get_order(pool: &State<DbPool>, user: AuthenticatedUser, order_id: String) -> Result<Json<OrderWithItems>, Status> {
    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, f64, Option<String>, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, CAST(total_amount AS DOUBLE), parent_order_id, is_flagged, flag_reason, created_at, updated_at FROM orders WHERE id = ?"
    )
    .bind(&order_id)
    .fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_order: select order query failed: {}", e); Status::InternalServerError })?;

    match row {
        Some((id, uid, on, sp, said, status, ps, total, parent, flagged, fr, ca, ua)) => {
            if uid != user.user_id && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
            let items = sqlx::query_as::<_, (String, String, String, Option<String>, i32, f64, f64)>(
                "SELECT id, order_id, publication_title, series_name, quantity, CAST(unit_price AS DOUBLE), CAST(line_total AS DOUBLE) FROM order_line_items WHERE order_id = ?"
            )
            .bind(&id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("get_order: select line items query failed: {}", e); Status::InternalServerError })?;

            let line_items: Vec<OrderLineItem> = items.into_iter().map(|(lid, oid, pt, sn, qty, up, lt)| {
                OrderLineItem { id: lid, order_id: oid, publication_title: pt, series_name: sn, quantity: qty, unit_price: up, line_total: lt }
            }).collect();

            let order = Order { id, user_id: uid, order_number: on, subscription_period: sp, shipping_address_id: said, status, payment_status: ps, total_amount: total, parent_order_id: parent, is_flagged: flagged, flag_reason: fr, created_at: ca, updated_at: ua };
            Ok(Json(OrderWithItems { order, line_items }))
        }
        None => Err(Status::NotFound),
    }
}

#[put("/<order_id>/status", data = "<req>", rank = 2)]
pub async fn update_order_status(pool: &State<DbPool>, user: AuthenticatedUser, order_id: String, req: Json<UpdateOrderStatusRequest>) -> Result<Status, Status> {
    user.require_privileged()?;
    let valid = ["pending", "confirmed", "processing", "shipped", "delivered", "cancelled"];
    if !valid.contains(&req.status.as_str()) {
        return Err(Status::BadRequest);
    }

    sqlx::query("UPDATE orders SET status = ?, updated_at = NOW() WHERE id = ?")
        .bind(&req.status).bind(&order_id).execute(pool.inner()).await.map_err(|e| { log::error!("update_order_status: update order status failed: {}", e); Status::InternalServerError })?;

    Ok(Status::Ok)
}

#[get("/my")]
pub async fn my_orders(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Order>>, Status> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, f64, Option<String>, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, CAST(total_amount AS DOUBLE), parent_order_id, is_flagged, flag_reason, created_at, updated_at FROM orders WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user.user_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("my_orders: select orders query failed: {}", e); Status::InternalServerError })?;

    let orders: Vec<Order> = rows.into_iter().map(|(id, user_id, on, sp, said, status, ps, total, parent, flagged, fr, ca, ua)| {
        Order { id, user_id, order_number: on, subscription_period: sp, shipping_address_id: said, status, payment_status: ps, total_amount: total, parent_order_id: parent, is_flagged: flagged, flag_reason: fr, created_at: ca, updated_at: ua }
    }).collect();

    Ok(Json(orders))
}

/// Split order by series: creates new orders per unique series
#[post("/split", data = "<req>")]
pub async fn split_order(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<SplitOrderRequest>) -> Result<Json<Vec<Order>>, Status> {
    user.require_permission("orders.manage")?;

    // Get original order
    let orig = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, user_id, subscription_period, shipping_address_id FROM orders WHERE id = ?"
    )
    .bind(&req.order_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("split_order: select original order failed: {}", e); Status::InternalServerError })?;

    let (orig_id, orig_user, orig_period, orig_addr) = match orig {
        Some(o) => o,
        None => return Err(Status::NotFound),
    };

    // Get line items grouped by series
    let items = sqlx::query_as::<_, (String, String, Option<String>, i32, f64, f64)>(
        "SELECT id, publication_title, series_name, quantity, CAST(unit_price AS DOUBLE), CAST(line_total AS DOUBLE) FROM order_line_items WHERE order_id = ?"
    )
    .bind(&orig_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("split_order: select line items failed: {}", e); Status::InternalServerError })?;

    let mut series_map: std::collections::HashMap<String, Vec<(String, String, Option<String>, i32, f64, f64)>> = std::collections::HashMap::new();
    for item in items {
        let series = item.2.clone().unwrap_or_else(|| "default".to_string());
        series_map.entry(series).or_default().push(item);
    }

    if series_map.len() <= 1 {
        return Err(Status::BadRequest); // Nothing to split
    }

    let mut new_orders = Vec::new();

    for (_series, items) in &series_map {
        let new_id = Uuid::new_v4().to_string();
        let new_number = generate_order_number();
        let total: f64 = items.iter().map(|i| i.5).sum();

        sqlx::query(
            "INSERT INTO orders (id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, total_amount, parent_order_id, is_flagged, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', 'unpaid', ?, ?, false, NOW(), NOW())"
        )
        .bind(&new_id).bind(&orig_user).bind(&new_number).bind(&orig_period).bind(&orig_addr)
        .bind(total).bind(&orig_id)
        .execute(pool.inner()).await.map_err(|e| { log::error!("split_order: insert split order failed: {}", e); Status::InternalServerError })?;

        for item in items {
            let li_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO order_line_items (id, order_id, publication_title, series_name, quantity, unit_price, line_total) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&li_id).bind(&new_id).bind(&item.1).bind(&item.2).bind(item.3).bind(item.4).bind(item.5)
            .execute(pool.inner()).await.map_err(|e| { log::error!("split_order: insert line item failed: {}", e); Status::InternalServerError })?;
        }

        new_orders.push(Order {
            id: new_id, user_id: orig_user.clone(), order_number: new_number,
            subscription_period: orig_period.clone(), shipping_address_id: orig_addr.clone(),
            status: "pending".to_string(), payment_status: "unpaid".to_string(),
            total_amount: total, parent_order_id: Some(orig_id.clone()),
            is_flagged: false, flag_reason: None, created_at: None, updated_at: None,
        });
    }

    // Mark original as split
    sqlx::query("UPDATE orders SET status = 'split', updated_at = NOW() WHERE id = ?")
        .bind(&orig_id).execute(pool.inner()).await.map_err(|e| { log::error!("split_order: update original order status failed: {}", e); Status::InternalServerError })?;

    Ok(Json(new_orders))
}

/// Merge multiple orders from same user into one
#[post("/merge", data = "<req>")]
pub async fn merge_orders(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<MergeOrdersRequest>) -> Result<Json<OrderWithItems>, Status> {
    user.require_permission("orders.manage")?;
    if req.order_ids.len() < 2 {
        return Err(Status::BadRequest);
    }

    // Verify all orders belong to same user
    let mut owner_id: Option<String> = None;
    let mut period: Option<String> = None;
    let mut addr: Option<String> = None;

    for oid in &req.order_ids {
        let row = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT user_id, subscription_period, shipping_address_id FROM orders WHERE id = ?"
        )
        .bind(oid).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("merge_orders: select order owner failed: {}", e); Status::InternalServerError })?;

        match row {
            Some((uid, sp, sa)) => {
                if let Some(ref existing) = owner_id {
                    if existing != &uid { return Err(Status::BadRequest); }
                } else {
                    owner_id = Some(uid);
                    period = Some(sp);
                    addr = sa;
                }
            }
            None => return Err(Status::NotFound),
        }
    }

    let merged_id = Uuid::new_v4().to_string();
    let merged_number = generate_order_number();
    let mut all_items = Vec::new();
    let mut total = 0.0_f64;

    for oid in &req.order_ids {
        let items = sqlx::query_as::<_, (String, Option<String>, i32, f64, f64)>(
            "SELECT publication_title, series_name, quantity, CAST(unit_price AS DOUBLE), CAST(line_total AS DOUBLE) FROM order_line_items WHERE order_id = ?"
        )
        .bind(oid).fetch_all(pool.inner()).await.map_err(|e| { log::error!("merge_orders: select line items failed: {}", e); Status::InternalServerError })?;

        for (pt, sn, qty, up, lt) in items {
            total += lt;
            let li_id = Uuid::new_v4().to_string();
            all_items.push(OrderLineItem {
                id: li_id.clone(), order_id: merged_id.clone(), publication_title: pt.clone(),
                series_name: sn.clone(), quantity: qty, unit_price: up, line_total: lt,
            });
        }
    }

    sqlx::query(
        "INSERT INTO orders (id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, total_amount, is_flagged, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', 'unpaid', ?, false, NOW(), NOW())"
    )
    .bind(&merged_id).bind(owner_id.as_ref().unwrap()).bind(&merged_number).bind(period.as_ref().unwrap())
    .bind(&addr).bind(total)
    .execute(pool.inner()).await.map_err(|e| { log::error!("merge_orders: insert merged order failed: {}", e); Status::InternalServerError })?;

    for item in &all_items {
        sqlx::query(
            "INSERT INTO order_line_items (id, order_id, publication_title, series_name, quantity, unit_price, line_total) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&item.id).bind(&merged_id).bind(&item.publication_title).bind(&item.series_name)
        .bind(item.quantity).bind(item.unit_price).bind(item.line_total)
        .execute(pool.inner()).await.map_err(|e| { log::error!("merge_orders: insert merged line item failed: {}", e); Status::InternalServerError })?;
    }

    // Mark originals as merged
    for oid in &req.order_ids {
        sqlx::query("UPDATE orders SET status = 'merged', parent_order_id = ?, updated_at = NOW() WHERE id = ?")
            .bind(&merged_id).bind(oid).execute(pool.inner()).await.map_err(|e| { log::error!("merge_orders: update original order status failed: {}", e); Status::InternalServerError })?;
    }

    let order = Order {
        id: merged_id, user_id: owner_id.unwrap(), order_number: merged_number,
        subscription_period: period.unwrap(), shipping_address_id: addr,
        status: "pending".to_string(), payment_status: "unpaid".to_string(),
        total_amount: total, parent_order_id: None, is_flagged: false, flag_reason: None,
        created_at: None, updated_at: None,
    };

    Ok(Json(OrderWithItems { order, line_items: all_items }))
}

#[post("/fulfillment", data = "<req>")]
pub async fn log_fulfillment_event(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateFulfillmentEventRequest>) -> Result<Json<FulfillmentEvent>, Status> {
    user.require_permission("orders.fulfillment")?;

    let valid_types = ["missing_issue", "reshipment", "delay", "discontinuation", "edition_change", "delivered"];
    if !valid_types.contains(&req.event_type.as_str()) {
        return Err(Status::BadRequest);
    }

    // Reason cannot be blank
    if req.reason.trim().is_empty() {
        return Err(Status::UnprocessableEntity);
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO fulfillment_events (id, order_id, line_item_id, event_type, issue_identifier, reason, expected_date, actual_date, logged_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())"
    )
    .bind(&id).bind(&req.order_id).bind(&req.line_item_id).bind(&req.event_type)
    .bind(&req.issue_identifier).bind(&req.reason)
    .bind(&req.expected_date).bind(&req.actual_date).bind(&user.user_id)
    .execute(pool.inner()).await.map_err(|e| { log::error!("log_fulfillment_event: insert fulfillment event failed: {}", e); Status::InternalServerError })?;

    // Generate/update reconciliation record for this fulfillment event
    if let Some(ref issue_id) = req.issue_identifier {
        generate_reconciliation_record(
            pool.inner(),
            &req.order_id,
            req.line_item_id.as_deref(),
            issue_id,
            &req.event_type,
        ).await;
    }

    Ok(Json(FulfillmentEvent {
        id, order_id: req.order_id.clone(), line_item_id: req.line_item_id.clone(),
        event_type: req.event_type.clone(), issue_identifier: req.issue_identifier.clone(),
        reason: req.reason.clone(), expected_date: req.expected_date.clone(),
        actual_date: req.actual_date.clone(), logged_by: user.user_id, created_at: None,
    }))
}

#[get("/<order_id>/fulfillment")]
pub async fn list_fulfillment_events(pool: &State<DbPool>, user: AuthenticatedUser, order_id: String) -> Result<Json<Vec<FulfillmentEvent>>, Status> {
    // IDOR: verify caller owns this order or is privileged
    let owner = sqlx::query_scalar::<_, String>("SELECT user_id FROM orders WHERE id = ?")
        .bind(&order_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("list_fulfillment_events: select order owner failed: {}", e); Status::InternalServerError })?;
    match owner {
        Some(uid) => {
            if uid != user.user_id && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String, Option<String>, String, Option<String>, Option<String>, String, Option<chrono::NaiveDateTime>)>(
        "SELECT id, order_id, line_item_id, event_type, issue_identifier, reason, expected_date, actual_date, logged_by, created_at FROM fulfillment_events WHERE order_id = ? ORDER BY created_at DESC"
    )
    .bind(&order_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("list_fulfillment_events: select events query failed: {}", e); Status::InternalServerError })?;

    let events: Vec<FulfillmentEvent> = rows.into_iter().map(|(id, oid, lid, et, ii, reason, ed, ad, lb, ca)| {
        FulfillmentEvent { id, order_id: oid, line_item_id: lid, event_type: et, issue_identifier: ii, reason, expected_date: ed, actual_date: ad, logged_by: lb, created_at: ca }
    }).collect();

    Ok(Json(events))
}

#[get("/<order_id>/reconciliation")]
pub async fn get_reconciliation(pool: &State<DbPool>, user: AuthenticatedUser, order_id: String) -> Result<Json<Vec<ReconciliationRecord>>, Status> {
    // Staff/admin always allowed; order owner also allowed
    if !user.is_privileged() {
        let owner = sqlx::query_scalar::<_, String>("SELECT user_id FROM orders WHERE id = ?")
            .bind(&order_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("get_reconciliation: select order owner failed: {}", e); Status::InternalServerError })?;
        match owner {
            Some(uid) if uid == user.user_id => {} // order owner — allow
            Some(_) => return Err(Status::Forbidden),
            None => return Err(Status::NotFound),
        }
    }

    let rows = sqlx::query_as::<_, (String, String, Option<String>, String, i32, i32, String, Option<String>)>(
        "SELECT id, order_id, line_item_id, issue_identifier, expected_qty, received_qty, status, notes FROM reconciliation_records WHERE order_id = ? ORDER BY issue_identifier"
    )
    .bind(&order_id).fetch_all(pool.inner()).await.map_err(|e| { log::error!("get_reconciliation: select records query failed: {}", e); Status::InternalServerError })?;

    let records: Vec<ReconciliationRecord> = rows.into_iter().map(|(id, oid, lid, ii, eq, rq, status, notes)| {
        ReconciliationRecord { id, order_id: oid, line_item_id: lid, issue_identifier: ii, expected_qty: eq, received_qty: rq, status, notes }
    }).collect();

    Ok(Json(records))
}

#[put("/reconciliation/<record_id>", data = "<req>", rank = 1)]
pub async fn update_reconciliation(pool: &State<DbPool>, user: AuthenticatedUser, record_id: String, req: Json<UpdateReconciliationRequest>) -> Result<Status, Status> {
    user.require_privileged()?;

    // Get expected qty
    let expected = sqlx::query_scalar::<_, i32>("SELECT expected_qty FROM reconciliation_records WHERE id = ?")
        .bind(&record_id).fetch_optional(pool.inner()).await.map_err(|e| { log::error!("update_reconciliation: select expected_qty failed: {}", e); Status::InternalServerError })?;

    let status = match expected {
        Some(eq) => if req.received_qty == eq { "matched" } else { "discrepancy" },
        None => return Err(Status::NotFound),
    };

    sqlx::query("UPDATE reconciliation_records SET received_qty = ?, status = ?, notes = ?, updated_at = NOW() WHERE id = ?")
        .bind(req.received_qty).bind(status).bind(&req.notes).bind(&record_id)
        .execute(pool.inner()).await.map_err(|e| { log::error!("update_reconciliation: update record failed: {}", e); Status::InternalServerError })?;

    Ok(Status::Ok)
}

#[post("/clear-flag", data = "<req>")]
pub async fn clear_flag(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<ClearFlagRequest>) -> Result<Status, Status> {
    user.require_permission("orders.manage")?;

    sqlx::query("UPDATE orders SET is_flagged = false, flag_cleared_by = ?, flag_cleared_at = NOW(), updated_at = NOW() WHERE id = ?")
        .bind(&user.user_id).bind(&req.order_id).execute(pool.inner()).await.map_err(|e| { log::error!("clear_flag: update order flag failed: {}", e); Status::InternalServerError })?;

    sqlx::query("UPDATE abnormal_order_flags SET is_cleared = true, cleared_by = ?, cleared_at = NOW() WHERE order_id = ?")
        .bind(&user.user_id).bind(&req.order_id).execute(pool.inner()).await.map_err(|e| { log::error!("clear_flag: update abnormal_order_flags failed: {}", e); Status::InternalServerError })?;

    Ok(Status::Ok)
}

/// Generate or update a reconciliation record when a fulfillment event is logged.
/// - "delivered" events increment received_qty and reconcile against expected.
/// - Other events (missing_issue, reshipment, etc.) create a pending record if none exists.
async fn generate_reconciliation_record(
    pool: &DbPool,
    order_id: &str,
    line_item_id: Option<&str>,
    issue_identifier: &str,
    event_type: &str,
) {
    // Check if a reconciliation record already exists for this order+issue
    let existing = sqlx::query_as::<_, (String, i32, i32)>(
        "SELECT id, expected_qty, received_qty FROM reconciliation_records WHERE order_id = ? AND issue_identifier = ?"
    )
    .bind(order_id)
    .bind(issue_identifier)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match existing {
        Some((rec_id, expected_qty, received_qty)) => {
            // Update existing record based on event type
            if event_type == "delivered" {
                let new_received = received_qty + 1;
                let status = if new_received == expected_qty { "matched" } else { "discrepancy" };
                let _ = sqlx::query(
                    "UPDATE reconciliation_records SET received_qty = ?, status = ?, updated_at = NOW() WHERE id = ?"
                )
                .bind(new_received)
                .bind(status)
                .bind(&rec_id)
                .execute(pool)
                .await;
            } else {
                // Non-delivery events mark record as discrepancy if it was matched
                let _ = sqlx::query(
                    "UPDATE reconciliation_records SET status = 'discrepancy', notes = CONCAT(COALESCE(notes, ''), ?), updated_at = NOW() WHERE id = ? AND status = 'matched'"
                )
                .bind(format!("; {} event logged", event_type))
                .bind(&rec_id)
                .execute(pool)
                .await;
            }
        }
        None => {
            // Create new reconciliation record
            let rec_id = Uuid::new_v4().to_string();
            // For delivered events, start with received_qty=1; otherwise 0 (pending)
            let (received_qty, status) = if event_type == "delivered" {
                (1, "discrepancy") // 1 received vs expected from line item qty
            } else {
                (0, "pending")
            };

            // Look up expected quantity from line item if available
            let expected_qty = if let Some(li_id) = line_item_id {
                sqlx::query_scalar::<_, i32>(
                    "SELECT quantity FROM order_line_items WHERE id = ?"
                )
                .bind(li_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .unwrap_or(1)
            } else {
                1
            };

            let final_status = if event_type == "delivered" && received_qty == expected_qty {
                "matched"
            } else {
                status
            };

            let _ = sqlx::query(
                "INSERT INTO reconciliation_records (id, order_id, line_item_id, issue_identifier, expected_qty, received_qty, status, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())"
            )
            .bind(&rec_id)
            .bind(order_id)
            .bind(line_item_id)
            .bind(issue_identifier)
            .bind(expected_qty)
            .bind(received_qty)
            .bind(final_status)
            .bind(format!("Auto-generated from {} event", event_type))
            .execute(pool)
            .await;
        }
    }
}

/// Generate initial reconciliation records for a newly created order.
/// Creates one pending record per line item so reconciliation tracking starts at order creation.
pub async fn generate_initial_reconciliation(pool: &DbPool, order_id: &str) {
    let items = sqlx::query_as::<_, (String, String, Option<String>, i32)>(
        "SELECT id, publication_title, series_name, quantity FROM order_line_items WHERE order_id = ?"
    )
    .bind(order_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (li_id, pub_title, series, qty) in items {
        let issue_id = format!("{}-initial", series.as_deref().unwrap_or(&pub_title));
        let rec_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO reconciliation_records (id, order_id, line_item_id, issue_identifier, expected_qty, received_qty, status, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, 'pending', 'Auto-generated at order creation', NOW(), NOW())"
        )
        .bind(&rec_id)
        .bind(order_id)
        .bind(&li_id)
        .bind(&issue_id)
        .bind(qty)
        .execute(pool)
        .await;
    }
}

#[get("/flagged")]
pub async fn list_flagged_orders(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Order>>, Status> {
    user.require_permission("orders.manage")?;

    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, f64, Option<String>, bool, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, user_id, order_number, subscription_period, shipping_address_id, status, payment_status, CAST(total_amount AS DOUBLE), parent_order_id, is_flagged, flag_reason, created_at, updated_at FROM orders WHERE is_flagged = true ORDER BY created_at DESC"
    )
    .fetch_all(pool.inner()).await.map_err(|e| { log::error!("list_flagged_orders: select flagged orders failed: {}", e); Status::InternalServerError })?;

    let orders: Vec<Order> = rows.into_iter().map(|(id, uid, on, sp, said, s, ps, ta, p, f, fr, ca, ua)| {
        Order { id, user_id: uid, order_number: on, subscription_period: sp, shipping_address_id: said, status: s, payment_status: ps, total_amount: ta, parent_order_id: p, is_flagged: f, flag_reason: fr, created_at: ca, updated_at: ua }
    }).collect();

    Ok(Json(orders))
}
