/// Display formatting helpers used by frontend page components.
/// Pure functions — no browser APIs, testable from native targets.

/// Format a monetary amount as "$X.XX".
pub fn format_currency(amount: f64) -> String {
    format!("${:.2}", amount)
}

/// Format file size for human display.
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Reconciliation badge CSS class — mirrors the inline logic in orders.rs page.
pub fn reconciliation_badge_class(status: &str) -> &'static str {
    match status {
        "matched" => "status-badge status-active",
        "discrepancy" => "status-badge status-rejected",
        _ => "status-badge status-pending",
    }
}

/// SLA display label — mirrors cases.rs and admin.rs page logic.
pub fn sla_display(first_response_overdue: bool, resolution_overdue: bool) -> &'static str {
    if first_response_overdue {
        "Response Overdue"
    } else if resolution_overdue {
        "Resolution Overdue"
    } else {
        "On Track"
    }
}

/// Case transition buttons available for a given status.
/// Mirrors the `match cws.case.status.as_str()` block in admin.rs:407 and cases.rs:283.
pub fn available_case_transitions(status: &str) -> Vec<(&'static str, &'static str)> {
    match status {
        "submitted" => vec![("in_review", "Start Review")],
        "in_review" => vec![
            ("awaiting_evidence", "Request Evidence"),
            ("arbitrated", "Arbitrate"),
        ],
        "awaiting_evidence" => vec![
            ("in_review", "Resume Review"),
            ("arbitrated", "Arbitrate"),
        ],
        "arbitrated" => vec![("approved", "Approve"), ("denied", "Deny")],
        "approved" | "denied" => vec![("closed", "Close")],
        "closed" => vec![],
        _ => vec![],
    }
}

/// Render stars for a review rating (1-5).
pub fn stars_for_rating(rating: i32) -> String {
    let filled = rating.max(0).min(5) as usize;
    let empty = 5 - filled;
    format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
}

/// Subscription period human label.
pub fn subscription_label(period: &str) -> &'static str {
    match period {
        "monthly" => "Monthly",
        "quarterly" => "Quarterly",
        "annual" => "Annual",
        _ => "Custom",
    }
}

/// Fulfillment event type → human label.
pub fn fulfillment_event_label(event_type: &str) -> &'static str {
    match event_type {
        "missing_issue" => "Missing Issue",
        "reshipment" => "Reshipment",
        "delay" => "Delayed",
        "discontinuation" => "Discontinued",
        "edition_change" => "Edition Change",
        "delivered" => "Delivered",
        _ => "Unknown Event",
    }
}

/// Case priority display ordering (lower = more urgent).
pub fn priority_rank(priority: &str) -> u32 {
    match priority {
        "urgent" => 1,
        "high" => 2,
        "medium" => 3,
        "low" => 4,
        _ => 999,
    }
}

/// Calculate order total from line items.
pub fn calculate_order_total(items: &[(i32, f64)]) -> f64 {
    items.iter().map(|(qty, price)| *qty as f64 * price).sum()
}

/// Draft progress indicator text.
pub fn draft_progress(current_version: i32, max_versions: i32) -> String {
    format!("Version {} of {} max", current_version, max_versions)
}
