use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::api;
use crate::Route;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseWithSla {
    case: Case,
    first_response_overdue: bool,
    resolution_overdue: bool,
    hours_until_first_response: Option<f64>,
    hours_until_resolution: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    order_id: String,
    reporter_id: String,
    assigned_to: Option<String>,
    case_type: String,
    subject: String,
    description: String,
    status: String,
    priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseComment {
    id: String,
    case_id: String,
    author_id: String,
    content: String,
    created_at: Option<String>,
}

#[derive(Serialize)]
struct CreateCase {
    order_id: String,
    case_type: String,
    subject: String,
    description: String,
    priority: Option<String>,
}

#[derive(Serialize)]
struct UpdateCaseStatus {
    status: String,
}

#[derive(Serialize)]
struct AssignCaseReq {
    assigned_to: String,
}

#[derive(Serialize)]
struct AddCommentReq {
    content: String,
}

#[component]
pub fn CasesPage() -> Element {
    let cases = use_resource(|| async {
        api::get::<Vec<CaseWithSla>>("/api/cases/my").await.unwrap_or_default()
    });

    let cases_read = cases.read();

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h2 { "My Cases" }
                Link { to: Route::NewCase {}, class: "btn btn-primary", "New Case" }
            }
            if let Some(cs) = cases_read.as_ref() {
                div { class: "table-container",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "Subject" }
                                th { "Type" }
                                th { "Priority" }
                                th { "Status" }
                                th { "SLA" }
                            }
                        }
                        tbody {
                            for cws in cs.iter() {
                                tr { key: "{cws.case.id}",
                                    td { "{cws.case.subject}" }
                                    td { "{cws.case.case_type}" }
                                    td {
                                        span { class: "priority-badge priority-{cws.case.priority}", "{cws.case.priority}" }
                                    }
                                    td {
                                        span { class: "status-badge status-{cws.case.status}", "{cws.case.status}" }
                                    }
                                    td {
                                        if cws.first_response_overdue {
                                            span { class: "status-badge status-rejected", "Response Overdue" }
                                        } else if cws.resolution_overdue {
                                            span { class: "status-badge status-rejected", "Resolution Overdue" }
                                        } else {
                                            span { class: "status-badge status-active", "On Track" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                p { "Loading cases..." }
            }
        }
    }
}

#[component]
pub fn NewCasePage() -> Element {
    let mut order_id = use_signal(String::new);
    let mut case_type = use_signal(|| "return".to_string());
    let mut subject = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut priority = use_signal(|| "medium".to_string());
    let mut message = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let onsubmit = move |_: FormEvent| {
        let nav = nav.clone();
        let data = CreateCase {
            order_id: order_id.read().clone(),
            case_type: case_type.read().clone(),
            subject: subject.read().clone(),
            description: description.read().clone(),
            priority: Some(priority.read().clone()),
        };
        spawn(async move {
            match api::post::<CaseWithSla, _>("/api/cases/", &data).await {
                Ok(_) => { nav.push(Route::Cases {}); },
                Err(e) => { message.set(Some(e)); },
            }
        });
    };

    rsx! {
        div { class: "page-container",
            h2 { "New Support Case" }
            if let Some(msg) = message.read().as_ref() {
                div { class: "error-message", "{msg}" }
            }
            form { onsubmit,
                div { class: "form-group",
                    label { "Order ID" }
                    input {
                        r#type: "text",
                        value: "{order_id}",
                        oninput: move |e: FormEvent| order_id.set(e.value()),
                        required: true
                    }
                }
                div { class: "form-group",
                    label { "Case Type" }
                    select {
                        value: "{case_type}",
                        onchange: move |e: FormEvent| case_type.set(e.value()),
                        option { value: "return", "Return" }
                        option { value: "refund", "Refund" }
                        option { value: "exchange", "Exchange" }
                    }
                }
                div { class: "form-group",
                    label { "Subject" }
                    input {
                        r#type: "text",
                        value: "{subject}",
                        oninput: move |e: FormEvent| subject.set(e.value()),
                        required: true
                    }
                }
                div { class: "form-group",
                    label { "Priority" }
                    select {
                        value: "{priority}",
                        onchange: move |e: FormEvent| priority.set(e.value()),
                        option { value: "low", "Low" }
                        option { value: "medium", "Medium" }
                        option { value: "high", "High" }
                        option { value: "urgent", "Urgent" }
                    }
                }
                div { class: "form-group",
                    label { "Description" }
                    textarea {
                        value: "{description}",
                        oninput: move |e: FormEvent| description.set(e.value()),
                        rows: "6",
                        required: true
                    }
                }
                button { r#type: "submit", class: "btn btn-primary", "Submit Case" }
            }
        }
    }
}

/// Staff/admin case detail page — shows full case info, assignment, status transitions, and comments.
#[component]
pub fn AdminCaseDetailPage(case_id: String) -> Element {
    let cid = case_id.clone();
    let cid2 = case_id.clone();

    let case_data = use_resource(move || {
        let cid = cid.clone();
        async move {
            api::get::<CaseWithSla>(&format!("/api/cases/{}", cid)).await.ok()
        }
    });

    let comments = use_resource(move || {
        let cid = cid2.clone();
        async move {
            api::get::<Vec<CaseComment>>(&format!("/api/cases/{}/comments", cid)).await.unwrap_or_default()
        }
    });

    let mut msg = use_signal(|| Option::<String>::None);
    let mut assign_to = use_signal(String::new);
    let mut new_comment = use_signal(String::new);

    let case_read = case_data.read();
    let comments_read = comments.read();

    rsx! {
        div { class: "page-container",
            h2 { "Case Detail" }

            if let Some(msg_text) = msg.read().as_ref() {
                div { class: "status-badge status-active", "{msg_text}" }
            }

            if let Some(Some(cws)) = case_read.as_ref() {
                div { class: "dashboard-grid", style: "margin-bottom: 24px;",
                    div { class: "dashboard-card",
                        h3 { "{cws.case.subject}" }
                        p { strong { "Type: " } "{cws.case.case_type}" }
                        p { strong { "Priority: " }
                            span { class: "priority-badge priority-{cws.case.priority}", "{cws.case.priority}" }
                        }
                        p { strong { "Status: " }
                            span { class: "status-badge status-{cws.case.status}", "{cws.case.status}" }
                        }
                        p { strong { "Reporter: " } "{cws.case.reporter_id}" }
                        p { strong { "Order: " } "{cws.case.order_id}" }
                        p { strong { "Assigned to: " }
                            if let Some(ref a) = cws.case.assigned_to { "{a}" } else { "Unassigned" }
                        }
                    }
                    div { class: "dashboard-card",
                        h3 { "SLA Status" }
                        if cws.first_response_overdue {
                            p { span { class: "status-badge status-rejected", "First Response OVERDUE" } }
                        } else if let Some(hrs) = cws.hours_until_first_response {
                            p { "First response due in {hrs:.1} hours" }
                        }
                        if cws.resolution_overdue {
                            p { span { class: "status-badge status-rejected", "Resolution OVERDUE" } }
                        } else if let Some(hrs) = cws.hours_until_resolution {
                            p { "Resolution due in {hrs:.1} hours" }
                        }
                    }
                }

                // Description
                div { style: "margin-bottom: 24px; padding: 16px; background: #f9f9f9; border-radius: 8px;",
                    h4 { "Description" }
                    p { "{cws.case.description}" }
                }

                // Status transition buttons
                div { style: "margin-bottom: 24px; display: flex; gap: 8px; flex-wrap: wrap;",
                    {
                        let transitions: Vec<(&str, &str)> = match cws.case.status.as_str() {
                            "submitted" => vec![("in_review", "Start Review")],
                            "in_review" => vec![("awaiting_evidence", "Request Evidence"), ("arbitrated", "Arbitrate")],
                            "awaiting_evidence" => vec![("in_review", "Resume Review"), ("arbitrated", "Arbitrate")],
                            "arbitrated" => vec![("approved", "Approve"), ("denied", "Deny")],
                            "approved" | "denied" => vec![("closed", "Close")],
                            _ => vec![],
                        };
                        rsx! {
                            for (status, label) in transitions {
                                button {
                                    class: "btn btn-primary",
                                    onclick: {
                                        let case_id_c = case_id.clone();
                                        let new_status = status.to_string();
                                        move |_| {
                                            let case_id_c = case_id_c.clone();
                                            let data = UpdateCaseStatus { status: new_status.clone() };
                                            spawn(async move {
                                                let url = format!("/api/cases/{}/status", case_id_c);
                                                match api::put_empty(&url, &data).await {
                                                    Ok(_) => msg.set(Some("Status updated".to_string())),
                                                    Err(e) => msg.set(Some(e)),
                                                }
                                            });
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }

                // Assignment form
                div { style: "margin-bottom: 24px; padding: 16px; border: 1px solid #ddd; border-radius: 8px;",
                    h4 { "Assign Case" }
                    form {
                        onsubmit: {
                            let case_id_c = case_id.clone();
                            move |_: FormEvent| {
                                let case_id_c = case_id_c.clone();
                                let data = AssignCaseReq { assigned_to: assign_to.read().clone() };
                                spawn(async move {
                                    let url = format!("/api/cases/{}/assign", case_id_c);
                                    match api::put_empty(&url, &data).await {
                                        Ok(_) => msg.set(Some("Case assigned".to_string())),
                                        Err(e) => msg.set(Some(e)),
                                    }
                                });
                            }
                        },
                        div { style: "display: flex; gap: 8px; align-items: end;",
                            div { class: "form-group", style: "flex: 1;",
                                label { "Staff User ID" }
                                input {
                                    r#type: "text",
                                    value: "{assign_to}",
                                    oninput: move |e: FormEvent| assign_to.set(e.value()),
                                    placeholder: "Enter user ID to assign",
                                    required: true
                                }
                            }
                            button { r#type: "submit", class: "btn btn-primary", "Assign" }
                        }
                    }
                }

                // Comments section
                div { style: "margin-bottom: 24px;",
                    h4 { "Comments" }
                    if let Some(coms) = comments_read.as_ref() {
                        if coms.is_empty() {
                            p { style: "color: #888;", "No comments yet." }
                        } else {
                            for com in coms.iter() {
                                div { key: "{com.id}", style: "padding: 12px; margin-bottom: 8px; background: #f5f5f5; border-radius: 6px; border-left: 3px solid #4a90d9;",
                                    div { style: "display: flex; justify-content: space-between; margin-bottom: 4px;",
                                        strong { "{com.author_id}" }
                                        if let Some(ref ca) = com.created_at {
                                            span { style: "color: #888; font-size: 0.85em;", "{ca}" }
                                        }
                                    }
                                    p { style: "margin: 0;", "{com.content}" }
                                }
                            }
                        }
                    }

                    // Add comment form
                    form {
                        onsubmit: {
                            let case_id_c = case_id.clone();
                            move |_: FormEvent| {
                                let case_id_c = case_id_c.clone();
                                let data = AddCommentReq { content: new_comment.read().clone() };
                                spawn(async move {
                                    let url = format!("/api/cases/{}/comments", case_id_c);
                                    match api::post::<CaseComment, _>(&url, &data).await {
                                        Ok(_) => {
                                            msg.set(Some("Comment added".to_string()));
                                            new_comment.set(String::new());
                                        }
                                        Err(e) => msg.set(Some(e)),
                                    }
                                });
                            }
                        },
                        div { class: "form-group",
                            label { "Add Comment" }
                            textarea {
                                value: "{new_comment}",
                                oninput: move |e: FormEvent| new_comment.set(e.value()),
                                rows: "3",
                                placeholder: "Write a comment...",
                                required: true
                            }
                        }
                        button { r#type: "submit", class: "btn btn-primary", "Post Comment" }
                    }
                }
            } else {
                p { "Loading case..." }
            }
        }
    }
}
