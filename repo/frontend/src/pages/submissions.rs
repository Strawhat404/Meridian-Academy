use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::api;
use crate::Route;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Submission {
    id: String,
    author_id: String,
    title: String,
    summary: Option<String>,
    submission_type: String,
    status: String,
    current_version: i32,
    max_versions: i32,
    deadline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionVersion {
    id: String,
    version_number: i32,
    file_name: String,
    file_size: i64,
    file_type: String,
    file_hash: String,
    submitted_at: Option<String>,
}

#[derive(Serialize)]
struct CreateSubmission {
    title: String,
    summary: Option<String>,
    submission_type: String,
    tags: Option<String>,
    deadline: Option<String>,
}

#[derive(Serialize)]
struct SubmitVersion {
    file_name: String,
    file_data: String, // base64
    form_data: Option<String>,
}

#[component]
pub fn SubmissionsPage() -> Element {
    let submissions = use_resource(|| async {
        api::get::<Vec<Submission>>("/api/submissions/my").await.unwrap_or_default()
    });

    let mut selected_id = use_signal(|| Option::<String>::None);
    let versions = use_resource(move || async move {
        match selected_id.read().clone() {
            Some(id) => api::get::<Vec<SubmissionVersion>>(&format!("/api/submissions/{}/versions", id)).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // Upload version form
    let mut upload_file_name = use_signal(String::new);
    let mut upload_file_data = use_signal(String::new);
    let mut upload_msg = use_signal(|| Option::<String>::None);

    let subs_read = submissions.read();

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h2 { "My Submissions" }
                Link { to: Route::NewSubmission {}, class: "btn btn-primary", "New Submission" }
            }

            if let Some(msg) = upload_msg.read().as_ref() {
                div { class: "status-badge status-active", "{msg}" }
            }

            if let Some(subs) = subs_read.as_ref() {
                div { class: "table-container",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "Title" }
                                th { "Type" }
                                th { "Status" }
                                th { "Versions" }
                                th { "Deadline" }
                                th { "Actions" }
                            }
                        }
                        tbody {
                            for sub in subs.iter() {
                                tr { key: "{sub.id}",
                                    td { "{sub.title}" }
                                    td { "{sub.submission_type}" }
                                    td {
                                        span { class: "status-badge status-{sub.status}", "{sub.status}" }
                                    }
                                    td { "{sub.current_version}/{sub.max_versions}" }
                                    td {
                                        if let Some(ref dl) = sub.deadline {
                                            "{dl}"
                                        } else {
                                            "—"
                                        }
                                    }
                                    td {
                                        button {
                                            class: "btn btn-small",
                                            onclick: {
                                                let id = sub.id.clone();
                                                move |_| selected_id.set(Some(id.clone()))
                                            },
                                            "History"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                p { "Loading submissions..." }
            }

            // Version history panel
            if let Some(ref sid) = selected_id.read().clone() {
                div { class: "profile-card",
                    div { class: "page-header",
                        h3 { "Version History" }
                        button { class: "btn btn-small", onclick: move |_| selected_id.set(None), "Close" }
                    }

                    if let Some(vers) = versions.read().as_ref() {
                        if vers.is_empty() {
                            p { "No versions submitted yet." }
                        } else {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Version" }
                                        th { "File" }
                                        th { "Size" }
                                        th { "Type" }
                                        th { "Submitted" }
                                        th { "Download" }
                                    }
                                }
                                tbody {
                                    for v in vers.iter() {
                                        tr { key: "{v.id}",
                                            td { "v{v.version_number}" }
                                            td { "{v.file_name}" }
                                            td { "{v.file_size} bytes" }
                                            td { "{v.file_type}" }
                                            td {
                                                if let Some(ref ts) = v.submitted_at {
                                                    "{ts}"
                                                } else {
                                                    "—"
                                                }
                                            }
                                            td {
                                                a {
                                                    class: "btn btn-small",
                                                    href: "/api/submissions/{sid}/versions/{v.version_number}/download",
                                                    target: "_blank",
                                                    "Download"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Upload new version
                    h4 { "Submit New Version" }
                    div { class: "form-group",
                        label { "Select file (PDF, DOCX, PNG, JPG — max 25 MB)" }
                        input {
                            r#type: "file",
                            accept: ".pdf,.docx,.png,.jpg,.jpeg",
                            onchange: {
                                let sid = sid.clone();
                                move |e: FormEvent| {
                                    let files = e.files();
                                    if let Some(file_engine) = files {
                                        let file_list = file_engine.files();
                                        if let Some(fname) = file_list.first() {
                                            let fname = fname.clone();
                                            upload_file_name.set(fname.clone());
                                            let sid = sid.clone();
                                            spawn(async move {
                                                if let Some(bytes) = file_engine.read_file(&fname).await {
                                                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                                    upload_file_data.set(b64);
                                                    let data = SubmitVersion {
                                                        file_name: fname,
                                                        file_data: upload_file_data.read().clone(),
                                                        form_data: None,
                                                    };
                                                    let url = format!("/api/submissions/{}/versions", sid);
                                                    match api::post::<SubmissionVersion, _>(&url, &data).await {
                                                        Ok(v) => upload_msg.set(Some(format!("Version {} submitted", v.version_number))),
                                                        Err(e) => upload_msg.set(Some(e)),
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !upload_file_name.read().is_empty() {
                        p { class: "text-light", "Selected: {upload_file_name}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NewSubmissionPage() -> Element {
    let mut title = use_signal(String::new);
    let mut summary = use_signal(String::new);
    let mut submission_type = use_signal(|| "journal_article".to_string());
    let mut tags = use_signal(String::new);
    let mut deadline = use_signal(String::new);
    let mut message = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let onsubmit = move |_: FormEvent| {
        let nav = nav.clone();
        let dl = deadline.read().clone();
        let data = CreateSubmission {
            title: title.read().clone(),
            summary: { let s = summary.read().clone(); if s.is_empty() { None } else { Some(s) } },
            submission_type: submission_type.read().clone(),
            tags: { let t = tags.read().clone(); if t.is_empty() { None } else { Some(t) } },
            deadline: if dl.is_empty() { None } else { Some(format!("{}:00", dl)) },
        };
        spawn(async move {
            match api::post::<Submission, _>("/api/submissions", &data).await {
                Ok(_) => { nav.push(Route::Submissions {}); },
                Err(e) => { message.set(Some(e)); },
            }
        });
    };

    rsx! {
        div { class: "page-container",
            h2 { "New Submission" }
            if let Some(msg) = message.read().as_ref() {
                div { class: "error-message", "{msg}" }
            }
            form { onsubmit,
                div { class: "form-group",
                    label { "Title (max 120 chars)" }
                    input {
                        r#type: "text",
                        value: "{title}",
                        oninput: move |e: FormEvent| title.set(e.value()),
                        required: true,
                        maxlength: "120"
                    }
                }
                div { class: "form-group",
                    label { "Type" }
                    select {
                        value: "{submission_type}",
                        onchange: move |e: FormEvent| submission_type.set(e.value()),
                        option { value: "journal_article", "Journal Article" }
                        option { value: "conference_paper", "Conference Paper" }
                        option { value: "thesis", "Thesis" }
                        option { value: "book_chapter", "Book Chapter" }
                    }
                }
                div { class: "form-group",
                    label { "Summary (max 500 chars)" }
                    textarea {
                        value: "{summary}",
                        oninput: move |e: FormEvent| summary.set(e.value()),
                        rows: "4",
                        maxlength: "500"
                    }
                }
                div { class: "form-group",
                    label { "Tags (comma-separated)" }
                    input {
                        r#type: "text",
                        value: "{tags}",
                        oninput: move |e: FormEvent| tags.set(e.value()),
                        placeholder: "research, machine-learning, ..."
                    }
                }
                div { class: "form-group",
                    label { "Deadline (optional)" }
                    input {
                        r#type: "datetime-local",
                        value: "{deadline}",
                        oninput: move |e: FormEvent| deadline.set(e.value()),
                    }
                }
                button { r#type: "submit", class: "btn btn-primary", "Create Draft" }
            }
        }
    }
}
