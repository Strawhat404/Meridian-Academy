mod components;
mod pages;
mod services;

use dioxus::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::launch(App);
}

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/admin/provision")]
    AdminProvision {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/submissions")]
    Submissions {},
    #[route("/submissions/new")]
    NewSubmission {},
    #[route("/orders")]
    Orders {},
    #[route("/orders/new")]
    NewOrder {},
    #[route("/reviews")]
    Reviews {},
    #[route("/cases")]
    Cases {},
    #[route("/cases/new")]
    NewCase {},
    #[route("/admin")]
    Admin {},
    #[route("/admin/users")]
    AdminUsers {},
    #[route("/profile")]
    Profile {},
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let auth = services::auth::get_current_user();
    let is_logged_in = auth.is_some();

    rsx! {
        components::layout::Layout {
            div { class: "home-container",
                h1 { "Meridian Academic Publishing & Fulfillment Portal" }
                p { class: "subtitle", "Your comprehensive platform for academic content submissions, peer reviews, publication orders, and case management." }

                if is_logged_in {
                    div { class: "cta-buttons",
                        Link { to: Route::Dashboard {}, class: "btn btn-primary", "Go to Dashboard" }
                    }
                } else {
                    div { class: "cta-buttons",
                        Link { to: Route::Login {}, class: "btn btn-primary", "Sign In" }
                    }
                }

                div { class: "features-grid",
                    div { class: "feature-card",
                        h3 { "Content Submissions" }
                        p { "Submit journal articles, conference papers, theses, and book chapters for peer review and publication." }
                    }
                    div { class: "feature-card",
                        h3 { "Peer Review" }
                        p { "Participate in the peer review process with structured ratings, comments, and recommendations." }
                    }
                    div { class: "feature-card",
                        h3 { "Publication Orders" }
                        p { "Order physical copies of journals and publications with full fulfillment tracking." }
                    }
                    div { class: "feature-card",
                        h3 { "Case Management" }
                        p { "Report and track issues with orders through our after-sales support system." }
                    }
                }
            }
        }
    }
}

#[component]
fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let onsubmit = move |_: FormEvent| {
        let nav = nav.clone();
        let username_val = username.read().clone();
        let password_val = password.read().clone();
        spawn(async move {
            match services::auth::login(&username_val, &password_val).await {
                Ok(_) => { nav.push(Route::Dashboard {}); },
                Err(e) => { error.set(Some(e)); },
            }
        });
    };

    rsx! {
        components::layout::Layout {
            div { class: "auth-container",
                h2 { "Sign In" }
                if let Some(err) = error.read().as_ref() {
                    div { class: "error-message", "{err}" }
                }
                form { onsubmit,
                    div { class: "form-group",
                        label { r#for: "username", "Username" }
                        input {
                            r#type: "text",
                            id: "username",
                            value: "{username}",
                            oninput: move |e: FormEvent| username.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "password", "Password" }
                        input {
                            r#type: "password",
                            id: "password",
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                            required: true
                        }
                    }
                    button { r#type: "submit", class: "btn btn-primary", "Sign In" }
                }
                p { "Don't have an account? ",
                    "Contact your administrator for an account."
                }
            }
        }
    }
}

#[component]
fn AdminProvision() -> Element {
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut role = use_signal(|| "student".to_string());
    let mut error = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let onsubmit = move |_: FormEvent| {
        let nav = nav.clone();
        let data = services::auth::RegisterData {
            username: username.read().clone(),
            email: email.read().clone(),
            password: password.read().clone(),
            first_name: first_name.read().clone(),
            last_name: last_name.read().clone(),
            role: role.read().clone(),
        };
        spawn(async move {
            match services::auth::register(&data).await {
                Ok(_) => { nav.push(Route::Dashboard {}); },
                Err(e) => { error.set(Some(e)); },
            }
        });
    };

    rsx! {
        components::layout::Layout {
            div { class: "auth-container",
                h2 { "Create Account" }
                if let Some(err) = error.read().as_ref() {
                    div { class: "error-message", "{err}" }
                }
                form { onsubmit,
                    div { class: "form-group",
                        label { r#for: "username", "Username" }
                        input {
                            r#type: "text",
                            id: "username",
                            value: "{username}",
                            oninput: move |e: FormEvent| username.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "first_name", "First Name" }
                        input {
                            r#type: "text",
                            id: "first_name",
                            value: "{first_name}",
                            oninput: move |e: FormEvent| first_name.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "last_name", "Last Name" }
                        input {
                            r#type: "text",
                            id: "last_name",
                            value: "{last_name}",
                            oninput: move |e: FormEvent| last_name.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "email", "Email" }
                        input {
                            r#type: "email",
                            id: "email",
                            value: "{email}",
                            oninput: move |e: FormEvent| email.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "password", "Password" }
                        input {
                            r#type: "password",
                            id: "password",
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "role", "Role" }
                        select {
                            id: "role",
                            value: "{role}",
                            onchange: move |e: FormEvent| role.set(e.value()),
                            option { value: "student", "Student" }
                            option { value: "instructor", "Instructor" }
                            option { value: "academic_staff", "Academic Staff" }
                        }
                    }
                    button { r#type: "submit", class: "btn btn-primary", "Register" }
                }
                p { "Already have an account? ",
                    Link { to: Route::Login {}, "Sign in" }
                }
            }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    let user = services::auth::get_current_user();

    rsx! {
        components::layout::Layout {
            div { class: "dashboard-container",
                if let Some(u) = user {
                    h2 { "Welcome, {u.first_name} {u.last_name}" }
                    p { class: "role-badge", "Role: {u.role}" }

                    div { class: "dashboard-grid",
                        if u.role == "student" || u.role == "instructor" {
                            div { class: "dashboard-card",
                                h3 { "My Submissions" }
                                p { "View and manage your academic content submissions." }
                                Link { to: Route::Submissions {}, class: "btn", "View Submissions" }
                            }
                        }
                        div { class: "dashboard-card",
                            h3 { "Orders" }
                            p { "Order physical copies of publications." }
                            Link { to: Route::Orders {}, class: "btn", "View Orders" }
                        }
                        if u.role == "instructor" || u.role == "academic_staff" {
                            div { class: "dashboard-card",
                                h3 { "Peer Reviews" }
                                p { "Review submitted academic content." }
                                Link { to: Route::Reviews {}, class: "btn", "View Reviews" }
                            }
                        }
                        div { class: "dashboard-card",
                            h3 { "Support Cases" }
                            p { "Manage after-sales support cases." }
                            Link { to: Route::Cases {}, class: "btn", "View Cases" }
                        }
                        if u.role == "administrator" {
                            div { class: "dashboard-card",
                                h3 { "Administration" }
                                p { "System administration and user management." }
                                Link { to: Route::Admin {}, class: "btn", "Admin Panel" }
                            }
                        }
                    }
                } else {
                    h2 { "Please sign in to access your dashboard." }
                    Link { to: Route::Login {}, class: "btn btn-primary", "Sign In" }
                }
            }
        }
    }
}

#[component]
fn Submissions() -> Element {
    rsx! {
        components::layout::Layout {
            pages::submissions::SubmissionsPage {}
        }
    }
}

#[component]
fn NewSubmission() -> Element {
    rsx! {
        components::layout::Layout {
            pages::submissions::NewSubmissionPage {}
        }
    }
}

#[component]
fn Orders() -> Element {
    rsx! {
        components::layout::Layout {
            pages::orders::OrdersPage {}
        }
    }
}

#[component]
fn NewOrder() -> Element {
    rsx! {
        components::layout::Layout {
            pages::orders::NewOrderPage {}
        }
    }
}

#[component]
fn Reviews() -> Element {
    rsx! {
        components::layout::Layout {
            pages::reviews::ReviewsPage {}
        }
    }
}

#[component]
fn Cases() -> Element {
    rsx! {
        components::layout::Layout {
            pages::cases::CasesPage {}
        }
    }
}

#[component]
fn NewCase() -> Element {
    rsx! {
        components::layout::Layout {
            pages::cases::NewCasePage {}
        }
    }
}

#[component]
fn Admin() -> Element {
    rsx! {
        components::layout::Layout {
            pages::admin::AdminPage {}
        }
    }
}

#[component]
fn AdminUsers() -> Element {
    rsx! {
        components::layout::Layout {
            pages::admin::AdminUsersPage {}
        }
    }
}

#[component]
fn Profile() -> Element {
    rsx! {
        components::layout::Layout {
            pages::profile::ProfilePage {}
        }
    }
}
