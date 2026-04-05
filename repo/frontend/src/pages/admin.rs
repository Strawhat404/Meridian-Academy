use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::api;
use crate::Route;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DashboardStats {
    total_users: i64,
    total_submissions: i64,
    total_orders: i64,
    pending_cases: i64,
    flagged_orders: i64,
    total_revenue: f64,
    blocked_content: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserInfo {
    id: String,
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditEntry {
    id: String,
    user_id: Option<String>,
    action: String,
    target_type: Option<String>,
    target_id: Option<String>,
    details: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensitiveWord {
    id: String,
    word: String,
    action: String,
    replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlaggedOrder {
    id: String,
    order_number: String,
    user_id: String,
    flag_reason: Option<String>,
    total_amount: f64,
}

#[derive(Serialize)]
struct AddSensitiveWord {
    word: String,
    action: String,
    replacement: Option<String>,
}

#[derive(Serialize)]
struct ClearFlag {
    order_id: String,
}

#[component]
pub fn AdminPage() -> Element {
    let stats = use_resource(|| async {
        api::get::<DashboardStats>("/api/admin/dashboard").await.ok()
    });

    let mut active_tab = use_signal(|| "overview".to_string());
    let mut tab_msg = use_signal(|| Option::<String>::None);

    // Audit log
    let audit_log = use_resource(move || async move {
        if *active_tab.read() == "audit" {
            api::get::<Vec<AuditEntry>>("/api/admin/audit-logs").await.unwrap_or_default()
        } else {
            vec![]
        }
    });

    // Sensitive words
    let sensitive_words = use_resource(move || async move {
        if *active_tab.read() == "sensitive" {
            api::get::<Vec<SensitiveWord>>("/api/content/sensitive-words").await.unwrap_or_default()
        } else {
            vec![]
        }
    });

    // Flagged orders
    let flagged_orders = use_resource(move || async move {
        if *active_tab.read() == "flagged" {
            api::get::<Vec<FlaggedOrder>>("/api/orders/flagged").await.unwrap_or_default()
        } else {
            vec![]
        }
    });

    // Sensitive word form
    let mut sw_word = use_signal(String::new);
    let mut sw_action = use_signal(|| "replace".to_string());
    let mut sw_replacement = use_signal(String::new);

    let stats_read = stats.read();

    rsx! {
        div { class: "page-container",
            h2 { "Administration Dashboard" }

            if let Some(msg) = tab_msg.read().as_ref() {
                div { class: "status-badge status-active", "{msg}" }
            }

            // Tab bar
            div { style: "display:flex;gap:8px;margin-bottom:16px;flex-wrap:wrap;",
                button { class: if *active_tab.read() == "overview" { "btn" } else { "btn btn-secondary" }, onclick: move |_| active_tab.set("overview".to_string()), "Overview" }
                button { class: if *active_tab.read() == "users" { "btn" } else { "btn btn-secondary" }, onclick: move |_| active_tab.set("users".to_string()), "Users" }
                button { class: if *active_tab.read() == "audit" { "btn" } else { "btn btn-secondary" }, onclick: move |_| active_tab.set("audit".to_string()), "Audit Log" }
                button { class: if *active_tab.read() == "sensitive" { "btn" } else { "btn btn-secondary" }, onclick: move |_| active_tab.set("sensitive".to_string()), "Sensitive Words" }
                button { class: if *active_tab.read() == "flagged" { "btn" } else { "btn btn-secondary" }, onclick: move |_| active_tab.set("flagged".to_string()), "Flagged Orders" }
            }

            // Overview tab
            if *active_tab.read() == "overview" {
                if let Some(Some(s)) = stats_read.as_ref() {
                    div { class: "stats-grid",
                        div { class: "stat-card", h3 { "{s.total_users}" } p { "Total Users" } }
                        div { class: "stat-card", h3 { "{s.total_submissions}" } p { "Submissions" } }
                        div { class: "stat-card", h3 { "{s.total_orders}" } p { "Orders" } }
                        div { class: "stat-card", h3 { "{s.pending_cases}" } p { "Pending Cases" } }
                        div { class: "stat-card", h3 { "{s.flagged_orders}" } p { "Flagged Orders" } }
                        div { class: "stat-card", h3 { "${s.total_revenue:.2}" } p { "Revenue" } }
                        div { class: "stat-card", h3 { "{s.blocked_content}" } p { "Blocked Content" } }
                    }
                } else {
                    p { "Loading dashboard..." }
                }
            }

            // Users tab
            if *active_tab.read() == "users" {
                Link { to: Route::AdminUsers {}, class: "btn btn-primary", style: "margin-bottom:12px;display:inline-block;", "Manage Users" }
            }

            // Audit log tab
            if *active_tab.read() == "audit" {
                if let Some(entries) = audit_log.read().as_ref() {
                    if entries.is_empty() {
                        p { "No audit entries found." }
                    } else {
                        div { class: "table-container",
                            table { class: "data-table",
                                thead { tr { th { "Action" } th { "Target" } th { "Details" } th { "When" } } }
                                tbody {
                                    for entry in entries.iter() {
                                        tr { key: "{entry.id}",
                                            td { "{entry.action}" }
                                            td {
                                                if let (Some(ref tt), Some(ref tid)) = (&entry.target_type, &entry.target_id) {
                                                    "{tt}: {tid}"
                                                } else {
                                                    "—"
                                                }
                                            }
                                            td { if let Some(ref d) = entry.details { "{d}" } else { "—" } }
                                            td { if let Some(ref ca) = entry.created_at { "{ca}" } else { "—" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p { "Loading audit log..." }
                }
            }

            // Sensitive words tab
            if *active_tab.read() == "sensitive" {
                if let Some(words) = sensitive_words.read().as_ref() {
                    if words.is_empty() {
                        p { "No sensitive words configured." }
                    } else {
                        div { class: "table-container",
                            table { class: "data-table",
                                thead { tr { th { "Word" } th { "Action" } th { "Replacement" } th { "Remove" } } }
                                tbody {
                                    for sw in words.iter() {
                                        tr { key: "{sw.id}",
                                            td { "{sw.word}" }
                                            td { "{sw.action}" }
                                            td { if let Some(ref r) = sw.replacement { "{r}" } else { "—" } }
                                            td {
                                                button {
                                                    class: "btn btn-small",
                                                    onclick: {
                                                        let id = sw.id.clone();
                                                        move |_| {
                                                            let id = id.clone();
                                                            spawn(async move {
                                                                let url = format!("/api/content/sensitive-words/{}", id);
                                                                match api::delete(&url).await {
                                                                    Ok(_) => tab_msg.set(Some("Word removed".to_string())),
                                                                    Err(e) => tab_msg.set(Some(e)),
                                                                }
                                                            });
                                                        }
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                h4 { "Add Sensitive Word" }
                form {
                    onsubmit: move |_: FormEvent| {
                        let repl = sw_replacement.read().clone();
                        let data = AddSensitiveWord {
                            word: sw_word.read().clone(),
                            action: sw_action.read().clone(),
                            replacement: if repl.is_empty() { None } else { Some(repl) },
                        };
                        spawn(async move {
                            match api::post::<SensitiveWord, _>("/api/content/sensitive-words", &data).await {
                                Ok(_) => tab_msg.set(Some("Word added".to_string())),
                                Err(e) => tab_msg.set(Some(e)),
                            }
                        });
                    },
                    div { class: "form-group",
                        label { "Word" }
                        input { r#type: "text", value: "{sw_word}", oninput: move |e: FormEvent| sw_word.set(e.value()), required: true }
                    }
                    div { class: "form-group",
                        label { "Policy" }
                        select {
                            value: "{sw_action}",
                            onchange: move |e: FormEvent| sw_action.set(e.value()),
                            option { value: "replace", "Replace" }
                            option { value: "block", "Block" }
                        }
                    }
                    div { class: "form-group",
                        label { "Replacement (for replace policy)" }
                        input { r#type: "text", value: "{sw_replacement}", oninput: move |e: FormEvent| sw_replacement.set(e.value()), placeholder: "[REDACTED]" }
                    }
                    button { r#type: "submit", class: "btn btn-primary", "Add Word" }
                }
            }

            // Flagged orders tab
            if *active_tab.read() == "flagged" {
                if let Some(orders) = flagged_orders.read().as_ref() {
                    if orders.is_empty() {
                        p { "No flagged orders." }
                    } else {
                        div { class: "table-container",
                            table { class: "data-table",
                                thead { tr { th { "Order #" } th { "User" } th { "Total" } th { "Reason" } th { "Action" } } }
                                tbody {
                                    for order in orders.iter() {
                                        tr { key: "{order.id}",
                                            td { "{order.order_number}" }
                                            td { "{order.user_id}" }
                                            td { "${order.total_amount:.2}" }
                                            td { if let Some(ref r) = order.flag_reason { "{r}" } else { "—" } }
                                            td {
                                                button {
                                                    class: "btn btn-small",
                                                    onclick: {
                                                        let oid = order.id.clone();
                                                        move |_| {
                                                            let data = ClearFlag { order_id: oid.clone() };
                                                            spawn(async move {
                                                                match api::post_empty("/api/orders/clear-flag", &data).await {
                                                                    Ok(_) => tab_msg.set(Some("Flag cleared".to_string())),
                                                                    Err(e) => tab_msg.set(Some(e)),
                                                                }
                                                            });
                                                        }
                                                    },
                                                    "Clear Flag"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p { "Loading flagged orders..." }
                }
            }
        }
    }
}

#[component]
pub fn AdminUsersPage() -> Element {
    let users = use_resource(|| async {
        api::get::<Vec<UserInfo>>("/api/users").await.unwrap_or_default()
    });

    let users_read = users.read();

    rsx! {
        div { class: "page-container",
            h2 { "User Management" }
            if let Some(user_list) = users_read.as_ref() {
                div { class: "table-container",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "Username" }
                                th { "Name" }
                                th { "Email" }
                                th { "Role" }
                                th { "Status" }
                            }
                        }
                        tbody {
                            for u in user_list.iter() {
                                tr { key: "{u.id}",
                                    td { "{u.username}" }
                                    td { "{u.first_name} {u.last_name}" }
                                    td { "{u.email}" }
                                    td { "{u.role}" }
                                    td {
                                        if u.is_active {
                                            span { class: "status-badge status-active", "Active" }
                                        } else {
                                            span { class: "status-badge status-inactive", "Inactive" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                p { "Loading users..." }
            }
        }
    }
}
