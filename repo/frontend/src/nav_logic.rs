/// Navigation visibility logic extracted from components/nav.rs.
/// Determines which nav items are shown based on user role.

/// Whether the Submissions link is visible for this role.
/// Mirrors nav.rs: `if u.role == "student" || u.role == "instructor"`
pub fn show_submissions(role: &str) -> bool {
    role == "student" || role == "instructor"
}

/// Whether the Admin link is visible for this role.
/// Mirrors nav.rs: `if u.role == "administrator" || u.role == "academic_staff"`
pub fn show_admin(role: &str) -> bool {
    role == "administrator" || role == "academic_staff"
}

/// Whether the user has staff-level visibility (sees all orders, can manage).
pub fn is_staff(role: &str) -> bool {
    role == "administrator" || role == "academic_staff"
}

/// Returns the correct orders API path based on role.
/// Staff/admin see all orders; others see only their own.
pub fn orders_api_path(role: &str) -> &'static str {
    if is_staff(role) {
        "/api/orders"
    } else {
        "/api/orders/my"
    }
}

/// Returns the correct cases API path based on role.
pub fn cases_api_path(role: &str) -> &'static str {
    if is_staff(role) {
        "/api/cases"
    } else {
        "/api/cases/my"
    }
}

/// Returns the correct submissions API path based on role.
pub fn submissions_api_path(role: &str) -> &'static str {
    if is_staff(role) {
        "/api/submissions"
    } else {
        "/api/submissions/my"
    }
}

/// Full list of nav menu items for a given role.
pub fn menu_items(role: &str) -> Vec<&'static str> {
    let mut items = vec!["Dashboard"];

    if show_submissions(role) {
        items.push("Submissions");
    }

    items.push("Orders");
    items.push("Reviews");
    items.push("Cases");

    if show_admin(role) {
        items.push("Admin");
    }

    items.push("Profile");
    items
}
