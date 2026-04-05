use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::services::{api, auth};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserAddress {
    id: String,
    user_id: String,
    label: String,
    street_line1: String,
    street_line2: Option<String>,
    city: String,
    state: String,
    zip_code: String,
    is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationItem {
    id: String,
    title: String,
    message: String,
    is_read: bool,
}

#[derive(Serialize)]
struct UpdateProfile {
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
    contact_info: Option<String>,
    invoice_title: Option<String>,
}

#[derive(Serialize)]
struct UpdateNotifPrefs {
    notify_submissions: Option<bool>,
    notify_orders: Option<bool>,
    notify_reviews: Option<bool>,
    notify_cases: Option<bool>,
}

#[derive(Serialize)]
struct CreateAddress {
    label: String,
    street_line1: String,
    street_line2: Option<String>,
    city: String,
    state: String,
    zip_code: String,
    is_default: Option<bool>,
}

#[derive(Serialize)]
struct SetDefault {
    address_id: String,
}

#[component]
pub fn ProfilePage() -> Element {
    let user = auth::get_current_user();

    // Edit profile state
    let mut edit_mode = use_signal(|| false);
    let mut edit_first = use_signal(|| user.as_ref().map(|u| u.first_name.clone()).unwrap_or_default());
    let mut edit_last = use_signal(|| user.as_ref().map(|u| u.last_name.clone()).unwrap_or_default());
    let mut edit_username = use_signal(|| user.as_ref().map(|u| u.username.clone()).unwrap_or_default());
    let mut edit_contact = use_signal(String::new);
    let mut edit_invoice = use_signal(|| user.as_ref().and_then(|u| u.invoice_title.clone()).unwrap_or_default());
    let mut profile_msg = use_signal(|| Option::<String>::None);

    // Notification prefs
    let mut notif_subs = use_signal(|| user.as_ref().map(|u| u.notify_submissions).unwrap_or(true));
    let mut notif_orders = use_signal(|| user.as_ref().map(|u| u.notify_orders).unwrap_or(true));
    let mut notif_reviews = use_signal(|| user.as_ref().map(|u| u.notify_reviews).unwrap_or(true));
    let mut notif_cases = use_signal(|| user.as_ref().map(|u| u.notify_cases).unwrap_or(true));

    // Address state
    let addresses = use_resource(|| async {
        api::get::<Vec<UserAddress>>("/api/users/addresses").await.unwrap_or_default()
    });
    let mut new_label = use_signal(String::new);
    let mut new_street1 = use_signal(String::new);
    let mut new_street2 = use_signal(String::new);
    let mut new_city = use_signal(String::new);
    let mut new_state = use_signal(String::new);
    let mut new_zip = use_signal(String::new);
    let mut new_default = use_signal(|| false);
    let mut addr_msg = use_signal(|| Option::<String>::None);

    // Notifications inbox
    let notifications = use_resource(|| async {
        api::get::<Vec<NotificationItem>>("/api/users/notifications").await.unwrap_or_default()
    });

    let save_profile = move |_: FormEvent| {
        let data = UpdateProfile {
            first_name: Some(edit_first.read().clone()),
            last_name: Some(edit_last.read().clone()),
            username: Some(edit_username.read().clone()),
            contact_info: Some(edit_contact.read().clone()),
            invoice_title: Some(edit_invoice.read().clone()),
        };
        spawn(async move {
            match api::put_empty("/api/users/profile", &data).await {
                Ok(_) => { profile_msg.set(Some("Profile updated".to_string())); edit_mode.set(false); }
                Err(e) => { profile_msg.set(Some(e)); }
            }
        });
    };

    let save_notifs = move |_| {
        let data = UpdateNotifPrefs {
            notify_submissions: Some(*notif_subs.read()),
            notify_orders: Some(*notif_orders.read()),
            notify_reviews: Some(*notif_reviews.read()),
            notify_cases: Some(*notif_cases.read()),
        };
        spawn(async move {
            match api::put_empty("/api/users/notification-prefs", &data).await {
                Ok(_) => { profile_msg.set(Some("Notification preferences saved".to_string())); }
                Err(e) => { profile_msg.set(Some(e)); }
            }
        });
    };

    let add_address = move |_: FormEvent| {
        let data = CreateAddress {
            label: new_label.read().clone(),
            street_line1: new_street1.read().clone(),
            street_line2: { let s = new_street2.read().clone(); if s.is_empty() { None } else { Some(s) } },
            city: new_city.read().clone(),
            state: new_state.read().clone(),
            zip_code: new_zip.read().clone(),
            is_default: Some(*new_default.read()),
        };
        spawn(async move {
            match api::post::<UserAddress, _>("/api/users/addresses", &data).await {
                Ok(_) => { addr_msg.set(Some("Address added".to_string())); }
                Err(e) => { addr_msg.set(Some(e)); }
            }
        });
    };

    rsx! {
        div { class: "page-container",
            h2 { "My Profile" }

            if let Some(msg) = profile_msg.read().as_ref() {
                div { class: "status-badge status-active", "{msg}" }
            }

            if let Some(u) = &user {
                // --- Profile Card ---
                div { class: "profile-card",
                    if *edit_mode.read() {
                        form { onsubmit: save_profile,
                            div { class: "form-group",
                                label { "First Name" }
                                input { r#type: "text", value: "{edit_first}", oninput: move |e: FormEvent| edit_first.set(e.value()), required: true }
                            }
                            div { class: "form-group",
                                label { "Last Name" }
                                input { r#type: "text", value: "{edit_last}", oninput: move |e: FormEvent| edit_last.set(e.value()), required: true }
                            }
                            div { class: "form-group",
                                label { "Username" }
                                input { r#type: "text", value: "{edit_username}", oninput: move |e: FormEvent| edit_username.set(e.value()), required: true }
                            }
                            div { class: "form-group",
                                label { "Contact Info" }
                                textarea { value: "{edit_contact}", oninput: move |e: FormEvent| edit_contact.set(e.value()), rows: "2" }
                            }
                            div { class: "form-group",
                                label { "Invoice Title" }
                                input { r#type: "text", value: "{edit_invoice}", oninput: move |e: FormEvent| edit_invoice.set(e.value()), placeholder: "Custom invoice header" }
                            }
                            button { r#type: "submit", class: "btn btn-primary", "Save Profile" }
                            button { r#type: "button", class: "btn btn-secondary", onclick: move |_| edit_mode.set(false), "Cancel" }
                        }
                    } else {
                        div { class: "profile-field", label { "Name" } p { "{u.first_name} {u.last_name}" } }
                        div { class: "profile-field", label { "Username" } p { "{u.username}" } }
                        div { class: "profile-field", label { "Email" } p { "{u.email}" } }
                        div { class: "profile-field", label { "Role" } p { class: "role-badge", "{u.role}" } }
                        div { class: "profile-field", label { "Invoice Title" }
                            if let Some(ref inv) = u.invoice_title {
                                p { "{inv}" }
                            } else {
                                p { class: "text-light", "Not set" }
                            }
                        }
                        button { class: "btn", onclick: move |_| edit_mode.set(true), "Edit Profile" }
                    }
                }

                // --- Notification Preferences ---
                h3 { "Notification Preferences" }
                p { class: "text-light", "In-app inbox banners are active. Email and SMS are unavailable in offline mode." }
                div { class: "profile-card",
                    div { class: "form-group",
                        label { input { r#type: "checkbox", checked: *notif_subs.read(), onchange: move |_| { let v = *notif_subs.read(); notif_subs.set(!v); } } " Submissions" }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", checked: *notif_orders.read(), onchange: move |_| { let v = *notif_orders.read(); notif_orders.set(!v); } } " Orders" }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", checked: *notif_reviews.read(), onchange: move |_| { let v = *notif_reviews.read(); notif_reviews.set(!v); } } " Reviews" }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", checked: *notif_cases.read(), onchange: move |_| { let v = *notif_cases.read(); notif_cases.set(!v); } } " Cases" }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", disabled: true } " Email (unavailable offline)" }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", disabled: true } " SMS (unavailable offline)" }
                    }
                    button { class: "btn", onclick: save_notifs, "Save Preferences" }
                }

                // --- Address Book ---
                h3 { "Shipping Addresses" }
                if let Some(msg) = addr_msg.read().as_ref() {
                    div { class: "status-badge status-active", "{msg}" }
                }
                if let Some(addrs) = addresses.read().as_ref() {
                    if addrs.is_empty() {
                        p { "No addresses saved." }
                    }
                    for addr in addrs.iter() {
                        div { class: "profile-card",
                            key: "{addr.id}",
                            div { style: "display:flex;align-items:center;gap:8px;flex-wrap:wrap;",
                                strong { "{addr.label}" }
                                if addr.is_default {
                                    span { class: "status-badge status-active", "Default" }
                                }
                                if !addr.is_default {
                                    button {
                                        class: "btn btn-small",
                                        onclick: {
                                            let aid = addr.id.clone();
                                            move |_| {
                                                let aid = aid.clone();
                                                spawn(async move {
                                                    let body = serde_json::json!({ "address_id": aid });
                                                    match api::put_empty("/api/users/addresses/default", &body).await {
                                                        Ok(_) => addr_msg.set(Some("Default address updated".to_string())),
                                                        Err(e) => addr_msg.set(Some(e)),
                                                    }
                                                });
                                            }
                                        },
                                        "Set Default"
                                    }
                                }
                                button {
                                    class: "btn btn-small",
                                    onclick: {
                                        let aid = addr.id.clone();
                                        move |_| {
                                            let aid = aid.clone();
                                            spawn(async move {
                                                let url = format!("/api/users/addresses/{}", aid);
                                                match api::delete(&url).await {
                                                    Ok(_) => addr_msg.set(Some("Address removed".to_string())),
                                                    Err(e) => addr_msg.set(Some(e)),
                                                }
                                            });
                                        }
                                    },
                                    "Delete"
                                }
                            }
                            p { "{addr.street_line1}" }
                            if let Some(ref s2) = addr.street_line2 {
                                p { "{s2}" }
                            }
                            p { "{addr.city}, {addr.state} {addr.zip_code}" }
                        }
                    }
                }

                h4 { "Add New Address" }
                form { onsubmit: add_address,
                    div { class: "form-group",
                        label { "Label" }
                        input { r#type: "text", value: "{new_label}", oninput: move |e: FormEvent| new_label.set(e.value()), required: true, placeholder: "Home, Office, etc." }
                    }
                    div { class: "form-group",
                        label { "Street Line 1" }
                        input { r#type: "text", value: "{new_street1}", oninput: move |e: FormEvent| new_street1.set(e.value()), required: true }
                    }
                    div { class: "form-group",
                        label { "Street Line 2" }
                        input { r#type: "text", value: "{new_street2}", oninput: move |e: FormEvent| new_street2.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "City" }
                        input { r#type: "text", value: "{new_city}", oninput: move |e: FormEvent| new_city.set(e.value()), required: true }
                    }
                    div { class: "form-group",
                        label { "State (2-letter)" }
                        input { r#type: "text", value: "{new_state}", oninput: move |e: FormEvent| new_state.set(e.value()), required: true, maxlength: "2" }
                    }
                    div { class: "form-group",
                        label { "ZIP Code" }
                        input { r#type: "text", value: "{new_zip}", oninput: move |e: FormEvent| new_zip.set(e.value()), required: true }
                    }
                    div { class: "form-group",
                        label { input { r#type: "checkbox", checked: *new_default.read(), onchange: move |_| { let v = *new_default.read(); new_default.set(!v); } } " Set as default" }
                    }
                    button { r#type: "submit", class: "btn btn-primary", "Add Address" }
                }

                // --- Notifications Inbox ---
                h3 { "Notifications" }
                if let Some(notifs) = notifications.read().as_ref() {
                    if notifs.is_empty() {
                        p { "No notifications." }
                    }
                    for notif in notifs.iter() {
                        div { class: "profile-card",
                            key: "{notif.id}",
                            if !notif.is_read {
                                span { class: "status-badge status-pending", "New" }
                            }
                            strong { " {notif.title}" }
                            p { "{notif.message}" }
                        }
                    }
                }
            } else {
                p { "Please sign in to view your profile." }
            }
        }
    }
}
