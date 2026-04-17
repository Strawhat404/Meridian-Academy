/// Pure-logic modules that do NOT depend on browser APIs (dioxus, gloo, web-sys).
/// These are importable from native-target test crates via:
///   `frontend = { path = "../frontend", default-features = false }`
///
/// Browser-only modules (components, pages, services) remain in main.rs's
/// module tree and are only compiled when the "web" feature is active.

pub mod validation;
pub mod formatting;
pub mod nav_logic;
pub mod status_display;
