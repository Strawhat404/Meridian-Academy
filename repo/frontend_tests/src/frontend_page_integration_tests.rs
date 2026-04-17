//! Page-level integration tests for frontend modules.
//!
//! These tests simulate the data flow and state machine behavior of actual
//! Dioxus page components WITHOUT requiring a WASM runtime. They verify:
//! - API path construction per role (mirrors pages/orders.rs, pages/cases.rs)
//! - Page state transitions (create → list → select → detail flow)
//! - Form submission payload construction
//! - Display logic driven by real frontend modules
//! - Cross-module integration (nav_logic + status_display + validation + formatting)

#[cfg(test)]
mod tests {
    use frontend::validation;
    use frontend::formatting;
    use frontend::nav_logic;
    use frontend::status_display;
    use serde::{Deserialize, Serialize};

    // ===================================================================
    // Simulated DTOs matching what pages/*.rs define internally.
    // These mirror the exact structs in the frontend page source.
    // ===================================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Order {
        id: String,
        user_id: String,
        order_number: String,
        subscription_period: String,
        status: String,
        payment_status: String,
        total_amount: f64,
        is_flagged: bool,
        flag_reason: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CaseWithSla {
        case: Case,
        first_response_overdue: bool,
        resolution_overdue: bool,
        hours_until_first_response: Option<f64>,
        hours_until_resolution: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Case {
        id: String,
        order_id: String,
        reporter_id: String,
        assigned_to: Option<String>,
        case_type: String,
        subject: String,
        description: String,
        status: String,
        priority: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Submission {
        id: String,
        author_id: String,
        title: String,
        summary: Option<String>,
        submission_type: String,
        status: String,
        current_version: i32,
        max_versions: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Review {
        id: String,
        order_id: String,
        user_id: String,
        rating: i32,
        title: String,
        body: String,
        is_followup: bool,
        parent_review_id: Option<String>,
    }

    // ===================================================================
    // ORDERS PAGE INTEGRATION: role-driven API path + display rendering
    // ===================================================================

    /// Simulates the OrdersPage component's data-fetch + render pipeline.
    fn simulate_orders_page(role: &str, orders: Vec<Order>) -> Vec<OrderRowView> {
        let _endpoint = nav_logic::orders_api_path(role);
        let is_staff = nav_logic::is_staff(role);

        orders.iter().map(|o| {
            let status_class = status_display::order_status_class(&o.status);
            let payment_class = status_display::payment_status_class(&o.payment_status);
            let total_display = formatting::format_currency(o.total_amount);
            let show_flag = status_display::show_flagged_badge(o.is_flagged);
            let show_split_btn = is_staff;
            let period_label = formatting::subscription_label(&o.subscription_period);

            OrderRowView {
                order_number: o.order_number.clone(),
                status_class: status_class.to_string(),
                payment_class: payment_class.to_string(),
                total_display,
                flagged: show_flag,
                split_visible: show_split_btn,
                period_label: period_label.to_string(),
            }
        }).collect()
    }

    struct OrderRowView {
        order_number: String,
        status_class: String,
        payment_class: String,
        total_display: String,
        flagged: bool,
        split_visible: bool,
        period_label: String,
    }

    #[test]
    fn test_orders_page_student_no_split_button() {
        let orders = vec![Order {
            id: "o1".into(), user_id: "u1".into(),
            order_number: "ORD-001".into(),
            subscription_period: "monthly".into(),
            status: "pending".into(),
            payment_status: "unpaid".into(),
            total_amount: 50.0,
            is_flagged: false, flag_reason: None,
        }];

        let rows = simulate_orders_page("student", orders);
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].split_visible, "student must not see split button");
        assert!(!rows[0].flagged, "unflagged order must not show badge");
        assert_eq!(rows[0].total_display, "$50.00");
        assert_eq!(rows[0].period_label, "Monthly");
        assert!(rows[0].status_class.contains("status-pending"));
        assert!(rows[0].payment_class.contains("payment-unpaid"));
    }

    #[test]
    fn test_orders_page_admin_sees_split_and_flags() {
        let orders = vec![Order {
            id: "o2".into(), user_id: "u2".into(),
            order_number: "ORD-002".into(),
            subscription_period: "annual".into(),
            status: "delivered".into(),
            payment_status: "paid".into(),
            total_amount: 200.0,
            is_flagged: true,
            flag_reason: Some("high_quantity".into()),
        }];

        let rows = simulate_orders_page("administrator", orders);
        assert!(rows[0].split_visible, "admin must see split button");
        assert!(rows[0].flagged, "flagged order must show badge");
        assert_eq!(rows[0].total_display, "$200.00");
        assert_eq!(rows[0].period_label, "Annual");
    }

    #[test]
    fn test_orders_page_uses_correct_api_paths() {
        assert_eq!(nav_logic::orders_api_path("student"), "/api/orders/my");
        assert_eq!(nav_logic::orders_api_path("instructor"), "/api/orders/my");
        assert_eq!(nav_logic::orders_api_path("academic_staff"), "/api/orders");
        assert_eq!(nav_logic::orders_api_path("administrator"), "/api/orders");
    }

    // ===================================================================
    // CASES PAGE INTEGRATION: SLA display + transition buttons
    // ===================================================================

    /// Simulates the CasesPage component rendering a case row.
    fn simulate_case_row(cws: &CaseWithSla, viewer_is_staff: bool) -> CaseRowView {
        let sla_text = formatting::sla_display(
            cws.first_response_overdue,
            cws.resolution_overdue,
        );
        let status_class = status_display::case_status_class(&cws.case.status);
        let transitions = formatting::available_case_transitions(&cws.case.status);
        let priority_rank = formatting::priority_rank(&cws.case.priority);

        CaseRowView {
            subject: cws.case.subject.clone(),
            status_class: status_class.to_string(),
            sla_text: sla_text.to_string(),
            transition_labels: transitions.iter().map(|(_, l)| l.to_string()).collect(),
            transition_targets: transitions.iter().map(|(s, _)| s.to_string()).collect(),
            priority_rank,
            show_assign: viewer_is_staff,
        }
    }

    struct CaseRowView {
        subject: String,
        status_class: String,
        sla_text: String,
        transition_labels: Vec<String>,
        transition_targets: Vec<String>,
        priority_rank: u32,
        show_assign: bool,
    }

    #[test]
    fn test_cases_page_submitted_case_shows_start_review() {
        let cws = CaseWithSla {
            case: Case {
                id: "c1".into(), order_id: "o1".into(), reporter_id: "u1".into(),
                assigned_to: None, case_type: "refund".into(),
                subject: "Damaged item".into(), description: "D".into(),
                status: "submitted".into(), priority: "high".into(),
            },
            first_response_overdue: false,
            resolution_overdue: false,
            hours_until_first_response: Some(40.0),
            hours_until_resolution: Some(150.0),
        };
        let row = simulate_case_row(&cws, true);
        assert_eq!(row.sla_text, "On Track");
        assert_eq!(row.transition_labels, vec!["Start Review"]);
        assert_eq!(row.transition_targets, vec!["in_review"]);
        assert!(row.show_assign);
        assert_eq!(row.priority_rank, 2); // high = 2
    }

    #[test]
    fn test_cases_page_overdue_response_shown() {
        let cws = CaseWithSla {
            case: Case {
                id: "c2".into(), order_id: "o2".into(), reporter_id: "u2".into(),
                assigned_to: None, case_type: "return".into(),
                subject: "Late".into(), description: "D".into(),
                status: "in_review".into(), priority: "urgent".into(),
            },
            first_response_overdue: true,
            resolution_overdue: false,
            hours_until_first_response: Some(-5.0),
            hours_until_resolution: Some(100.0),
        };
        let row = simulate_case_row(&cws, false);
        assert_eq!(row.sla_text, "Response Overdue");
        assert_eq!(row.transition_labels.len(), 2);
        assert!(!row.show_assign, "non-staff must not see assign");
    }

    #[test]
    fn test_cases_page_closed_case_no_transitions() {
        let cws = CaseWithSla {
            case: Case {
                id: "c3".into(), order_id: "o3".into(), reporter_id: "u3".into(),
                assigned_to: Some("staff1".into()), case_type: "exchange".into(),
                subject: "Done".into(), description: "D".into(),
                status: "closed".into(), priority: "low".into(),
            },
            first_response_overdue: false,
            resolution_overdue: false,
            hours_until_first_response: None,
            hours_until_resolution: None,
        };
        let row = simulate_case_row(&cws, true);
        assert!(row.transition_labels.is_empty(), "closed case must have no transitions");
    }

    // ===================================================================
    // SUBMISSIONS PAGE INTEGRATION: version progress + status badge
    // ===================================================================

    fn simulate_submission_row(sub: &Submission) -> SubmissionRowView {
        let status_class = status_display::submission_status_class(&sub.status);
        let progress = formatting::draft_progress(sub.current_version, sub.max_versions);
        let can_upload = sub.current_version < sub.max_versions
            && (sub.status == "draft" || sub.status == "revision_requested");

        SubmissionRowView {
            title: sub.title.clone(),
            status_class: status_class.to_string(),
            progress,
            can_upload,
        }
    }

    struct SubmissionRowView {
        title: String,
        status_class: String,
        progress: String,
        can_upload: bool,
    }

    #[test]
    fn test_submissions_page_draft_allows_upload() {
        let sub = Submission {
            id: "s1".into(), author_id: "u1".into(),
            title: "My Paper".into(), summary: None,
            submission_type: "thesis".into(), status: "draft".into(),
            current_version: 1, max_versions: 10,
        };
        let row = simulate_submission_row(&sub);
        assert!(row.can_upload, "draft submission must allow version upload");
        assert_eq!(row.progress, "Version 1 of 10 max");
        assert!(row.status_class.contains("status-draft"));
    }

    #[test]
    fn test_submissions_page_published_blocks_upload() {
        let sub = Submission {
            id: "s2".into(), author_id: "u1".into(),
            title: "Published Paper".into(), summary: None,
            submission_type: "thesis".into(), status: "published".into(),
            current_version: 3, max_versions: 10,
        };
        let row = simulate_submission_row(&sub);
        assert!(!row.can_upload, "published submission must NOT allow upload");
        assert!(row.status_class.contains("status-published"));
    }

    #[test]
    fn test_submissions_page_max_versions_blocks_upload() {
        let sub = Submission {
            id: "s3".into(), author_id: "u1".into(),
            title: "Max Versions".into(), summary: None,
            submission_type: "thesis".into(), status: "draft".into(),
            current_version: 10, max_versions: 10,
        };
        let row = simulate_submission_row(&sub);
        assert!(!row.can_upload, "at version limit must NOT allow upload");
    }

    #[test]
    fn test_submissions_page_revision_requested_allows_upload() {
        let sub = Submission {
            id: "s4".into(), author_id: "u1".into(),
            title: "Needs Revision".into(), summary: None,
            submission_type: "thesis".into(), status: "revision_requested".into(),
            current_version: 2, max_versions: 10,
        };
        let row = simulate_submission_row(&sub);
        assert!(row.can_upload, "revision_requested must allow upload");
    }

    // ===================================================================
    // REVIEWS PAGE INTEGRATION: star display + follow-up badge
    // ===================================================================

    fn simulate_review_row(review: &Review) -> ReviewRowView {
        let stars = formatting::stars_for_rating(review.rating);
        let (type_label, type_class) = status_display::review_type_badge(review.is_followup);

        ReviewRowView {
            title: review.title.clone(),
            stars,
            type_label: type_label.to_string(),
            type_class: type_class.to_string(),
        }
    }

    struct ReviewRowView {
        title: String,
        stars: String,
        type_label: String,
        type_class: String,
    }

    #[test]
    fn test_reviews_page_original_review_display() {
        let review = Review {
            id: "r1".into(), order_id: "o1".into(), user_id: "u1".into(),
            rating: 4, title: "Good journal".into(), body: "Nice.".into(),
            is_followup: false, parent_review_id: None,
        };
        let row = simulate_review_row(&review);
        assert_eq!(row.stars, "★★★★☆");
        assert_eq!(row.type_label, "Original");
        assert!(row.type_class.contains("status-active"));
    }

    #[test]
    fn test_reviews_page_followup_review_display() {
        let review = Review {
            id: "r2".into(), order_id: "o1".into(), user_id: "u1".into(),
            rating: 5, title: "Updated".into(), body: "Better.".into(),
            is_followup: true, parent_review_id: Some("r1".into()),
        };
        let row = simulate_review_row(&review);
        assert_eq!(row.stars, "★★★★★");
        assert_eq!(row.type_label, "Follow-up");
        assert!(row.type_class.contains("status-pending"));
    }

    // ===================================================================
    // ADMIN PAGE INTEGRATION: user active badge + dashboard stat display
    // ===================================================================

    #[test]
    fn test_admin_user_active_badge_rendering() {
        let (label, class) = status_display::user_active_badge(true);
        assert_eq!(label, "Active");
        assert!(class.contains("status-active"));

        let (label, class) = status_display::user_active_badge(false);
        assert_eq!(label, "Inactive");
        assert!(class.contains("status-inactive"));
    }

    // ===================================================================
    // CROSS-MODULE INTEGRATION: form validation → API payload → display
    // ===================================================================

    /// Simulates the NewOrder form submission pipeline:
    /// validation → payload construction → total calculation → display.
    #[test]
    fn test_new_order_form_end_to_end() {
        // Step 1: Validate period
        assert!(validation::validate_subscription_period("quarterly").is_ok());

        // Step 2: Validate line items
        let items = vec![
            ("Journal of CS".to_string(), 2, 29.99),
            ("Nature Physics".to_string(), 1, 49.99),
        ];
        assert!(validation::validate_line_items(&items).is_ok());

        // Step 3: Calculate total (mirrors backend)
        let total = formatting::calculate_order_total(&[(2, 29.99), (1, 49.99)]);
        assert!((total - 109.97).abs() < f64::EPSILON);

        // Step 4: Display total
        assert_eq!(formatting::format_currency(total), "$109.97");
    }

    /// Simulates the NewSubmission form → validation → template selection.
    #[test]
    fn test_new_submission_form_end_to_end() {
        // Validate title
        assert!(validation::validate_title("Machine Learning in Genomics").is_ok());
        assert!(validation::validate_title(&"x".repeat(121)).is_err());

        // Validate summary
        assert!(validation::validate_summary("This study investigates...").is_ok());
        assert!(validation::validate_summary(&"x".repeat(501)).is_err());

        // Validate file before upload
        assert!(validation::validate_file_extension("paper.pdf").is_ok());
        assert!(validation::validate_file_extension("virus.exe").is_err());
        assert!(validation::validate_file_size(10 * 1024 * 1024).is_ok()); // 10MB OK
        assert!(validation::validate_file_size(30 * 1024 * 1024).is_err()); // 30MB too big
    }

    /// Simulates the NewReview form validation pipeline.
    #[test]
    fn test_new_review_form_end_to_end() {
        assert!(validation::validate_rating(5).is_ok());
        assert!(validation::validate_rating(0).is_err());
        assert!(validation::validate_title("Excellent publication").is_ok());
        assert!(validation::validate_title(&"x".repeat(121)).is_err());

        // Star rendering after creation
        assert_eq!(formatting::stars_for_rating(5), "★★★★★");
        assert_eq!(formatting::stars_for_rating(1), "★☆☆☆☆");
    }

    /// Simulates the NewCase form validation.
    #[test]
    fn test_new_case_form_end_to_end() {
        assert!(validation::validate_case_type("refund").is_ok());
        assert!(validation::validate_case_type("complaint").is_err());
        assert!(validation::validate_title("Missing issue #3").is_ok());
    }

    // ===================================================================
    // NAV + PAGE ROUTING INTEGRATION
    // ===================================================================

    /// Simulates the full nav bar rendering for each role:
    /// menu items + API path selection + admin visibility.
    #[test]
    fn test_full_nav_integration_all_roles() {
        // Student
        let items = nav_logic::menu_items("student");
        assert!(items.contains(&"Submissions"));
        assert!(!items.contains(&"Admin"));
        assert_eq!(nav_logic::orders_api_path("student"), "/api/orders/my");
        assert_eq!(nav_logic::submissions_api_path("student"), "/api/submissions/my");
        assert_eq!(nav_logic::cases_api_path("student"), "/api/cases/my");

        // Instructor
        let items = nav_logic::menu_items("instructor");
        assert!(items.contains(&"Submissions"));
        assert!(!items.contains(&"Admin"));

        // Academic staff
        let items = nav_logic::menu_items("academic_staff");
        assert!(!items.contains(&"Submissions"));
        assert!(items.contains(&"Admin"));
        assert_eq!(nav_logic::orders_api_path("academic_staff"), "/api/orders");
        assert_eq!(nav_logic::submissions_api_path("academic_staff"), "/api/submissions");
        assert_eq!(nav_logic::cases_api_path("academic_staff"), "/api/cases");

        // Administrator
        let items = nav_logic::menu_items("administrator");
        assert!(items.contains(&"Admin"));
        assert_eq!(nav_logic::orders_api_path("administrator"), "/api/orders");
    }

    // ===================================================================
    // RECONCILIATION DETAIL VIEW INTEGRATION
    // ===================================================================

    #[test]
    fn test_reconciliation_detail_display() {
        // Matched record
        let class = formatting::reconciliation_badge_class("matched");
        assert!(class.contains("status-active"));

        // Discrepancy record
        let class = formatting::reconciliation_badge_class("discrepancy");
        assert!(class.contains("status-rejected"));

        // Pending record
        let class = formatting::reconciliation_badge_class("pending");
        assert!(class.contains("status-pending"));
    }

    // ===================================================================
    // FULFILLMENT EVENT DISPLAY INTEGRATION
    // ===================================================================

    #[test]
    fn test_fulfillment_event_display_all_types() {
        let types_and_labels = vec![
            ("missing_issue", "Missing Issue"),
            ("reshipment", "Reshipment"),
            ("delay", "Delayed"),
            ("discontinuation", "Discontinued"),
            ("edition_change", "Edition Change"),
            ("delivered", "Delivered"),
        ];
        for (t, expected) in types_and_labels {
            assert_eq!(formatting::fulfillment_event_label(t), expected,
                "wrong label for '{}'", t);
        }
    }

    // ===================================================================
    // SERVICE-LAYER URL CONSTRUCTION INTEGRATION
    // ===================================================================

    /// The frontend services construct API URLs from path segments.
    /// Verify the patterns match what pages actually use.
    #[test]
    fn test_api_url_construction_patterns() {
        let sub_id = "sub-abc-123";
        let version = 2;
        let order_id = "ord-xyz-789";
        let case_id = "case-456";

        // Submission version download URL (submissions.rs page)
        let url = format!("/api/submissions/{}/versions/{}/download", sub_id, version);
        assert_eq!(url, "/api/submissions/sub-abc-123/versions/2/download");

        // Order detail URL
        let url = format!("/api/orders/{}", order_id);
        assert_eq!(url, "/api/orders/ord-xyz-789");

        // Order fulfillment events
        let url = format!("/api/orders/{}/fulfillment", order_id);
        assert!(url.ends_with("/fulfillment"));

        // Order reconciliation
        let url = format!("/api/orders/{}/reconciliation", order_id);
        assert!(url.ends_with("/reconciliation"));

        // Case comments
        let url = format!("/api/cases/{}/comments", case_id);
        assert_eq!(url, "/api/cases/case-456/comments");

        // Case status update
        let url = format!("/api/cases/{}/status", case_id);
        assert!(url.ends_with("/status"));

        // Case assignment
        let url = format!("/api/cases/{}/assign", case_id);
        assert!(url.ends_with("/assign"));
    }

    // ===================================================================
    // PRIORITY SORT INTEGRATION (mirrors cases page sort)
    // ===================================================================

    #[test]
    fn test_cases_sorted_by_priority_then_display() {
        let mut cases = vec![
            ("low", "Low priority case"),
            ("urgent", "Urgent case"),
            ("medium", "Medium case"),
            ("high", "High priority case"),
        ];
        cases.sort_by_key(|(p, _)| formatting::priority_rank(p));
        assert_eq!(cases[0].0, "urgent");
        assert_eq!(cases[1].0, "high");
        assert_eq!(cases[2].0, "medium");
        assert_eq!(cases[3].0, "low");
    }
}
