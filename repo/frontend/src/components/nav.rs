use dioxus::prelude::*;
use crate::Route;
use crate::services::auth;

#[component]
pub fn NavBar() -> Element {
    let user = auth::get_current_user();
    let is_logged_in = user.is_some();

    rsx! {
        nav { class: "navbar",
            div { class: "nav-brand",
                Link { to: Route::Home {}, "Meridian Academy" }
            }
            div { class: "nav-links",
                if let Some(u) = user {
                    Link { to: Route::Dashboard {}, "Dashboard" }
                    if u.role == "student" || u.role == "instructor" {
                        Link { to: Route::Submissions {}, "Submissions" }
                    }
                    Link { to: Route::Orders {}, "Orders" }
                    Link { to: Route::Reviews {}, "Reviews" }
                    Link { to: Route::Cases {}, "Cases" }
                    if u.role == "administrator" || u.role == "academic_staff" {
                        Link { to: Route::Admin {}, "Admin" }
                    }
                    Link { to: Route::Profile {}, "{u.first_name}" }
                    button {
                        class: "btn btn-logout",
                        onclick: move |_| {
                            spawn(async move {
                                auth::logout().await;
                                web_sys::window()
                                    .and_then(|w| w.location().set_href("/login").ok());
                            });
                        },
                        "Sign Out"
                    }
                }
                if !is_logged_in {
                    Link { to: Route::Login {}, "Sign In" }
                    // No public registration — admin-provisioned only
                }
            }
        }
    }
}
