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
    case_type: String,
    subject: String,
    description: String,
    status: String,
    priority: String,
}

#[derive(Serialize)]
struct CreateCase {
    order_id: String,
    case_type: String,
    subject: String,
    description: String,
    priority: Option<String>,
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
