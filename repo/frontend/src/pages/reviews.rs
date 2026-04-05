use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::api;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Review {
    id: String,
    order_id: String,
    line_item_id: Option<String>,
    user_id: String,
    rating: i32,
    title: String,
    body: String,
    is_followup: bool,
    parent_review_id: Option<String>,
}

#[derive(Serialize)]
struct CreateReview {
    order_id: String,
    line_item_id: Option<String>,
    rating: i32,
    title: String,
    body: String,
}

#[derive(Serialize)]
struct CreateFollowup {
    parent_review_id: String,
    rating: i32,
    title: String,
    body: String,
}

#[derive(Serialize)]
struct AddImage {
    file_name: String,
    file_data: String,
}

#[component]
pub fn ReviewsPage() -> Element {
    let reviews = use_resource(|| async {
        api::get::<Vec<Review>>("/api/reviews/my").await.unwrap_or_default()
    });

    // Create review form
    let mut show_create = use_signal(|| false);
    let mut cr_order_id = use_signal(String::new);
    let mut cr_rating = use_signal(|| "5".to_string());
    let mut cr_title = use_signal(String::new);
    let mut cr_body = use_signal(String::new);
    let mut cr_msg = use_signal(|| Option::<String>::None);

    // Followup form
    let mut show_followup = use_signal(|| false);
    let mut fu_parent_id = use_signal(String::new);
    let mut fu_rating = use_signal(|| "5".to_string());
    let mut fu_title = use_signal(String::new);
    let mut fu_body = use_signal(String::new);

    // Image upload
    let mut img_review_id = use_signal(String::new);
    let mut img_name = use_signal(String::new);

    let submit_review = move |_: FormEvent| {
        let data = CreateReview {
            order_id: cr_order_id.read().clone(),
            line_item_id: None,
            rating: cr_rating.read().parse().unwrap_or(5),
            title: cr_title.read().clone(),
            body: cr_body.read().clone(),
        };
        spawn(async move {
            match api::post::<Review, _>("/api/reviews", &data).await {
                Ok(_) => { cr_msg.set(Some("Review submitted".to_string())); show_create.set(false); }
                Err(e) => { cr_msg.set(Some(e)); }
            }
        });
    };

    let submit_followup = move |_: FormEvent| {
        let data = CreateFollowup {
            parent_review_id: fu_parent_id.read().clone(),
            rating: fu_rating.read().parse().unwrap_or(5),
            title: fu_title.read().clone(),
            body: fu_body.read().clone(),
        };
        spawn(async move {
            match api::post::<Review, _>("/api/reviews/followup", &data).await {
                Ok(_) => { cr_msg.set(Some("Follow-up submitted".to_string())); show_followup.set(false); }
                Err(e) => { cr_msg.set(Some(e)); }
            }
        });
    };

    let revs_read = reviews.read();

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h2 { "My Reviews" }
                button { class: "btn btn-primary", onclick: move |_| show_create.set(true), "Write Review" }
            }

            if let Some(msg) = cr_msg.read().as_ref() {
                div { class: "status-badge status-active", "{msg}" }
            }

            // --- Create Review Form ---
            if *show_create.read() {
                div { class: "profile-card",
                    h3 { "New Review" }
                    form { onsubmit: submit_review,
                        div { class: "form-group",
                            label { "Order ID" }
                            input { r#type: "text", value: "{cr_order_id}", oninput: move |e: FormEvent| cr_order_id.set(e.value()), required: true }
                        }
                        div { class: "form-group",
                            label { "Rating (1-5)" }
                            select { value: "{cr_rating}", onchange: move |e: FormEvent| cr_rating.set(e.value()),
                                option { value: "1", "1" } option { value: "2", "2" } option { value: "3", "3" }
                                option { value: "4", "4" } option { value: "5", "5" }
                            }
                        }
                        div { class: "form-group",
                            label { "Title (max 120 chars)" }
                            input { r#type: "text", value: "{cr_title}", oninput: move |e: FormEvent| cr_title.set(e.value()), required: true, maxlength: "120" }
                        }
                        div { class: "form-group",
                            label { "Review Body" }
                            textarea { value: "{cr_body}", oninput: move |e: FormEvent| cr_body.set(e.value()), rows: "5", required: true }
                        }
                        button { r#type: "submit", class: "btn btn-primary", "Submit Review" }
                        button { r#type: "button", class: "btn btn-secondary", onclick: move |_| show_create.set(false), "Cancel" }
                    }
                }
            }

            // --- Follow-up Form ---
            if *show_followup.read() {
                div { class: "profile-card",
                    h3 { "Follow-up Response" }
                    p { class: "text-light", "You may post one follow-up within 14 days of your original review." }
                    form { onsubmit: submit_followup,
                        div { class: "form-group",
                            label { "Parent Review ID" }
                            input { r#type: "text", value: "{fu_parent_id}", oninput: move |e: FormEvent| fu_parent_id.set(e.value()), required: true }
                        }
                        div { class: "form-group",
                            label { "Updated Rating (1-5)" }
                            select { value: "{fu_rating}", onchange: move |e: FormEvent| fu_rating.set(e.value()),
                                option { value: "1", "1" } option { value: "2", "2" } option { value: "3", "3" }
                                option { value: "4", "4" } option { value: "5", "5" }
                            }
                        }
                        div { class: "form-group",
                            label { "Title" }
                            input { r#type: "text", value: "{fu_title}", oninput: move |e: FormEvent| fu_title.set(e.value()), required: true, maxlength: "120" }
                        }
                        div { class: "form-group",
                            label { "Follow-up Body" }
                            textarea { value: "{fu_body}", oninput: move |e: FormEvent| fu_body.set(e.value()), rows: "4", required: true }
                        }
                        button { r#type: "submit", class: "btn btn-primary", "Submit Follow-up" }
                        button { r#type: "button", class: "btn btn-secondary", onclick: move |_| show_followup.set(false), "Cancel" }
                    }
                }
            }

            // --- Image Upload ---
            div { class: "profile-card",
                h4 { "Upload Review Image (max 6 images, 5 MB each)" }
                div { class: "form-group",
                    label { "Select Review" }
                    select {
                        value: "{img_review_id}",
                        onchange: move |e: FormEvent| img_review_id.set(e.value()),
                        option { value: "", "— select a review —" }
                        if let Some(revs) = revs_read.as_ref() {
                            for rev in revs.iter().filter(|r| !r.is_followup) {
                                option { value: "{rev.id}", "{rev.title}" }
                            }
                        }
                    }
                }
                div { class: "form-group",
                    label { "Select Image (PNG or JPG, max 5 MB)" }
                    input {
                        r#type: "file",
                        accept: ".png,.jpg,.jpeg",
                        onchange: move |e: FormEvent| {
                            let files = e.files();
                            if let Some(file_engine) = files {
                                let file_list = file_engine.files();
                                if let Some(fname) = file_list.first() {
                                    let fname = fname.clone();
                                    img_name.set(fname.clone());
                                    let rid = img_review_id.read().clone();
                                    if rid.is_empty() {
                                        cr_msg.set(Some("Please select a review first".to_string()));
                                        return;
                                    }
                                    spawn(async move {
                                        if let Some(bytes) = file_engine.read_file(&fname).await {
                                            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                            let data = AddImage { file_name: fname, file_data: b64 };
                                            let url = format!("/api/reviews/{}/images", rid);
                                            match api::post_empty(&url, &data).await {
                                                Ok(_) => cr_msg.set(Some("Image uploaded".to_string())),
                                                Err(e) => cr_msg.set(Some(e)),
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                if !img_name.read().is_empty() {
                    p { class: "text-light", "Selected: {img_name}" }
                }
            }

            // --- Reviews List ---
            if let Some(revs) = revs_read.as_ref() {
                if revs.is_empty() {
                    p { "No reviews yet. Order a publication and submit a review once delivered." }
                }
                div { class: "table-container",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "Title" }
                                th { "Rating" }
                                th { "Order" }
                                th { "Type" }
                                th { "Actions" }
                            }
                        }
                        tbody {
                            for review in revs.iter() {
                                tr { key: "{review.id}",
                                    td { "{review.title}" }
                                    td { "{review.rating}/5" }
                                    td { "{review.order_id}" }
                                    td {
                                        if review.is_followup {
                                            span { class: "status-badge status-pending", "Follow-up" }
                                        } else {
                                            span { class: "status-badge status-active", "Original" }
                                        }
                                    }
                                    td {
                                        if !review.is_followup {
                                            button {
                                                class: "btn btn-small",
                                                onclick: {
                                                    let rid = review.id.clone();
                                                    move |_| {
                                                        fu_parent_id.set(rid.clone());
                                                        show_followup.set(true);
                                                    }
                                                },
                                                "Follow-up"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                p { "Loading reviews..." }
            }
        }
    }
}
