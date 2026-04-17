/// Status → CSS class mapping used by page components to render badges.
/// Extracted from inline RSX in pages/orders.rs, pages/cases.rs,
/// pages/submissions.rs, pages/admin.rs, pages/reviews.rs.

/// Order status badge class.
/// In the page RSX this appears as `class: "status-badge status-{order.status}"`.
/// This function adds semantic mapping for statuses that need non-identity classes.
pub fn order_status_class(status: &str) -> &'static str {
    match status {
        "pending" => "status-badge status-pending",
        "confirmed" => "status-badge status-confirmed",
        "processing" => "status-badge status-processing",
        "shipped" => "status-badge status-shipped",
        "delivered" => "status-badge status-delivered",
        "cancelled" => "status-badge status-cancelled",
        "split" => "status-badge status-split",
        "merged" => "status-badge status-merged",
        other => {
            // Fallback: use the status as the CSS modifier (matches RSX pattern)
            // This is a const-safe approximation; the RSX does `status-{status}`
            match other {
                _ => "status-badge status-unknown",
            }
        }
    }
}

/// Payment status badge class.
pub fn payment_status_class(status: &str) -> &'static str {
    match status {
        "unpaid" => "payment-unpaid",
        "paid" => "payment-paid",
        "held" => "payment-held",
        "refunded" => "payment-refunded",
        "partial_refund" => "payment-partial_refund",
        _ => "payment-unknown",
    }
}

/// Submission status badge class.
pub fn submission_status_class(status: &str) -> &'static str {
    match status {
        "draft" => "status-badge status-draft",
        "submitted" => "status-badge status-submitted",
        "in_review" => "status-badge status-in_review",
        "revision_requested" => "status-badge status-revision_requested",
        "accepted" => "status-badge status-accepted",
        "rejected" => "status-badge status-rejected",
        "published" => "status-badge status-published",
        "blocked" => "status-badge status-blocked",
        _ => "status-badge status-unknown",
    }
}

/// Case status badge class.
pub fn case_status_class(status: &str) -> &'static str {
    match status {
        "submitted" => "status-badge status-submitted",
        "in_review" => "status-badge status-in_review",
        "awaiting_evidence" => "status-badge status-awaiting_evidence",
        "arbitrated" => "status-badge status-arbitrated",
        "approved" => "status-badge status-approved",
        "denied" => "status-badge status-denied",
        "closed" => "status-badge status-closed",
        _ => "status-badge status-unknown",
    }
}

/// User active/inactive badge.
/// Mirrors admin.rs: `if user.is_active { "Active" } else { "Inactive" }`.
pub fn user_active_badge(is_active: bool) -> (&'static str, &'static str) {
    if is_active {
        ("Active", "status-badge status-active")
    } else {
        ("Inactive", "status-badge status-inactive")
    }
}

/// Review type badge.
/// Mirrors reviews.rs: follow-up vs original.
pub fn review_type_badge(is_followup: bool) -> (&'static str, &'static str) {
    if is_followup {
        ("Follow-up", "status-badge status-pending")
    } else {
        ("Original", "status-badge status-active")
    }
}

/// Flagged order badge visibility.
pub fn show_flagged_badge(is_flagged: bool) -> bool {
    is_flagged
}
