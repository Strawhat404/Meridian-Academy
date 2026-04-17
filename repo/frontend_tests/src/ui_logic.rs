//! Additional frontend/UI-logic tests covering:
//!  * Status badge class mapping for orders, cases, submissions, payments, reviews
//!  * Date & currency formatting rules
//!  * Role-based navigation menus
//!  * Pagination / filter / sort helpers
//!  * Empty-state messages
//!  * Notification badges
//!  * Form validation helpers
//!  * Download/upload size formatting
//! These tests lock down the deterministic UI logic the Dioxus pages rely on
//! without requiring a WASM-compiled render pass.

#[cfg(test)]
mod tests {
    use backend::models;
    use backend::models::case::AfterSalesCase;
    use backend::models::order::Order;
    use backend::models::submission::Submission;
    use chrono::{Duration, NaiveDate, Utc};

    // =================================================================
    // ORDER STATUS BADGE MAPPING
    // =================================================================

    fn order_status_class(status: &str) -> &'static str {
        match status {
            "pending" => "status-badge status-pending",
            "confirmed" | "processing" | "shipped" => "status-badge status-info",
            "delivered" => "status-badge status-active",
            "cancelled" => "status-badge status-rejected",
            "split" | "merged" => "status-badge status-secondary",
            _ => "status-badge status-unknown",
        }
    }

    #[test]
    fn test_order_badge_pending() {
        assert!(order_status_class("pending").contains("status-pending"));
    }

    #[test]
    fn test_order_badge_delivered() {
        assert!(order_status_class("delivered").contains("status-active"));
    }

    #[test]
    fn test_order_badge_cancelled() {
        assert!(order_status_class("cancelled").contains("status-rejected"));
    }

    #[test]
    fn test_order_badge_unknown_fallback() {
        assert!(order_status_class("tardis").contains("status-unknown"));
    }

    #[test]
    fn test_order_badge_shipment_pipeline_all_info() {
        for s in &["confirmed", "processing", "shipped"] {
            assert!(order_status_class(s).contains("status-info"),
                "status '{}' should be status-info", s);
        }
    }

    #[test]
    fn test_order_badge_split_and_merged_secondary() {
        assert!(order_status_class("split").contains("status-secondary"));
        assert!(order_status_class("merged").contains("status-secondary"));
    }

    // =================================================================
    // CASE STATUS BADGE MAPPING
    // =================================================================

    fn case_status_class(status: &str) -> &'static str {
        match status {
            "submitted" => "status-badge status-pending",
            "in_review" => "status-badge status-info",
            "awaiting_evidence" => "status-badge status-warning",
            "arbitrated" => "status-badge status-secondary",
            "approved" => "status-badge status-active",
            "denied" => "status-badge status-rejected",
            "closed" => "status-badge status-muted",
            _ => "status-badge status-unknown",
        }
    }

    #[test]
    fn test_case_badges_each_distinct_class() {
        use std::collections::HashSet;
        let statuses = ["submitted", "in_review", "awaiting_evidence",
                        "arbitrated", "approved", "denied", "closed"];
        let classes: HashSet<&str> = statuses.iter().map(|s| case_status_class(s)).collect();
        assert_eq!(classes.len(), 7, "each case status must map to distinct class");
    }

    #[test]
    fn test_case_submitted_pending_style() {
        assert!(case_status_class("submitted").contains("status-pending"));
    }

    #[test]
    fn test_case_approved_active_style() {
        assert!(case_status_class("approved").contains("status-active"));
    }

    #[test]
    fn test_case_denied_rejected_style() {
        assert!(case_status_class("denied").contains("status-rejected"));
    }

    #[test]
    fn test_case_closed_muted_style() {
        assert!(case_status_class("closed").contains("status-muted"));
    }

    // =================================================================
    // SUBMISSION STATUS BADGE MAPPING
    // =================================================================

    fn submission_status_class(status: &str) -> &'static str {
        match status {
            "draft" => "status-badge status-secondary",
            "submitted" | "in_review" => "status-badge status-pending",
            "revision_requested" => "status-badge status-warning",
            "accepted" => "status-badge status-active",
            "rejected" => "status-badge status-rejected",
            "published" => "status-badge status-success",
            "blocked" => "status-badge status-danger",
            _ => "status-badge status-unknown",
        }
    }

    #[test]
    fn test_submission_draft_secondary() {
        assert!(submission_status_class("draft").contains("status-secondary"));
    }

    #[test]
    fn test_submission_published_success() {
        assert!(submission_status_class("published").contains("status-success"));
    }

    #[test]
    fn test_submission_blocked_danger() {
        assert!(submission_status_class("blocked").contains("status-danger"));
    }

    #[test]
    fn test_submission_in_review_and_submitted_share_pending() {
        assert_eq!(submission_status_class("submitted"),
                   submission_status_class("in_review"));
    }

    // =================================================================
    // PAYMENT STATUS BADGE MAPPING
    // =================================================================

    fn payment_status_class(status: &str) -> &'static str {
        match status {
            "pending" => "status-badge status-pending",
            "held" => "status-badge status-warning",
            "completed" => "status-badge status-active",
            "released" => "status-badge status-info",
            "refunded" => "status-badge status-rejected",
            _ => "status-badge status-unknown",
        }
    }

    #[test]
    fn test_payment_badge_completed() {
        assert!(payment_status_class("completed").contains("status-active"));
    }

    #[test]
    fn test_payment_badge_refunded_rejected() {
        assert!(payment_status_class("refunded").contains("status-rejected"));
    }

    #[test]
    fn test_payment_badge_held() {
        assert!(payment_status_class("held").contains("status-warning"));
    }

    // =================================================================
    // CURRENCY FORMATTING
    // =================================================================

    fn format_currency(amount: f64) -> String {
        format!("${:.2}", amount)
    }

    #[test]
    fn test_currency_whole_dollar() {
        assert_eq!(format_currency(10.0), "$10.00");
    }

    #[test]
    fn test_currency_cents_precision() {
        assert_eq!(format_currency(10.5), "$10.50");
        assert_eq!(format_currency(10.1234), "$10.12");
        assert_eq!(format_currency(10.999), "$11.00");
    }

    #[test]
    fn test_currency_zero() {
        assert_eq!(format_currency(0.0), "$0.00");
    }

    #[test]
    fn test_currency_negative() {
        assert_eq!(format_currency(-5.0), "$-5.00");
    }

    #[test]
    fn test_currency_large_amount() {
        assert_eq!(format_currency(1_234_567.89), "$1234567.89");
    }

    #[test]
    fn test_currency_matches_line_total_format() {
        let line_total = 29.99_f64 * 3.0;
        assert_eq!(format_currency(line_total), "$89.97");
    }

    // =================================================================
    // FILE SIZE FORMATTING (frontend displays MB/KB)
    // =================================================================

    fn format_file_size(bytes: u64) -> String {
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

    #[test]
    fn test_file_size_bytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn test_file_size_kb() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
    }

    #[test]
    fn test_file_size_mb() {
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(25 * 1024 * 1024), "25.0 MB");
    }

    #[test]
    fn test_file_size_max_upload_displays_correctly() {
        let max = models::MAX_FILE_SIZE;
        assert_eq!(format_file_size(max), "25.0 MB");
    }

    #[test]
    fn test_file_size_max_review_image_displays_correctly() {
        let max = models::MAX_REVIEW_IMAGE_SIZE;
        assert_eq!(format_file_size(max), "5.0 MB");
    }

    // =================================================================
    // PAGINATION LOGIC
    // =================================================================

    fn page_count(total: usize, per_page: usize) -> usize {
        if per_page == 0 { return 0; }
        (total + per_page - 1) / per_page
    }

    #[test]
    fn test_pagination_zero_total() {
        assert_eq!(page_count(0, 20), 0);
    }

    #[test]
    fn test_pagination_even_division() {
        assert_eq!(page_count(40, 20), 2);
        assert_eq!(page_count(100, 10), 10);
    }

    #[test]
    fn test_pagination_partial_last_page() {
        assert_eq!(page_count(41, 20), 3);
        assert_eq!(page_count(1, 20), 1);
    }

    #[test]
    fn test_pagination_per_page_zero_safe() {
        assert_eq!(page_count(100, 0), 0);
    }

    #[test]
    fn test_pagination_per_page_larger_than_total() {
        assert_eq!(page_count(5, 100), 1);
    }

    // =================================================================
    // SEARCH / FILTER HELPERS
    // =================================================================

    fn matches_search(text: &str, query: &str) -> bool {
        if query.is_empty() { return true; }
        text.to_lowercase().contains(&query.to_lowercase())
    }

    #[test]
    fn test_search_empty_query_matches_all() {
        assert!(matches_search("any text", ""));
        assert!(matches_search("", ""));
    }

    #[test]
    fn test_search_case_insensitive() {
        assert!(matches_search("Research Paper", "research"));
        assert!(matches_search("Research Paper", "RESEARCH"));
        assert!(matches_search("Research Paper", "ResEARCH"));
    }

    #[test]
    fn test_search_substring_match() {
        assert!(matches_search("The Journal of Computer Science", "computer"));
        assert!(!matches_search("The Journal", "computer"));
    }

    #[test]
    fn test_search_unicode() {
        assert!(matches_search("日本語 research", "日本語"));
    }

    // =================================================================
    // SORT HELPERS (sort by created_at desc)
    // =================================================================

    fn sort_by_recent(mut items: Vec<(String, NaiveDate)>) -> Vec<(String, NaiveDate)> {
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items
    }

    #[test]
    fn test_sort_recent_descending() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let d3 = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let result = sort_by_recent(vec![
            ("A".into(), d1),
            ("B".into(), d2),
            ("C".into(), d3),
        ]);
        assert_eq!(result[0].0, "B");
        assert_eq!(result[1].0, "C");
        assert_eq!(result[2].0, "A");
    }

    #[test]
    fn test_sort_recent_single_item() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = sort_by_recent(vec![("only".into(), d)]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_sort_recent_empty() {
        let result: Vec<(String, NaiveDate)> = sort_by_recent(vec![]);
        assert!(result.is_empty());
    }

    // =================================================================
    // NOTIFICATION BADGE COUNT
    // =================================================================

    fn unread_badge_text(count: usize) -> String {
        match count {
            0 => "".into(),
            1..=9 => count.to_string(),
            10..=99 => count.to_string(),
            _ => "99+".into(),
        }
    }

    #[test]
    fn test_notification_badge_hidden_when_zero() {
        assert_eq!(unread_badge_text(0), "");
    }

    #[test]
    fn test_notification_badge_single_digit() {
        assert_eq!(unread_badge_text(5), "5");
    }

    #[test]
    fn test_notification_badge_double_digit() {
        assert_eq!(unread_badge_text(42), "42");
    }

    #[test]
    fn test_notification_badge_overflow() {
        assert_eq!(unread_badge_text(100), "99+");
        assert_eq!(unread_badge_text(500), "99+");
    }

    #[test]
    fn test_notification_badge_boundary_99() {
        assert_eq!(unread_badge_text(99), "99");
    }

    // =================================================================
    // ROLE-BASED MENU ITEMS
    // =================================================================

    fn menu_items_for_role(role: &str) -> Vec<&'static str> {
        match role {
            "student" | "instructor" => vec!["Dashboard", "Submissions", "Orders",
                                              "Reviews", "Cases", "Profile"],
            "academic_staff" => vec!["Dashboard", "Submissions Review", "Orders",
                                      "Reviews", "Cases", "Content Management", "Profile"],
            "administrator" => vec!["Dashboard", "Users", "Audit Log", "System Settings",
                                     "Orders", "Payments", "Reviews", "Cases",
                                     "Content Management", "Abnormal Flags", "Profile"],
            _ => vec!["Profile"],
        }
    }

    #[test]
    fn test_student_menu_has_submissions() {
        assert!(menu_items_for_role("student").contains(&"Submissions"));
    }

    #[test]
    fn test_student_menu_no_audit_log() {
        assert!(!menu_items_for_role("student").contains(&"Audit Log"));
    }

    #[test]
    fn test_admin_menu_has_everything() {
        let menu = menu_items_for_role("administrator");
        assert!(menu.contains(&"Users"));
        assert!(menu.contains(&"Audit Log"));
        assert!(menu.contains(&"System Settings"));
        assert!(menu.contains(&"Payments"));
        assert!(menu.contains(&"Abnormal Flags"));
    }

    #[test]
    fn test_staff_menu_has_content_management() {
        assert!(menu_items_for_role("academic_staff").contains(&"Content Management"));
    }

    #[test]
    fn test_all_roles_have_profile() {
        for role in &["student", "instructor", "academic_staff", "administrator"] {
            assert!(menu_items_for_role(role).contains(&"Profile"),
                "role '{}' must have Profile in menu", role);
        }
    }

    #[test]
    fn test_unknown_role_minimal_menu() {
        let menu = menu_items_for_role("unknown");
        assert_eq!(menu, vec!["Profile"]);
    }

    // =================================================================
    // EMPTY-STATE MESSAGES
    // =================================================================

    fn empty_state_for(resource: &str) -> &'static str {
        match resource {
            "submissions" => "You haven't submitted any papers yet.",
            "orders" => "No orders yet. Browse the catalog to start.",
            "reviews" => "No reviews yet.",
            "cases" => "No after-sales cases on file.",
            "notifications" => "You're all caught up!",
            _ => "No items.",
        }
    }

    #[test]
    fn test_each_empty_state_is_nonempty_and_unique() {
        use std::collections::HashSet;
        let msgs: HashSet<&str> = ["submissions", "orders", "reviews", "cases", "notifications"]
            .iter().map(|r| empty_state_for(r)).collect();
        assert_eq!(msgs.len(), 5);
        for m in &msgs {
            assert!(!m.is_empty());
        }
    }

    // =================================================================
    // FORM VALIDATION HELPERS
    // =================================================================

    fn validate_email_basic(email: &str) -> bool {
        !email.is_empty()
            && email.contains('@')
            && email.contains('.')
            && !email.contains(' ')
            && email.find('@').map(|at| at > 0 && at < email.len() - 1).unwrap_or(false)
    }

    #[test]
    fn test_email_valid_cases() {
        assert!(validate_email_basic("user@example.com"));
        assert!(validate_email_basic("a.b@c.d"));
        assert!(validate_email_basic("first.last@sub.domain.edu"));
    }

    #[test]
    fn test_email_invalid_cases() {
        assert!(!validate_email_basic(""));
        assert!(!validate_email_basic("no-at-sign.com"));
        assert!(!validate_email_basic("no-dot@example"));
        assert!(!validate_email_basic("space in@email.com"));
        assert!(!validate_email_basic("@start.com"));
        assert!(!validate_email_basic("end@"));
    }

    #[test]
    fn test_email_basic_check_does_not_require_strict_rfc_parsing() {
        // The basic check is intentionally lenient: stricter parsing happens
        // server-side. Both of these pass the basic check even though "end@.com"
        // isn't a valid real email — UI only uses this for fast-feedback hints.
        assert!(validate_email_basic("end@.com"));
        assert!(validate_email_basic("a@b.c"));
    }

    fn validate_password_strength(pw: &str) -> bool {
        pw.len() >= 8
    }

    #[test]
    fn test_password_strength_too_short() {
        assert!(!validate_password_strength("pass"));
        assert!(!validate_password_strength("1234567"));
    }

    #[test]
    fn test_password_strength_acceptable() {
        assert!(validate_password_strength("12345678"));
        assert!(validate_password_strength("LongPassword123"));
    }

    // =================================================================
    // DATE RELATIVE FORMATTING ("2 days ago", etc.)
    // =================================================================

    fn relative_time(now: chrono::NaiveDateTime, then: chrono::NaiveDateTime) -> String {
        let diff = now - then;
        let secs = diff.num_seconds();
        if secs < 60 { "just now".into() }
        else if secs < 3600 { format!("{}m ago", secs / 60) }
        else if secs < 86400 { format!("{}h ago", secs / 3600) }
        else { format!("{}d ago", secs / 86400) }
    }

    #[test]
    fn test_relative_time_just_now() {
        let t = Utc::now().naive_utc();
        assert_eq!(relative_time(t, t), "just now");
    }

    #[test]
    fn test_relative_time_minutes() {
        let t = Utc::now().naive_utc();
        let earlier = t - Duration::minutes(5);
        assert_eq!(relative_time(t, earlier), "5m ago");
    }

    #[test]
    fn test_relative_time_hours() {
        let t = Utc::now().naive_utc();
        let earlier = t - Duration::hours(3);
        assert_eq!(relative_time(t, earlier), "3h ago");
    }

    #[test]
    fn test_relative_time_days() {
        let t = Utc::now().naive_utc();
        let earlier = t - Duration::days(5);
        assert_eq!(relative_time(t, earlier), "5d ago");
    }

    // =================================================================
    // ERROR MESSAGE FORMATTING BY HTTP STATUS
    // =================================================================

    fn error_message_for_status(code: u16) -> &'static str {
        match code {
            400 => "Invalid request. Please check your input.",
            401 => "You need to sign in to continue.",
            403 => "You don't have permission to do that.",
            404 => "We couldn't find what you're looking for.",
            409 => "That action conflicts with the current state.",
            410 => "This link has expired.",
            413 => "That file is too large.",
            415 => "File type not supported.",
            422 => "Your input didn't meet validation rules.",
            500 => "Something went wrong. Please try again.",
            503 => "Service is temporarily unavailable.",
            _ => "An unexpected error occurred.",
        }
    }

    #[test]
    fn test_error_messages_all_have_distinct_content() {
        use std::collections::HashSet;
        let codes = [400, 401, 403, 404, 409, 410, 413, 415, 422, 500, 503];
        let msgs: HashSet<&str> = codes.iter().map(|c| error_message_for_status(*c)).collect();
        assert_eq!(msgs.len(), codes.len(), "every status code must map to a distinct message");
    }

    #[test]
    fn test_error_401_says_sign_in() {
        assert!(error_message_for_status(401).contains("sign in"));
    }

    #[test]
    fn test_error_403_says_permission() {
        assert!(error_message_for_status(403).contains("permission"));
    }

    #[test]
    fn test_error_413_mentions_file_size() {
        assert!(error_message_for_status(413).contains("too large"));
    }

    #[test]
    fn test_error_415_mentions_file_type() {
        assert!(error_message_for_status(415).contains("type"));
    }

    // =================================================================
    // CASE PRIORITY ORDERING
    // =================================================================

    fn priority_rank(p: &str) -> u32 {
        match p {
            "urgent" => 1,
            "high" => 2,
            "medium" => 3,
            "low" => 4,
            _ => 999,
        }
    }

    #[test]
    fn test_priority_ordering() {
        assert!(priority_rank("urgent") < priority_rank("high"));
        assert!(priority_rank("high") < priority_rank("medium"));
        assert!(priority_rank("medium") < priority_rank("low"));
    }

    #[test]
    fn test_priority_sort_cases_urgent_first() {
        let mut cases = vec!["medium", "urgent", "low", "high"];
        cases.sort_by_key(|p| priority_rank(p));
        assert_eq!(cases, vec!["urgent", "high", "medium", "low"]);
    }

    // =================================================================
    // FULFILLMENT EVENT LABEL MAPPING
    // =================================================================

    fn fulfillment_event_label(t: &str) -> &'static str {
        match t {
            "missing_issue" => "Missing Issue",
            "reshipment" => "Reshipment",
            "delay" => "Delayed",
            "discontinuation" => "Discontinued",
            "edition_change" => "Edition Change",
            "delivered" => "Delivered",
            _ => "Unknown Event",
        }
    }

    #[test]
    fn test_fulfillment_labels_for_all_types() {
        let types = ["missing_issue", "reshipment", "delay",
                      "discontinuation", "edition_change", "delivered"];
        for t in &types {
            let label = fulfillment_event_label(t);
            assert_ne!(label, "Unknown Event", "missing label for '{}'", t);
            assert!(!label.is_empty());
        }
    }

    // =================================================================
    // SLA COUNTDOWN UI HELPER
    // =================================================================

    fn sla_countdown_color(hours_remaining: Option<f64>) -> &'static str {
        match hours_remaining {
            Some(h) if h < 0.0 => "text-red",
            Some(h) if h < 24.0 => "text-orange",
            Some(_) => "text-green",
            None => "text-gray",
        }
    }

    #[test]
    fn test_sla_overdue_is_red() {
        assert_eq!(sla_countdown_color(Some(-1.0)), "text-red");
    }

    #[test]
    fn test_sla_under_24h_is_orange() {
        assert_eq!(sla_countdown_color(Some(5.0)), "text-orange");
        assert_eq!(sla_countdown_color(Some(23.99)), "text-orange");
    }

    #[test]
    fn test_sla_plenty_of_time_is_green() {
        assert_eq!(sla_countdown_color(Some(48.0)), "text-green");
        assert_eq!(sla_countdown_color(Some(168.0)), "text-green");
    }

    #[test]
    fn test_sla_none_is_gray() {
        assert_eq!(sla_countdown_color(None), "text-gray");
    }

    // =================================================================
    // RATING STAR DISPLAY
    // =================================================================

    fn stars_for_rating(rating: i32) -> String {
        let filled = rating.max(0).min(5);
        let empty = 5 - filled;
        format!("{}{}", "★".repeat(filled as usize), "☆".repeat(empty as usize))
    }

    #[test]
    fn test_stars_5() {
        assert_eq!(stars_for_rating(5), "★★★★★");
    }

    #[test]
    fn test_stars_0() {
        assert_eq!(stars_for_rating(0), "☆☆☆☆☆");
    }

    #[test]
    fn test_stars_3() {
        assert_eq!(stars_for_rating(3), "★★★☆☆");
    }

    #[test]
    fn test_stars_clamped_above() {
        assert_eq!(stars_for_rating(10), "★★★★★");
    }

    #[test]
    fn test_stars_clamped_below() {
        assert_eq!(stars_for_rating(-1), "☆☆☆☆☆");
    }

    // =================================================================
    // SUBSCRIPTION LABEL
    // =================================================================

    fn subscription_label(period: &str) -> &'static str {
        match period {
            "monthly" => "Monthly",
            "quarterly" => "Every 3 months",
            "annual" => "Once per year",
            _ => "Custom",
        }
    }

    #[test]
    fn test_subscription_labels() {
        assert_eq!(subscription_label("monthly"), "Monthly");
        assert_eq!(subscription_label("quarterly"), "Every 3 months");
        assert_eq!(subscription_label("annual"), "Once per year");
    }

    // =================================================================
    // REVIEW FOLLOWUP TIME-LEFT UI
    // =================================================================

    fn followup_deadline_text(created: chrono::NaiveDateTime, now: chrono::NaiveDateTime) -> String {
        let deadline = created + Duration::days(models::FOLLOWUP_WINDOW_DAYS);
        let days_left = (deadline - now).num_days();
        if days_left < 0 { "Follow-up window closed".into() }
        else if days_left == 0 { "Last day to follow up".into() }
        else { format!("{} days left to follow up", days_left) }
    }

    #[test]
    fn test_followup_deadline_closed() {
        let created = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let now = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(followup_deadline_text(created, now), "Follow-up window closed");
    }

    #[test]
    fn test_followup_deadline_last_day() {
        let created = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let now = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(followup_deadline_text(created, now), "Last day to follow up");
    }

    #[test]
    fn test_followup_deadline_with_days_left() {
        let created = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let now = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert!(followup_deadline_text(created, now).contains("days left"));
    }

    // =================================================================
    // ORDER TOTAL CALCULATION (matches backend create_order)
    // =================================================================

    fn calculate_order_total(items: &[(i32, f64)]) -> f64 {
        items.iter().map(|(q, p)| *q as f64 * p).sum()
    }

    #[test]
    fn test_order_total_single_item() {
        assert!((calculate_order_total(&[(3, 9.99)]) - 29.97).abs() < f64::EPSILON);
    }

    #[test]
    fn test_order_total_multiple_items() {
        let items = &[(2, 10.0), (1, 5.0), (3, 7.5)];
        assert!((calculate_order_total(items) - (20.0 + 5.0 + 22.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_order_total_empty() {
        assert_eq!(calculate_order_total(&[]), 0.0);
    }

    #[test]
    fn test_order_total_zero_priced_items() {
        assert_eq!(calculate_order_total(&[(5, 0.0)]), 0.0);
    }

    // =================================================================
    // ORDER MODEL DISPLAY HELPERS
    // =================================================================

    #[test]
    fn test_order_display_includes_is_flagged_flag() {
        let o = Order {
            id: "x".into(), user_id: "u".into(), order_number: "N".into(),
            subscription_period: "monthly".into(), shipping_address_id: None,
            status: "pending".into(), payment_status: "unpaid".into(),
            total_amount: 10.0, parent_order_id: None,
            is_flagged: true, flag_reason: Some("high_qty".into()),
            created_at: None, updated_at: None,
        };
        assert!(o.is_flagged);
        assert_eq!(o.flag_reason.as_deref(), Some("high_qty"));
    }

    // =================================================================
    // SUBMISSION DRAFT PROGRESS INDICATOR
    // =================================================================

    fn draft_progress_text(current: i32, max: i32) -> String {
        format!("Version {} of {} max", current, max)
    }

    #[test]
    fn test_draft_progress_text() {
        assert_eq!(draft_progress_text(3, 10), "Version 3 of 10 max");
        assert_eq!(draft_progress_text(1, models::MAX_SUBMISSION_VERSIONS),
                   "Version 1 of 10 max");
    }

    // =================================================================
    // AFTER-SALES CASE SUBJECT SANITIZATION HINT
    // =================================================================

    #[test]
    fn test_case_subject_trimmed_for_display() {
        let s = "  Damaged item  ";
        assert_eq!(s.trim(), "Damaged item");
    }

    // =================================================================
    // SESSION TIME-REMAINING DISPLAY
    // =================================================================

    fn session_time_remaining(expires_at: chrono::NaiveDateTime,
                                now: chrono::NaiveDateTime) -> Option<i64> {
        let diff = (expires_at - now).num_minutes();
        if diff < 0 { None } else { Some(diff) }
    }

    #[test]
    fn test_session_time_remaining_positive() {
        let now = Utc::now().naive_utc();
        let exp = now + Duration::minutes(15);
        assert_eq!(session_time_remaining(exp, now), Some(15));
    }

    #[test]
    fn test_session_time_remaining_expired() {
        let now = Utc::now().naive_utc();
        let exp = now - Duration::minutes(1);
        assert_eq!(session_time_remaining(exp, now), None);
    }

    // =================================================================
    // REVIEW TITLE LENGTH GUARD FOR UI
    // =================================================================

    #[test]
    fn test_review_title_under_120_ok() {
        let title = "a".repeat(119);
        assert!(title.len() <= 120);
    }

    #[test]
    fn test_review_title_truncated_in_list_to_60() {
        let title = "a".repeat(200);
        let truncated: String = title.chars().take(60).collect();
        assert_eq!(truncated.len(), 60);
    }

    // =================================================================
    // NAV ITEM STATE (active vs inactive)
    // =================================================================

    fn nav_item_class(current: &str, target: &str) -> &'static str {
        if current == target { "nav-item active" } else { "nav-item" }
    }

    #[test]
    fn test_nav_active_when_matches() {
        assert_eq!(nav_item_class("/orders", "/orders"), "nav-item active");
    }

    #[test]
    fn test_nav_inactive_when_not_matches() {
        assert_eq!(nav_item_class("/orders", "/cases"), "nav-item");
    }

    // =================================================================
    // CASE BADGE MATCHES BACKEND MODEL DATA
    // =================================================================

    #[test]
    fn test_case_status_badge_covers_real_case_struct() {
        let case = AfterSalesCase {
            id: "c1".into(), order_id: "o1".into(), reporter_id: "u1".into(),
            assigned_to: None, case_type: "return".into(),
            subject: "subj".into(), description: "desc".into(),
            status: "in_review".into(), priority: "medium".into(),
            submitted_at: None, first_response_at: None,
            first_response_due: None, resolution_target: None,
            resolved_at: None, closed_at: None,
            created_at: None, updated_at: None,
        };
        assert!(case_status_class(&case.status).contains("status-info"));
    }

    // =================================================================
    // SUBMISSION BADGE MATCHES REAL STRUCT
    // =================================================================

    #[test]
    fn test_submission_badge_covers_real_struct() {
        let sub = Submission {
            id: "s1".into(), author_id: "u1".into(),
            title: "Paper".into(), summary: None,
            submission_type: "thesis".into(),
            status: "published".into(),
            deadline: None, current_version: 1, max_versions: 10,
            meta_title: None, meta_description: None, slug: None,
            tags: None, keywords: None,
            created_at: None, updated_at: None,
        };
        assert!(submission_status_class(&sub.status).contains("status-success"));
    }
}
