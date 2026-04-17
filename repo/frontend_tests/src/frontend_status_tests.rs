//! Tests for `frontend::status_display` — real frontend module imports.
//! Verifies CSS class mappings for order, payment, submission, case,
//! and review status badges as rendered by page components.

#[cfg(test)]
mod tests {
    use frontend::status_display::*;

    // ===== ORDER STATUS =====

    #[test]
    fn test_order_status_pending() {
        assert!(order_status_class("pending").contains("status-pending"));
    }

    #[test]
    fn test_order_status_delivered() {
        assert!(order_status_class("delivered").contains("status-delivered"));
    }

    #[test]
    fn test_order_status_cancelled() {
        assert!(order_status_class("cancelled").contains("status-cancelled"));
    }

    #[test]
    fn test_order_status_all_known_produce_distinct_classes() {
        use std::collections::HashSet;
        let statuses = ["pending", "confirmed", "processing", "shipped",
                        "delivered", "cancelled", "split", "merged"];
        let classes: HashSet<&str> = statuses.iter().map(|s| order_status_class(s)).collect();
        assert_eq!(classes.len(), statuses.len(),
            "each order status must produce a distinct CSS class");
    }

    #[test]
    fn test_order_status_unknown_fallback() {
        assert!(order_status_class("nonexistent").contains("status-unknown"));
    }

    // ===== PAYMENT STATUS =====

    #[test]
    fn test_payment_status_paid() {
        assert!(payment_status_class("paid").contains("payment-paid"));
    }

    #[test]
    fn test_payment_status_unpaid() {
        assert!(payment_status_class("unpaid").contains("payment-unpaid"));
    }

    #[test]
    fn test_payment_status_refunded() {
        assert!(payment_status_class("refunded").contains("payment-refunded"));
    }

    #[test]
    fn test_payment_status_held() {
        assert!(payment_status_class("held").contains("payment-held"));
    }

    #[test]
    fn test_payment_status_partial_refund() {
        assert!(payment_status_class("partial_refund").contains("payment-partial_refund"));
    }

    #[test]
    fn test_payment_status_unknown() {
        assert!(payment_status_class("xyz").contains("payment-unknown"));
    }

    // ===== SUBMISSION STATUS =====

    #[test]
    fn test_submission_status_draft() {
        assert!(submission_status_class("draft").contains("status-draft"));
    }

    #[test]
    fn test_submission_status_submitted() {
        assert!(submission_status_class("submitted").contains("status-submitted"));
    }

    #[test]
    fn test_submission_status_published() {
        assert!(submission_status_class("published").contains("status-published"));
    }

    #[test]
    fn test_submission_status_blocked() {
        assert!(submission_status_class("blocked").contains("status-blocked"));
    }

    #[test]
    fn test_submission_status_rejected() {
        assert!(submission_status_class("rejected").contains("status-rejected"));
    }

    #[test]
    fn test_submission_status_all_known_states() {
        let all = ["draft", "submitted", "in_review", "revision_requested",
                   "accepted", "rejected", "published", "blocked"];
        for s in &all {
            assert!(!submission_status_class(s).contains("status-unknown"),
                "known status '{}' must not map to unknown", s);
        }
    }

    // ===== CASE STATUS =====

    #[test]
    fn test_case_status_submitted() {
        assert!(case_status_class("submitted").contains("status-submitted"));
    }

    #[test]
    fn test_case_status_in_review() {
        assert!(case_status_class("in_review").contains("status-in_review"));
    }

    #[test]
    fn test_case_status_closed() {
        assert!(case_status_class("closed").contains("status-closed"));
    }

    #[test]
    fn test_case_status_all_known_states() {
        let all = ["submitted", "in_review", "awaiting_evidence",
                   "arbitrated", "approved", "denied", "closed"];
        for s in &all {
            assert!(!case_status_class(s).contains("status-unknown"),
                "known status '{}' must not map to unknown", s);
        }
    }

    #[test]
    fn test_case_status_all_distinct() {
        use std::collections::HashSet;
        let all = ["submitted", "in_review", "awaiting_evidence",
                   "arbitrated", "approved", "denied", "closed"];
        let classes: HashSet<&str> = all.iter().map(|s| case_status_class(s)).collect();
        assert_eq!(classes.len(), all.len());
    }

    // ===== USER ACTIVE BADGE =====

    #[test]
    fn test_user_active_badge() {
        let (label, class) = user_active_badge(true);
        assert_eq!(label, "Active");
        assert!(class.contains("status-active"));
    }

    #[test]
    fn test_user_inactive_badge() {
        let (label, class) = user_active_badge(false);
        assert_eq!(label, "Inactive");
        assert!(class.contains("status-inactive"));
    }

    // ===== REVIEW TYPE BADGE =====

    #[test]
    fn test_review_original_badge() {
        let (label, class) = review_type_badge(false);
        assert_eq!(label, "Original");
        assert!(class.contains("status-active"));
    }

    #[test]
    fn test_review_followup_badge() {
        let (label, class) = review_type_badge(true);
        assert_eq!(label, "Follow-up");
        assert!(class.contains("status-pending"));
    }

    // ===== FLAGGED ORDER BADGE =====

    #[test]
    fn test_show_flagged_badge_when_flagged() {
        assert!(show_flagged_badge(true));
    }

    #[test]
    fn test_hide_flagged_badge_when_not_flagged() {
        assert!(!show_flagged_badge(false));
    }
}
