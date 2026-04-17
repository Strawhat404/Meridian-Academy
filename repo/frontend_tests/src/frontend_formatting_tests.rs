//! Tests for `frontend::formatting` — real frontend module imports.
//! These exercise the actual formatting/display helpers used by page components.

#[cfg(test)]
mod tests {
    use frontend::formatting::*;

    // ===== CURRENCY =====

    #[test]
    fn test_format_currency_whole() {
        assert_eq!(format_currency(10.0), "$10.00");
    }

    #[test]
    fn test_format_currency_cents() {
        assert_eq!(format_currency(29.99), "$29.99");
    }

    #[test]
    fn test_format_currency_zero() {
        assert_eq!(format_currency(0.0), "$0.00");
    }

    #[test]
    fn test_format_currency_rounding() {
        assert_eq!(format_currency(10.999), "$11.00");
    }

    // ===== FILE SIZE =====

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn test_format_file_size_kb() {
        assert_eq!(format_file_size(1024), "1.0 KB");
    }

    #[test]
    fn test_format_file_size_mb() {
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_format_file_size_25mb() {
        assert_eq!(format_file_size(25 * 1024 * 1024), "25.0 MB");
    }

    // ===== RECONCILIATION BADGE =====

    #[test]
    fn test_reconciliation_badge_matched() {
        assert!(reconciliation_badge_class("matched").contains("status-active"));
    }

    #[test]
    fn test_reconciliation_badge_discrepancy() {
        assert!(reconciliation_badge_class("discrepancy").contains("status-rejected"));
    }

    #[test]
    fn test_reconciliation_badge_pending() {
        assert!(reconciliation_badge_class("pending").contains("status-pending"));
    }

    #[test]
    fn test_reconciliation_badge_unknown_defaults_to_pending() {
        assert!(reconciliation_badge_class("xyz").contains("status-pending"));
    }

    // ===== SLA DISPLAY =====

    #[test]
    fn test_sla_display_on_track() {
        assert_eq!(sla_display(false, false), "On Track");
    }

    #[test]
    fn test_sla_display_response_overdue() {
        assert_eq!(sla_display(true, false), "Response Overdue");
    }

    #[test]
    fn test_sla_display_resolution_overdue() {
        assert_eq!(sla_display(false, true), "Resolution Overdue");
    }

    #[test]
    fn test_sla_display_both_overdue_response_takes_priority() {
        assert_eq!(sla_display(true, true), "Response Overdue");
    }

    // ===== CASE TRANSITIONS =====

    #[test]
    fn test_case_transitions_submitted() {
        let t = available_case_transitions("submitted");
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].0, "in_review");
    }

    #[test]
    fn test_case_transitions_in_review() {
        let t = available_case_transitions("in_review");
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_case_transitions_arbitrated() {
        let t = available_case_transitions("arbitrated");
        assert_eq!(t.len(), 2);
        let targets: Vec<&str> = t.iter().map(|(s, _)| *s).collect();
        assert!(targets.contains(&"approved"));
        assert!(targets.contains(&"denied"));
    }

    #[test]
    fn test_case_transitions_closed_empty() {
        assert!(available_case_transitions("closed").is_empty());
    }

    #[test]
    fn test_case_transitions_match_backend_valid_transitions() {
        use backend::models;
        let all = ["submitted", "in_review", "awaiting_evidence",
                   "arbitrated", "approved", "denied", "closed"];
        for status in &all {
            for (target, _) in available_case_transitions(status) {
                assert!(models::valid_case_transition(status, target),
                    "Frontend transition '{}' -> '{}' must be valid in backend",
                    status, target);
            }
        }
    }

    // ===== STARS =====

    #[test]
    fn test_stars_five() {
        assert_eq!(stars_for_rating(5), "★★★★★");
    }

    #[test]
    fn test_stars_zero() {
        assert_eq!(stars_for_rating(0), "☆☆☆☆☆");
    }

    #[test]
    fn test_stars_three() {
        assert_eq!(stars_for_rating(3), "★★★☆☆");
    }

    // ===== SUBSCRIPTION LABEL =====

    #[test]
    fn test_subscription_labels() {
        assert_eq!(subscription_label("monthly"), "Monthly");
        assert_eq!(subscription_label("quarterly"), "Quarterly");
        assert_eq!(subscription_label("annual"), "Annual");
        assert_eq!(subscription_label("unknown"), "Custom");
    }

    // ===== FULFILLMENT EVENT LABELS =====

    #[test]
    fn test_fulfillment_labels_all_known_types() {
        let types = ["missing_issue", "reshipment", "delay",
                     "discontinuation", "edition_change", "delivered"];
        for t in &types {
            assert_ne!(fulfillment_event_label(t), "Unknown Event",
                "missing label for '{}'", t);
        }
    }

    #[test]
    fn test_fulfillment_label_unknown_type() {
        assert_eq!(fulfillment_event_label("zzz"), "Unknown Event");
    }

    // ===== PRIORITY RANK =====

    #[test]
    fn test_priority_ordering() {
        assert!(priority_rank("urgent") < priority_rank("high"));
        assert!(priority_rank("high") < priority_rank("medium"));
        assert!(priority_rank("medium") < priority_rank("low"));
    }

    // ===== ORDER TOTAL =====

    #[test]
    fn test_calculate_order_total_single() {
        assert!((calculate_order_total(&[(3, 9.99)]) - 29.97).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_order_total_multiple() {
        let total = calculate_order_total(&[(2, 10.0), (1, 5.0)]);
        assert!((total - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_order_total_empty() {
        assert_eq!(calculate_order_total(&[]), 0.0);
    }

    // ===== DRAFT PROGRESS =====

    #[test]
    fn test_draft_progress_text() {
        assert_eq!(draft_progress(1, 10), "Version 1 of 10 max");
        assert_eq!(draft_progress(5, 10), "Version 5 of 10 max");
    }
}
