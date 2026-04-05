use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::{api, auth};
use crate::Route;

// --- Models matching backend API ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    user_id: String,
    order_number: String,
    subscription_period: String,
    status: String,
    payment_status: String,
    total_amount: f64,
    is_flagged: bool,
    flag_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderWithItems {
    order: Order,
    line_items: Vec<LineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LineItem {
    id: String,
    order_id: String,
    publication_title: String,
    series_name: Option<String>,
    quantity: i32,
    unit_price: f64,
    line_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FulfillmentEvent {
    id: String,
    order_id: String,
    line_item_id: Option<String>,
    event_type: String,
    issue_identifier: Option<String>,
    reason: String,
    expected_date: Option<String>,
    actual_date: Option<String>,
    logged_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReconciliationRecord {
    id: String,
    order_id: String,
    line_item_id: Option<String>,
    issue_identifier: String,
    expected_qty: i32,
    received_qty: i32,
    status: String,
    notes: Option<String>,
}

#[derive(Serialize)]
struct CreateLineItem {
    publication_title: String,
    series_name: Option<String>,
    quantity: i32,
    unit_price: f64,
}

#[derive(Serialize)]
struct CreateOrder {
    subscription_period: String,
    shipping_address_id: Option<String>,
    line_items: Vec<CreateLineItem>,
}

#[derive(Serialize)]
struct SplitReq { order_id: String }

#[derive(Serialize)]
struct MergeReq { order_ids: Vec<String> }

#[derive(Serialize)]
struct FulfillmentReq {
    order_id: String,
    line_item_id: Option<String>,
    event_type: String,
    issue_identifier: Option<String>,
    reason: String,
    expected_date: Option<String>,
    actual_date: Option<String>,
}

#[derive(Serialize)]
struct ReconciliationUpdate {
    received_qty: i32,
    notes: Option<String>,
}

// ===== Orders List Page =====

#[component]
pub fn OrdersPage() -> Element {
    let user = auth::get_current_user();
    let is_staff = user.as_ref().map_or(false, |u| u.role == "administrator" || u.role == "academic_staff");

    // Staff/admin see ALL orders via /api/orders; regular users see only their own via /api/orders/my
    let orders = use_resource({
        let staff = is_staff;
        move || async move {
            let endpoint = if staff { "/api/orders" } else { "/api/orders/my" };
            api::get::<Vec<Order>>(endpoint).await.unwrap_or_default()
        }
    });

    // Detail view
    let mut selected_order = use_signal(|| Option::<String>::None);
    let order_detail = use_resource(move || async move {
        match selected_order.read().clone() {
            Some(id) => api::get::<OrderWithItems>(&format!("/api/orders/{}", id)).await.ok(),
            None => None,
        }
    });

    // Fulfillment events
    let fulfillment_events = use_resource(move || async move {
        match selected_order.read().clone() {
            Some(id) => api::get::<Vec<FulfillmentEvent>>(&format!("/api/orders/{}/fulfillment", id)).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // Reconciliation
    let reconciliation = use_resource(move || async move {
        match selected_order.read().clone() {
            Some(id) => api::get::<Vec<ReconciliationRecord>>(&format!("/api/orders/{}/reconciliation", id)).await.unwrap_or_default(),
            None => vec![],
        }
    });

    let mut msg = use_signal(|| Option::<String>::None);

    // Fulfillment form
    let mut fe_type = use_signal(|| "missing_issue".to_string());
    let mut fe_issue = use_signal(String::new);
    let mut fe_reason = use_signal(String::new);

    // Merge form
    let mut merge_ids = use_signal(String::new);

    let ords_read = orders.read();

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h2 { "Orders" }
                Link { to: Route::NewOrder {}, class: "btn btn-primary", "New Order" }
            }

            if let Some(m) = msg.read().as_ref() {
                div { class: "status-badge status-active", "{m}" }
            }

            // --- Orders Table ---
            if let Some(ords) = ords_read.as_ref() {
                div { class: "table-container",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "Order #" }
                                th { "Period" }
                                th { "Total" }
                                th { "Status" }
                                th { "Payment" }
                                th { "Actions" }
                            }
                        }
                        tbody {
                            for order in ords.iter() {
                                tr { key: "{order.id}",
                                    td { "{order.order_number}" }
                                    td { "{order.subscription_period}" }
                                    td { "${order.total_amount:.2}" }
                                    td { span { class: "status-badge status-{order.status}", "{order.status}" } }
                                    td { span { class: "status-badge payment-{order.payment_status}", "{order.payment_status}" } }
                                    td {
                                        button {
                                            class: "btn btn-small",
                                            onclick: { let oid = order.id.clone(); move |_| selected_order.set(Some(oid.clone())) },
                                            "Detail"
                                        }
                                        if is_staff {
                                            button {
                                                class: "btn btn-small",
                                                onclick: {
                                                    let oid = order.id.clone();
                                                    move |_| {
                                                        let oid = oid.clone();
                                                        spawn(async move {
                                                            match api::post::<Vec<Order>, _>("/api/orders/split", &SplitReq { order_id: oid }).await {
                                                                Ok(new_orders) => { msg.set(Some(format!("Split into {} orders", new_orders.len()))); }
                                                                Err(e) => { msg.set(Some(e)); }
                                                            }
                                                        });
                                                    }
                                                },
                                                "Split"
                                            }
                                        }
                                        if order.is_flagged {
                                            span { class: "status-badge status-rejected", "FLAGGED" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                p { "Loading orders..." }
            }

            // --- Staff: Merge Orders ---
            if is_staff {
                div { class: "profile-card",
                    h4 { "Merge Orders" }
                    p { class: "text-light", "Enter comma-separated Order IDs to merge (same subscriber)" }
                    div { class: "form-group",
                        input { r#type: "text", value: "{merge_ids}", oninput: move |e: FormEvent| merge_ids.set(e.value()), placeholder: "id1, id2, id3" }
                    }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            let ids: Vec<String> = merge_ids.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                            spawn(async move {
                                match api::post::<OrderWithItems, _>("/api/orders/merge", &MergeReq { order_ids: ids }).await {
                                    Ok(merged) => { msg.set(Some(format!("Merged into order {}", merged.order.order_number))); }
                                    Err(e) => { msg.set(Some(e)); }
                                }
                            });
                        },
                        "Merge"
                    }
                }
            }

            // --- Order Detail Panel ---
            if let Some(ref oid) = selected_order.read().clone() {
                div { class: "profile-card",
                    div { class: "page-header",
                        h3 { "Order Detail" }
                        button { class: "btn btn-small", onclick: move |_| selected_order.set(None), "Close" }
                    }

                    if let Some(Some(detail)) = order_detail.read().as_ref() {
                        p { "Order: {detail.order.order_number} | Status: {detail.order.status} | Payment: {detail.order.payment_status}" }

                        h4 { "Line Items" }
                        table { class: "data-table",
                            thead { tr { th { "Publication" } th { "Series" } th { "Qty" } th { "Price" } th { "Total" } } }
                            tbody {
                                for item in detail.line_items.iter() {
                                    tr { key: "{item.id}",
                                        td { "{item.publication_title}" }
                                        td { if let Some(ref sn) = item.series_name { "{sn}" } else { "—" } }
                                        td { "{item.quantity}" }
                                        td { "${item.unit_price:.2}" }
                                        td { "${item.line_total:.2}" }
                                    }
                                }
                            }
                        }
                    }

                    // --- Fulfillment Events ---
                    h4 { "Fulfillment Events" }
                    if let Some(events) = fulfillment_events.read().as_ref() {
                        if events.is_empty() {
                            p { class: "text-light", "No fulfillment events logged." }
                        } else {
                            table { class: "data-table",
                                thead { tr { th { "Type" } th { "Issue" } th { "Reason" } th { "Expected" } th { "Actual" } } }
                                tbody {
                                    for ev in events.iter() {
                                        tr { key: "{ev.id}",
                                            td { span { class: "status-badge", "{ev.event_type}" } }
                                            td { if let Some(ref ii) = ev.issue_identifier { "{ii}" } else { "—" } }
                                            td { "{ev.reason}" }
                                            td { if let Some(ref d) = ev.expected_date { "{d}" } else { "—" } }
                                            td { if let Some(ref d) = ev.actual_date { "{d}" } else { "—" } }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Staff: Log Fulfillment Event
                    if is_staff {
                        h4 { "Log Fulfillment Event" }
                        div { style: "display:flex;gap:8px;flex-wrap:wrap;align-items:flex-end;",
                            div { class: "form-group",
                                label { "Type" }
                                select { value: "{fe_type}", onchange: move |e: FormEvent| fe_type.set(e.value()),
                                    option { value: "missing_issue", "Missing Issue" }
                                    option { value: "reshipment", "Reshipment" }
                                    option { value: "delay", "Delay" }
                                    option { value: "discontinuation", "Discontinuation" }
                                    option { value: "edition_change", "Edition Change" }
                                    option { value: "delivered", "Delivered" }
                                }
                            }
                            div { class: "form-group",
                                label { "Issue ID" }
                                input { r#type: "text", value: "{fe_issue}", oninput: move |e: FormEvent| fe_issue.set(e.value()), placeholder: "Vol.2 Issue 3" }
                            }
                            div { class: "form-group",
                                label { "Reason (required)" }
                                input { r#type: "text", value: "{fe_reason}", oninput: move |e: FormEvent| fe_reason.set(e.value()), required: true }
                            }
                            button {
                                class: "btn",
                                onclick: {
                                    let oid2 = oid.clone();
                                    move |_| {
                                        let data = FulfillmentReq {
                                            order_id: oid2.clone(),
                                            line_item_id: None,
                                            event_type: fe_type.read().clone(),
                                            issue_identifier: { let s = fe_issue.read().clone(); if s.is_empty() { None } else { Some(s) } },
                                            reason: fe_reason.read().clone(),
                                            expected_date: None,
                                            actual_date: None,
                                        };
                                        spawn(async move {
                                            match api::post::<FulfillmentEvent, _>("/api/orders/fulfillment", &data).await {
                                                Ok(_) => { msg.set(Some("Fulfillment event logged".to_string())); }
                                                Err(e) => { msg.set(Some(e)); }
                                            }
                                        });
                                    }
                                },
                                "Log Event"
                            }
                        }
                    }

                    // --- Reconciliation ---
                    h4 { "Reconciliation" }
                    if let Some(recs) = reconciliation.read().as_ref() {
                        if recs.is_empty() {
                            p { class: "text-light", "No reconciliation records." }
                        } else {
                            table { class: "data-table",
                                thead { tr { th { "Issue" } th { "Expected" } th { "Received" } th { "Status" } th { "Notes" } } }
                                tbody {
                                    for rec in recs.iter() {
                                        tr { key: "{rec.id}",
                                            td { "{rec.issue_identifier}" }
                                            td { "{rec.expected_qty}" }
                                            td { "{rec.received_qty}" }
                                            td {
                                                span {
                                                    class: if rec.status == "matched" { "status-badge status-active" } else if rec.status == "discrepancy" { "status-badge status-rejected" } else { "status-badge status-pending" },
                                                    "{rec.status}"
                                                }
                                            }
                                            td { if let Some(ref n) = rec.notes { "{n}" } else { "—" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===== New Order Page =====

#[component]
pub fn NewOrderPage() -> Element {
    let mut pub_title = use_signal(String::new);
    let mut series = use_signal(String::new);
    let mut quantity = use_signal(|| "1".to_string());
    let mut unit_price = use_signal(|| "29.99".to_string());
    let mut period = use_signal(|| "quarterly".to_string());
    let mut message = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    // Multiple line items
    let mut extra_items: Signal<Vec<(String, Option<String>, i32, f64)>> = use_signal(Vec::new);
    let mut extra_title = use_signal(String::new);
    let mut extra_series = use_signal(String::new);
    let mut extra_qty = use_signal(|| "1".to_string());
    let mut extra_price = use_signal(|| "29.99".to_string());

    let onsubmit = move |_: FormEvent| {
        let nav = nav.clone();
        let qty: i32 = quantity.read().parse().unwrap_or(1);
        let price: f64 = unit_price.read().parse().unwrap_or(29.99);
        let sn = { let s = series.read().clone(); if s.is_empty() { None } else { Some(s) } };

        let mut items = vec![CreateLineItem {
            publication_title: pub_title.read().clone(),
            series_name: sn,
            quantity: qty,
            unit_price: price,
        }];

        for (t, s, q, p) in extra_items.read().iter() {
            items.push(CreateLineItem {
                publication_title: t.clone(),
                series_name: s.clone(),
                quantity: *q,
                unit_price: *p,
            });
        }

        let data = CreateOrder {
            subscription_period: period.read().clone(),
            shipping_address_id: None,
            line_items: items,
        };
        spawn(async move {
            match api::post::<OrderWithItems, _>("/api/orders", &data).await {
                Ok(_) => { nav.push(Route::Orders {}); },
                Err(e) => { message.set(Some(e)); },
            }
        });
    };

    let add_line_item = move |_| {
        let t = extra_title.read().clone();
        let s = { let v = extra_series.read().clone(); if v.is_empty() { None } else { Some(v) } };
        let q: i32 = extra_qty.read().parse().unwrap_or(1);
        let p: f64 = extra_price.read().parse().unwrap_or(29.99);
        extra_items.write().push((t, s, q, p));
        extra_title.set(String::new());
        extra_series.set(String::new());
    };

    rsx! {
        div { class: "page-container",
            h2 { "New Order" }
            if let Some(m) = message.read().as_ref() {
                div { class: "error-message", "{m}" }
            }
            form { onsubmit,
                div { class: "form-group",
                    label { "Subscription Period" }
                    select { value: "{period}", onchange: move |e: FormEvent| period.set(e.value()),
                        option { value: "monthly", "Monthly" }
                        option { value: "quarterly", "Quarterly" }
                        option { value: "annual", "Annual" }
                    }
                }
                h4 { "Line Item #1" }
                div { class: "form-group",
                    label { "Publication Title" }
                    input { r#type: "text", value: "{pub_title}", oninput: move |e: FormEvent| pub_title.set(e.value()), required: true }
                }
                div { class: "form-group",
                    label { "Series (optional)" }
                    input { r#type: "text", value: "{series}", oninput: move |e: FormEvent| series.set(e.value()) }
                }
                div { style: "display:flex;gap:12px;",
                    div { class: "form-group",
                        label { "Quantity" }
                        input { r#type: "number", value: "{quantity}", oninput: move |e: FormEvent| quantity.set(e.value()), min: "1" }
                    }
                    div { class: "form-group",
                        label { "Unit Price ($)" }
                        input { r#type: "number", step: "0.01", value: "{unit_price}", oninput: move |e: FormEvent| unit_price.set(e.value()) }
                    }
                }

                // Additional line items
                if !extra_items.read().is_empty() {
                    for (i, item) in extra_items.read().iter().enumerate() {
                        p { "Item #{i}: {item.0} — qty {item.2} @ ${item.3:.2}" }
                    }
                }

                h4 { "Add Another Line Item" }
                div { style: "display:flex;gap:8px;flex-wrap:wrap;align-items:flex-end;",
                    div { class: "form-group",
                        label { "Title" }
                        input { r#type: "text", value: "{extra_title}", oninput: move |e: FormEvent| extra_title.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Series" }
                        input { r#type: "text", value: "{extra_series}", oninput: move |e: FormEvent| extra_series.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Qty" }
                        input { r#type: "number", value: "{extra_qty}", oninput: move |e: FormEvent| extra_qty.set(e.value()), min: "1", style: "width:60px;" }
                    }
                    div { class: "form-group",
                        label { "Price" }
                        input { r#type: "number", step: "0.01", value: "{extra_price}", oninput: move |e: FormEvent| extra_price.set(e.value()), style: "width:80px;" }
                    }
                    button { r#type: "button", class: "btn btn-secondary", onclick: add_line_item, "+ Add" }
                }

                button { r#type: "submit", class: "btn btn-primary", "Place Order" }
            }
        }
    }
}
