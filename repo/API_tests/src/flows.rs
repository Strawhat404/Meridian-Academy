//! End-to-end flow tests: complete student, instructor, and staff journeys
//! exercised through the public REST API.

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    fn backend_url() -> String {
        std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct LoginResponse {
        token: String,
        user: UserResponse,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct UserResponse {
        id: String,
        username: String,
        email: String,
        first_name: String,
        last_name: String,
        role: String,
        is_active: bool,
    }

    fn client() -> Client { Client::new() }
    fn uid() -> String { uuid::Uuid::new_v4().to_string()[..8].to_string() }

    async fn login_admin(c: &Client) -> LoginResponse {
        let r = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": "admin", "password": "admin123" }))
            .send().await.expect("backend reachable");
        assert_eq!(r.status(), 200);
        r.json::<LoginResponse>().await.unwrap()
    }

    async fn create_user(c: &Client, username: &str, role: &str) -> LoginResponse {
        let admin = login_admin(c).await;
        let r = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "username": username,
                "email": format!("{}@meridian.edu", username),
                "password": "TestP@ss123",
                "first_name": "Test", "last_name": "User",
                "role": role
            }))
            .send().await.expect("backend reachable");
        assert!(r.status().is_success(), "provision failed: {}", r.status());
        r.json::<LoginResponse>().await.unwrap()
    }

    // =================================================================
    // STUDENT END-TO-END JOURNEY
    // =================================================================

    #[tokio::test]
    async fn e2e_student_orders_then_reviews_delivered_item() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2es_{}", id), "student").await;

        // 1. Student creates an order
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal X", "quantity": 1, "unit_price": 15.0}]
            }))
            .send().await.unwrap();
        assert_eq!(order_r.status(), 200, "order creation must succeed");
        let order_body: serde_json::Value = order_r.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();
        assert_eq!(order_body["order"]["status"].as_str().unwrap(), "pending");
        assert_eq!(order_body["order"]["payment_status"].as_str().unwrap(), "unpaid");

        // 2. Admin marks order delivered (review prerequisite)
        let upd = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.unwrap();
        assert_eq!(upd.status(), 200);

        // 3. Student posts a review
        let rev_r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": order_id, "rating": 5, "title": "Loved it", "body": "Great." }))
            .send().await.unwrap();
        assert_eq!(rev_r.status(), 200);
        let rev: serde_json::Value = rev_r.json().await.unwrap();
        assert_eq!(rev["rating"].as_i64().unwrap(), 5);
        assert!(!rev["is_followup"].as_bool().unwrap());

        // 4. Student lists their own reviews and sees it
        let my_revs = c.get(&format!("{}/api/reviews/my", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(my_revs.status(), 200);
        let revs: Vec<serde_json::Value> = my_revs.json().await.unwrap();
        assert!(revs.iter().any(|r| r["id"] == rev["id"]));
    }

    #[tokio::test]
    async fn e2e_student_case_happy_path_with_comments() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2ec_{}", id), "student").await;

        // Student places order
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "quarterly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 12.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        // Opens a case
        let case_r = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "case_type": "refund",
                "subject": "Issue with book", "description": "Back cover torn",
                "priority": "high"
            })).send().await.unwrap();
        assert_eq!(case_r.status(), 200);
        let case_body: serde_json::Value = case_r.json().await.unwrap();
        let case_id = case_body["case"]["id"].as_str().unwrap().to_string();
        assert_eq!(case_body["case"]["status"].as_str().unwrap(), "submitted");
        assert_eq!(case_body["case"]["priority"].as_str().unwrap(), "high");
        // SLA fields must be populated
        assert!(case_body["hours_until_first_response"].as_f64().is_some());
        assert!(case_body["hours_until_resolution"].as_f64().is_some());
        assert!(!case_body["first_response_overdue"].as_bool().unwrap());

        // Student adds a comment
        let comment_r = c.post(&format!("{}/api/cases/{}/comments", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "content": "Additional details..." }))
            .send().await.unwrap();
        assert_eq!(comment_r.status(), 200);

        // Student reads back comments
        let list_r = c.get(&format!("{}/api/cases/{}/comments", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(list_r.status(), 200);
        let comments: Vec<serde_json::Value> = list_r.json().await.unwrap();
        assert!(comments.iter().any(|c| c["content"] == "Additional details..."));

        // Student views `my_cases`
        let my_cases = c.get(&format!("{}/api/cases/my", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(my_cases.status(), 200);
        let cases: Vec<serde_json::Value> = my_cases.json().await.unwrap();
        assert!(cases.iter().any(|cs| cs["case"]["id"] == case_id));
    }

    #[tokio::test]
    async fn e2e_admin_case_status_transitions_through_arbitration() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2ea_{}", id), "student").await;

        // Student creates order + case
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let case_r = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "case_type": "return",
                "subject": "S", "description": "D"
            })).send().await.unwrap();
        let case_id = case_r.json::<serde_json::Value>().await.unwrap()
            ["case"]["id"].as_str().unwrap().to_string();

        // Admin: submitted → in_review
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "in_review" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Admin: in_review → arbitrated
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "arbitrated" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Admin: arbitrated → approved (also sets resolved_at on the backend)
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "approved" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Admin: approved → closed
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "closed" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify state via GET
        let detail = c.get(&format!("{}/api/cases/{}", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert_eq!(body["case"]["status"].as_str().unwrap(), "closed");

        // After close, no further transition allowed
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "in_review" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400, "terminal case state cannot transition");
    }

    #[tokio::test]
    async fn e2e_case_invalid_transition_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2ci_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let case_r = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "case_type": "refund",
                "subject": "S", "description": "D"
            })).send().await.unwrap();
        let case_id = case_r.json::<serde_json::Value>().await.unwrap()
            ["case"]["id"].as_str().unwrap().to_string();

        // submitted → approved is not a valid direct transition
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "approved" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400);

        // submitted → closed not allowed
        let r = c.put(&format!("{}/api/cases/{}/status", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "closed" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    // =================================================================
    // SUBMISSION LIFECYCLE
    // =================================================================

    #[tokio::test]
    async fn e2e_submission_lifecycle_draft_to_published() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2sl_{}", id), "student").await;
        // academic_staff can review
        let staff = create_user(&c, &format!("e2st_{}", id), "academic_staff").await;

        // 1. Student creates a submission (starts in "draft")
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "title": "My Research",
                "summary": "Abstract",
                "submission_type": "thesis"
            })).send().await.unwrap();
        assert_eq!(sub_r.status(), 200);
        let sub: serde_json::Value = sub_r.json().await.unwrap();
        let sub_id = sub["id"].as_str().unwrap().to_string();
        assert_eq!(sub["status"].as_str().unwrap(), "draft");

        // 2. Student submits for review: draft → submitted
        let r = c.post(&format!("{}/api/content/items/{}/submit", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200, "submit action must succeed");

        // 3. Staff approves: submitted → accepted
        let r = c.post(&format!("{}/api/content/items/{}/approve", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", staff.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // 4. Admin publishes: accepted → published
        let r = c.post(&format!("{}/api/content/items/{}/publish", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // 5. Re-submitting a published item must fail (not in draft/revision_requested)
        let r = c.post(&format!("{}/api/content/items/{}/submit", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 409, "cannot re-submit a published item");
    }

    #[tokio::test]
    async fn e2e_submission_rejected_then_revision_requested() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2sr_{}", id), "student").await;
        let staff = create_user(&c, &format!("e2st_{}", id), "academic_staff").await;

        // Create draft, submit
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

        let _ = c.post(&format!("{}/api/content/items/{}/submit", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();

        // Staff requests revision
        let r = c.post(&format!("{}/api/content/items/{}/request-revision", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", staff.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Student can re-submit from revision_requested
        let r = c.post(&format!("{}/api/content/items/{}/submit", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200, "must be able to resubmit after revision_requested");
    }

    #[tokio::test]
    async fn e2e_content_approve_requires_reviewer_role() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2ca_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

        // Student cannot approve their own submission
        let r = c.post(&format!("{}/api/content/items/{}/approve", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403, "students must not approve submissions");
    }

    #[tokio::test]
    async fn e2e_content_submit_not_found() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2snf_{}", id), "student").await;
        let r = c.post(&format!("{}/api/content/items/nonexistent-id/submit", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 404);
    }

    // =================================================================
    // PROFILE / NOTIFICATION PREFERENCES
    // =================================================================

    #[tokio::test]
    async fn e2e_profile_update_changes_are_reflected_in_me() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("e2pu_{}", id), "student").await;

        let r = c.put(&format!("{}/api/users/profile", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "first_name": "Updated",
                "last_name": "Name",
                "contact_info": "555-1234"
            })).send().await.unwrap();
        assert!(r.status().is_success(), "profile update must succeed, got {}", r.status());

        let me = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.unwrap();
        let body: serde_json::Value = me.json().await.unwrap();
        assert_eq!(body["first_name"].as_str().unwrap(), "Updated");
        assert_eq!(body["last_name"].as_str().unwrap(), "Name");
        assert_eq!(body["contact_info"].as_str().unwrap(), "555-1234");
    }

    #[tokio::test]
    async fn e2e_notification_prefs_update_persists() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("e2np_{}", id), "student").await;

        let r = c.put(&format!("{}/api/users/notification-prefs", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "notify_submissions": false,
                "notify_orders": false,
                "notify_reviews": true,
                "notify_cases": true
            })).send().await.unwrap();
        assert!(r.status().is_success());

        let me = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.unwrap();
        let body: serde_json::Value = me.json().await.unwrap();
        assert!(!body["notify_submissions"].as_bool().unwrap());
        assert!(!body["notify_orders"].as_bool().unwrap());
        assert!(body["notify_reviews"].as_bool().unwrap());
        assert!(body["notify_cases"].as_bool().unwrap());
    }

    // =================================================================
    // CHANGE PASSWORD
    // =================================================================

    #[tokio::test]
    async fn e2e_change_password_success() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("e2cp_{}", id), "student").await;
        let username = format!("e2cp_{}", id);

        let r = c.post(&format!("{}/api/auth/change-password", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "current_password": "TestP@ss123",
                "new_password": "NewP@ssX456"
            })).send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Old password rejected
        let r = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": username, "password": "TestP@ss123" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);

        // New password works
        let r = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": username, "password": "NewP@ssX456" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    #[tokio::test]
    async fn e2e_change_password_wrong_current_rejected() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("e2cpw_{}", id), "student").await;

        let r = c.post(&format!("{}/api/auth/change-password", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "current_password": "wrong_password",
                "new_password": "NewP@ssX456"
            })).send().await.unwrap();
        assert_eq!(r.status(), 401, "wrong current password must be rejected");
    }

    #[tokio::test]
    async fn e2e_change_password_unauthenticated_rejected() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/change-password", backend_url()))
            .json(&json!({
                "current_password": "x", "new_password": "y"
            })).send().await.unwrap();
        assert_eq!(r.status(), 401);
    }

    // =================================================================
    // ADMIN DASHBOARD / SETTINGS / AUDIT
    // =================================================================

    #[tokio::test]
    async fn e2e_admin_dashboard_returns_expected_keys() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.get(&format!("{}/api/admin/dashboard", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();

        for key in &["total_users", "total_submissions", "total_orders",
                     "pending_cases", "flagged_orders", "total_revenue",
                     "blocked_content"] {
            assert!(body.get(*key).is_some(), "dashboard missing key '{}'", key);
        }
    }

    #[tokio::test]
    async fn e2e_system_settings_exposes_constants() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.get(&format!("{}/api/admin/settings", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();

        assert_eq!(body["session_timeout_minutes"].as_i64().unwrap(), 30);
        assert_eq!(body["password_reset_expiry_minutes"].as_i64().unwrap(), 60);
        assert_eq!(body["soft_delete_hold_days"].as_i64().unwrap(), 30);
        assert_eq!(body["max_submission_versions"].as_i64().unwrap(), 10);
        assert_eq!(body["max_file_size_mb"].as_i64().unwrap(), 25);
        assert_eq!(body["max_review_images"].as_i64().unwrap(), 6);
        // Verify notification channels structure
        assert!(body["notification_channels"]["in_app"]["available"].as_bool().unwrap());
        assert!(!body["notification_channels"]["email"]["available"].as_bool().unwrap());
        assert!(!body["notification_channels"]["sms"]["available"].as_bool().unwrap());
        // Verify allowed types
        let allowed_types: Vec<&str> = body["allowed_submission_types"]
            .as_array().unwrap().iter()
            .filter_map(|v| v.as_str()).collect();
        assert!(allowed_types.contains(&"journal_article"));
        assert!(allowed_types.contains(&"thesis"));
    }

    #[tokio::test]
    async fn e2e_audit_log_plural_endpoint_admin_only() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.get(&format!("{}/api/admin/audit-logs", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        // Should be an array
        let _logs: Vec<serde_json::Value> = r.json().await.unwrap();

        // Non-admin denied
        let id = uid();
        let student = create_user(&c, &format!("e2al_{}", id), "student").await;
        let r = c.get(&format!("{}/api/admin/audit-logs", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    #[tokio::test]
    async fn e2e_dashboard_denied_for_student() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2db_{}", id), "student").await;
        let r = c.get(&format!("{}/api/admin/dashboard", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    #[tokio::test]
    async fn e2e_dashboard_denied_for_instructor() {
        let c = client();
        let id = uid();
        let instr = create_user(&c, &format!("e2di_{}", id), "instructor").await;
        let r = c.get(&format!("{}/api/admin/dashboard", backend_url()))
            .header("Authorization", format!("Bearer {}", instr.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // =================================================================
    // PAYMENT RECONCILIATION REPORT
    // =================================================================

    #[tokio::test]
    async fn e2e_payment_reconciliation_report_admin_only() {
        let c = client();
        let admin = login_admin(&c).await;

        let r = c.get(&format!("{}/api/payments/reconciliation-report", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();
        for key in &["report_date", "total_charges", "total_holds",
                     "total_refunds", "expected_balance", "actual_balance",
                     "discrepancy"] {
            assert!(body.get(*key).is_some(), "missing key '{}'", key);
        }

        // Staff without payments.manage — denied (requires admin.dashboard)
        let id = uid();
        let staff = create_user(&c, &format!("e2rr_{}", id), "academic_staff").await;
        let r = c.get(&format!("{}/api/payments/reconciliation-report", backend_url()))
            .header("Authorization", format!("Bearer {}", staff.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    #[tokio::test]
    async fn e2e_payment_list_order_owner_can_see_their_payments() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2pl_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 20.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        // Admin processes a payment
        let _ = c.post(&format!("{}/api/payments/", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "idempotency_key": format!("k-{}", id),
                "payment_method": "cash", "amount": 20.0, "transaction_type": "charge"
            })).send().await.unwrap();

        // Owner can see payments on their own order
        let r = c.get(&format!("{}/api/payments/order/{}", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let payments: Vec<serde_json::Value> = r.json().await.unwrap();
        assert_eq!(payments.len(), 1);
        assert_eq!(payments[0]["order_id"].as_str().unwrap(), order_id);
    }

    #[tokio::test]
    async fn e2e_payment_list_non_owner_non_staff_forbidden() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("e2plA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("e2plB_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 20.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let r = c.get(&format!("{}/api/payments/order/{}", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // =================================================================
    // ABNORMAL FLAGS LIFECYCLE
    // =================================================================

    #[tokio::test]
    async fn e2e_high_quantity_order_is_flagged_and_listed() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2hq_{}", id), "student").await;

        // Quantity > HIGH_QUANTITY_THRESHOLD (50) triggers flagging
        let r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Bulk", "quantity": 100, "unit_price": 1.0}]
            })).send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();
        assert!(body["order"]["is_flagged"].as_bool().unwrap());
        assert!(body["order"]["flag_reason"].as_str().unwrap().contains("quantity"));

        // Flag appears in abnormal-flags list
        let r = c.get(&format!("{}/api/payments/abnormal-flags", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let flags: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(flags.iter().any(|f|
            f["flag_type"].as_str().unwrap_or("") == "high_quantity"
        ), "high_quantity flag must be listed");
    }

    #[tokio::test]
    async fn e2e_abnormal_flag_can_be_cleared_by_admin() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2fc_{}", id), "student").await;

        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "B", "quantity": 200, "unit_price": 1.0}]
            })).send().await.unwrap();

        // Get open flags
        let r = c.get(&format!("{}/api/payments/abnormal-flags", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        let flags: Vec<serde_json::Value> = r.json().await.unwrap();
        let flag = flags.iter().find(|f| !f["is_cleared"].as_bool().unwrap_or(true))
            .expect("must find an open flag");
        let flag_id = flag["id"].as_str().unwrap().to_string();

        // Clear it
        let r = c.post(&format!("{}/api/payments/abnormal-flags/{}/clear", backend_url(), flag_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    #[tokio::test]
    async fn e2e_abnormal_flags_denied_for_student() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2fd_{}", id), "student").await;

        let r = c.get(&format!("{}/api/payments/abnormal-flags", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // =================================================================
    // FULFILLMENT EVENTS BY IDOR + VALIDATION
    // =================================================================

    #[tokio::test]
    async fn e2e_fulfillment_invalid_event_type_rejected() {
        let c = client();
        let admin = login_admin(&c).await;
        let id = uid();
        let student = create_user(&c, &format!("e2fe_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let r = c.post(&format!("{}/api/orders/fulfillment", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id,
                "event_type": "invalid_type",
                "reason": "Some reason"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_fulfillment_list_owner_can_see() {
        let c = client();
        let admin = login_admin(&c).await;
        let id = uid();
        let student = create_user(&c, &format!("e2fl_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_body: serde_json::Value = order_r.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();
        let li_id = order_body["line_items"][0]["id"].as_str().unwrap().to_string();

        let _ = c.post(&format!("{}/api/orders/fulfillment", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "line_item_id": li_id,
                "event_type": "delay",
                "reason": "Warehouse delay"
            })).send().await;

        // Owner can list fulfillment events for their own order
        let r = c.get(&format!("{}/api/orders/{}/fulfillment", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    #[tokio::test]
    async fn e2e_fulfillment_list_foreign_order_denied() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("e2flA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("e2flB_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        // student_b not allowed
        let r = c.get(&format!("{}/api/orders/{}/fulfillment", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // =================================================================
    // PROVISION VALIDATION
    // =================================================================

    #[tokio::test]
    async fn e2e_provision_duplicate_username_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let username = format!("dup_{}", id);
        // First provision OK
        let _ = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "username": username, "email": format!("{}@meridian.edu", username),
                "password": "TestP@ss123", "first_name": "A", "last_name": "B",
                "role": "student"
            })).send().await.unwrap();

        // Second with same username → 409
        let r = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "username": username, "email": format!("other_{}@meridian.edu", username),
                "password": "TestP@ss123", "first_name": "A", "last_name": "B",
                "role": "student"
            })).send().await.unwrap();
        assert_eq!(r.status(), 409);
    }

    #[tokio::test]
    async fn e2e_provision_invalid_role_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let r = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "username": format!("bad_role_{}", id),
                "email": format!("br{}@meridian.edu", id),
                "password": "TestP@ss123",
                "first_name": "A", "last_name": "B",
                "role": "superadmin"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    // =================================================================
    // REVIEW RATING BOUNDS ENFORCED BY API
    // =================================================================

    #[tokio::test]
    async fn e2e_review_rating_zero_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2rb_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let _ = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.unwrap();

        let r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id,
                "rating": 0, "title": "Zero rating", "body": "x"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_review_rating_six_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2r6_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let _ = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.unwrap();

        let r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id,
                "rating": 6, "title": "Too high", "body": "x"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_review_on_undelivered_order_rejected() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2ru_{}", id), "student").await;

        // Order left in "pending" state (no delivery)
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id,
                "rating": 5, "title": "Nice", "body": "x"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400, "cannot review an undelivered order");
    }

    #[tokio::test]
    async fn e2e_duplicate_review_on_same_order_conflict() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2dr_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let _ = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.unwrap();

        let _ = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id,
                "rating": 4, "title": "1st", "body": "x"
            })).send().await.unwrap();

        // Second review on same order → 409
        let r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id,
                "rating": 5, "title": "2nd", "body": "y"
            })).send().await.unwrap();
        assert_eq!(r.status(), 409);
    }

    // =================================================================
    // ORDER BOUNDARY VALIDATION
    // =================================================================

    #[tokio::test]
    async fn e2e_order_empty_line_items_rejected() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2oe_{}", id), "student").await;

        let r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": []
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_order_invalid_subscription_period_rejected() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2osp_{}", id), "student").await;

        let r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "weekly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 1.0}]
            })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_order_status_invalid_rejected_by_admin() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("e2os_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 1.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let r = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "returned_to_warehouse" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_merge_orders_requires_minimum_two() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.post(&format!("{}/api/orders/merge", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "order_ids": ["only-one"] }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400);
    }

    #[tokio::test]
    async fn e2e_merge_cross_user_rejected() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student_a = create_user(&c, &format!("e2mrA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("e2mrB_{}", id), "student").await;

        let order_a = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "A", "quantity": 1, "unit_price": 1.0}]
            })).send().await.unwrap();
        let order_a_id = order_a.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let order_b = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "B", "quantity": 1, "unit_price": 1.0}]
            })).send().await.unwrap();
        let order_b_id = order_b.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let r = c.post(&format!("{}/api/orders/merge", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "order_ids": [order_a_id, order_b_id] }))
            .send().await.unwrap();
        assert_eq!(r.status(), 400, "cross-user merge must be rejected");
    }

    // =================================================================
    // NOTIFICATIONS LIST + READ
    // =================================================================

    #[tokio::test]
    async fn e2e_notifications_generated_on_order_create() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2nl_{}", id), "student").await;

        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();

        let r = c.get(&format!("{}/api/users/notifications", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let notifications: Vec<serde_json::Value> = r.json().await.unwrap();
        // At least one notification for the order should exist
        assert!(notifications.iter().any(|n|
            n["title"].as_str().unwrap_or("").contains("Order")
        ), "must have an order-related notification");
    }

    // =================================================================
    // EXPORT-MY-DATA COMPREHENSIVENESS
    // =================================================================

    #[tokio::test]
    async fn e2e_export_includes_submitted_content_and_order() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2ex_{}", id), "student").await;

        // Create content
        let _ = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "title": "Exported Paper", "submission_type": "thesis"
            })).send().await.unwrap();

        // Create an order
        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "X", "quantity": 1, "unit_price": 5.0}]
            })).send().await.unwrap();

        // Export
        let r = c.get(&format!("{}/api/auth/export-my-data", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();

        let subs = body["submissions"].as_array().unwrap();
        assert!(subs.iter().any(|s| s["title"] == "Exported Paper"));

        let orders = body["orders"].as_array().unwrap();
        assert!(!orders.is_empty());
    }

    #[tokio::test]
    async fn e2e_export_user_profile_does_not_contain_password_hash() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("e2eph_{}", id), "student").await;
        let r = c.get(&format!("{}/api/auth/export-my-data", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        let body: serde_json::Value = r.json().await.unwrap();
        let txt = body.to_string();
        assert!(!txt.contains("password_hash"), "password_hash must not appear in export");
    }

    // =================================================================
    // HEALTH ENDPOINT
    // =================================================================

    #[tokio::test]
    async fn e2e_health_no_auth_required() {
        let c = client();
        let r = c.get(&format!("{}/health", backend_url())).send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    // =================================================================
    // TOKEN MANIPULATION TESTS
    // =================================================================

    #[tokio::test]
    async fn e2e_malformed_bearer_rejected() {
        let c = client();
        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", "NotBearer xyz")
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
    }

    #[tokio::test]
    async fn e2e_missing_bearer_prefix_rejected() {
        let c = client();
        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", "some_raw_token")
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
    }

    #[tokio::test]
    async fn e2e_empty_authorization_rejected() {
        let c = client();
        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", "")
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
    }

    #[tokio::test]
    async fn e2e_bearer_with_garbage_token_rejected() {
        let c = client();
        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", "Bearer aaa.bbb.ccc")
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
    }

    // =================================================================
    // SENSITIVE WORDS MANAGEMENT (admin)
    // =================================================================

    #[tokio::test]
    async fn e2e_sensitive_words_admin_add_and_remove() {
        let c = client();
        let admin = login_admin(&c).await;
        let id = uid();
        let word = format!("inappropriate_{}", id);

        // Add
        let r = c.post(&format!("{}/api/content/sensitive-words", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "word": word, "action": "block"
            })).send().await.unwrap();
        assert_eq!(r.status(), 200);
        let added: serde_json::Value = r.json().await.unwrap();
        let word_id = added["id"].as_str().unwrap().to_string();

        // Verify visible in list
        let r = c.get(&format!("{}/api/content/sensitive-words", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        let words: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(words.iter().any(|w| w["id"] == word_id));

        // Remove
        let r = c.delete(&format!("{}/api/content/sensitive-words/{}", backend_url(), word_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 204);
    }

    #[tokio::test]
    async fn e2e_sensitive_words_invalid_action_rejected() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.post(&format!("{}/api/content/sensitive-words", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "word": "x", "action": "deleteme"
            })).send().await.unwrap();
        assert_eq!(r.status(), 400, "invalid action must be rejected");
    }

    // =================================================================
    // MY_SUBMISSIONS SCOPING
    // =================================================================

    #[tokio::test]
    async fn e2e_my_submissions_only_returns_own() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("e2msA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("e2msB_{}", id), "student").await;

        // A creates submission
        let _ = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({ "title": "A's Paper", "submission_type": "thesis" }))
            .send().await.unwrap();

        // B's my_submissions must not include A's
        let r = c.get(&format!("{}/api/submissions/my", backend_url()))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let subs: Vec<serde_json::Value> = r.json().await.unwrap();
        for s in &subs {
            assert_eq!(s["author_id"].as_str().unwrap(), student_b.user.id);
            assert_ne!(s["title"].as_str().unwrap(), "A's Paper");
        }
    }

    // =================================================================
    // COVERING PREVIOUSLY UNCOVERED ENDPOINTS
    // =================================================================

    // ---- GET /api/cases (list_cases — staff sees all, student sees own) ----

    #[tokio::test]
    async fn e2e_list_cases_admin_sees_all() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("lca_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let _ = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "case_type": "return",
                "subject": "S", "description": "D"
            })).send().await.unwrap();

        // Admin: GET /api/cases returns cases from all users
        let r = c.get(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let cases: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(!cases.is_empty(), "admin must see at least one case");
        // Verify response shape
        for cs in &cases {
            assert!(cs["case"]["id"].is_string());
            assert!(cs["case"]["status"].is_string());
            assert!(cs.get("first_response_overdue").is_some());
            assert!(cs.get("resolution_overdue").is_some());
        }
    }

    #[tokio::test]
    async fn e2e_list_cases_student_sees_only_own() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("lcs_{}", id), "student").await;

        let r = c.get(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let cases: Vec<serde_json::Value> = r.json().await.unwrap();
        for cs in &cases {
            assert_eq!(cs["case"]["reporter_id"].as_str().unwrap(), student.user.id,
                "student must only see own cases");
        }
    }

    // ---- GET /api/orders/flagged (list_flagged_orders — requires orders.manage) ----

    #[tokio::test]
    async fn e2e_list_flagged_orders_admin_can_see() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("lfo_{}", id), "student").await;

        // Create a high-qty order to generate a flag
        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Bulk", "quantity": 100, "unit_price": 1.0}]
            })).send().await.unwrap();

        let r = c.get(&format!("{}/api/orders/flagged", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let orders: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(!orders.is_empty(), "must see flagged orders");
        for o in &orders {
            assert!(o["is_flagged"].as_bool().unwrap(), "all returned orders must be flagged");
        }
    }

    #[tokio::test]
    async fn e2e_list_flagged_orders_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("lfod_{}", id), "student").await;

        let r = c.get(&format!("{}/api/orders/flagged", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- GET /api/submissions (list_submissions — all-list endpoint) ----

    #[tokio::test]
    async fn e2e_list_submissions_admin_sees_all() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("lsa_{}", id), "student").await;

        let _ = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Admin Visible Paper", "submission_type": "thesis" }))
            .send().await.unwrap();

        let r = c.get(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let subs: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(!subs.is_empty());
        // Verify response shape
        let first = &subs[0];
        assert!(first["id"].is_string());
        assert!(first["title"].is_string());
        assert!(first["submission_type"].is_string());
        assert!(first["status"].is_string());
        assert!(first["current_version"].is_number());
    }

    #[tokio::test]
    async fn e2e_list_submissions_student_sees_only_own() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("lss_{}", id), "student").await;

        let _ = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Own Paper", "submission_type": "thesis" }))
            .send().await.unwrap();

        let r = c.get(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let subs: Vec<serde_json::Value> = r.json().await.unwrap();
        for s in &subs {
            assert_eq!(s["author_id"].as_str().unwrap(), student.user.id);
        }
    }

    // ---- POST /api/content/items/:param/reject ----

    #[tokio::test]
    async fn e2e_content_reject_by_staff() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("crj_{}", id), "student").await;
        let staff = create_user(&c, &format!("crjs_{}", id), "academic_staff").await;

        // Create and submit
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Reject Me", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        let _ = c.post(&format!("{}/api/content/items/{}/submit", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();

        // Staff rejects
        let r = c.post(&format!("{}/api/content/items/{}/reject", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", staff.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify status changed to rejected
        let detail = c.get(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert_eq!(body["status"].as_str().unwrap(), "rejected");
    }

    #[tokio::test]
    async fn e2e_content_reject_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("crjd_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        let r = c.post(&format!("{}/api/content/items/{}/reject", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- POST /api/content/items/:param/rollback/:param ----

    #[tokio::test]
    async fn e2e_content_rollback_version() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("crb_{}", id), "student").await;
        use base64::{engine::general_purpose, Engine};

        // Create submission + upload 2 versions
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Rollback Test", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        let pdf = b"%PDF-1.4 version one";
        let b64 = general_purpose::STANDARD.encode(pdf);
        let _ = c.post(&format!("{}/api/submissions/{}/versions", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "file_name": "v1.pdf", "file_data": b64 }))
            .send().await.unwrap();

        let pdf2 = b"%PDF-1.4 version two";
        let b64_2 = general_purpose::STANDARD.encode(pdf2);
        let _ = c.post(&format!("{}/api/submissions/{}/versions", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "file_name": "v2.pdf", "file_data": b64_2 }))
            .send().await.unwrap();

        // Rollback to version 1
        let r = c.post(&format!("{}/api/content/items/{}/rollback/1", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify current_version is now 1 and status is draft
        let detail = c.get(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert_eq!(body["current_version"].as_i64().unwrap(), 1);
        assert_eq!(body["status"].as_str().unwrap(), "draft");
    }

    #[tokio::test]
    async fn e2e_content_rollback_nonexistent_version_fails() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("crbf_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        // No versions uploaded — rollback to version 99 must fail
        let r = c.post(&format!("{}/api/content/items/{}/rollback/99", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 404);
    }

    // ---- POST /api/orders/clear-flag ----

    #[tokio::test]
    async fn e2e_clear_flag_clears_order_flag() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("clf_{}", id), "student").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "B", "quantity": 200, "unit_price": 1.0}]
            })).send().await.unwrap();
        let order_body: serde_json::Value = order_r.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();
        assert!(order_body["order"]["is_flagged"].as_bool().unwrap());

        // Clear the flag
        let r = c.post(&format!("{}/api/orders/clear-flag", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "order_id": order_id }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify flag is cleared
        let detail = c.get(&format!("{}/api/orders/{}", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert!(!body["order"]["is_flagged"].as_bool().unwrap(), "flag must be cleared");
    }

    #[tokio::test]
    async fn e2e_clear_flag_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("clfd_{}", id), "student").await;
        let r = c.post(&format!("{}/api/orders/clear-flag", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": "fake" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- POST /api/submissions/:param/approve (approve_blocked) ----

    #[tokio::test]
    async fn e2e_approve_blocked_submission() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("abk_{}", id), "student").await;
        let staff = create_user(&c, &format!("abks_{}", id), "academic_staff").await;

        // Create a submission and force its status to 'blocked' via admin update
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Blocked Content", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        // Admin sets status to blocked
        let _ = c.put(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "blocked" }))
            .send().await.unwrap();

        // Staff approves the blocked submission
        let r = c.post(&format!("{}/api/submissions/{}/approve", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", staff.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify status changed to submitted
        let detail = c.get(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert_eq!(body["status"].as_str().unwrap(), "submitted");
    }

    #[tokio::test]
    async fn e2e_approve_blocked_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("abkd_{}", id), "student").await;
        let r = c.post(&format!("{}/api/submissions/fake-id/approve", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- PUT /api/cases/:param/assign ----

    #[tokio::test]
    async fn e2e_assign_case_to_staff() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("acas_{}", id), "student").await;
        let staff = create_user(&c, &format!("acst_{}", id), "academic_staff").await;

        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let case_r = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "case_type": "exchange",
                "subject": "Assign test", "description": "D"
            })).send().await.unwrap();
        let case_id = case_r.json::<serde_json::Value>().await.unwrap()
            ["case"]["id"].as_str().unwrap().to_string();

        // Admin assigns to staff
        let r = c.put(&format!("{}/api/cases/{}/assign", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "assigned_to": staff.user.id }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // Verify assignment via GET
        let detail = c.get(&format!("{}/api/cases/{}", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.unwrap();
        let body: serde_json::Value = detail.json().await.unwrap();
        assert_eq!(body["case"]["assigned_to"].as_str().unwrap(), staff.user.id);
    }

    #[tokio::test]
    async fn e2e_assign_case_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("acsd_{}", id), "student").await;
        let r = c.put(&format!("{}/api/cases/fake-id/assign", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "assigned_to": "someone" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- PUT /api/submissions/:param (update_submission) ----

    #[tokio::test]
    async fn e2e_update_submission_owner_can_edit_title() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("usb_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Original Title", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        let r = c.put(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "Updated Title", "summary": "New abstract" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();
        assert_eq!(body["title"].as_str().unwrap(), "Updated Title");
        assert_eq!(body["summary"].as_str().unwrap(), "New abstract");
        // SEO fields should be auto-generated
        assert!(body["slug"].is_string());
        assert!(body["meta_title"].is_string());
    }

    #[tokio::test]
    async fn e2e_update_submission_non_owner_denied() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("usbA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("usbB_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({ "title": "A's Paper", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        let r = c.put(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .json(&json!({ "title": "Hijacked" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    #[tokio::test]
    async fn e2e_update_submission_student_cannot_change_status() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("usbs_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        // Student trying to set status directly must be forbidden
        let r = c.put(&format!("{}/api/submissions/{}", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "status": "published" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    // ---- PUT /api/users/notifications/:param/read ----

    #[tokio::test]
    async fn e2e_mark_notification_read() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("mnr_{}", id), "student").await;

        // Create an order to trigger a notification
        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "J", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();

        // List notifications to get an ID
        let r = c.get(&format!("{}/api/users/notifications", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let notifs: Vec<serde_json::Value> = r.json().await.unwrap();

        if let Some(notif) = notifs.first() {
            let notif_id = notif["id"].as_str().unwrap();
            assert!(!notif["is_read"].as_bool().unwrap(), "new notification should be unread");

            // Mark it as read
            let r = c.put(&format!("{}/api/users/notifications/{}/read", backend_url(), notif_id))
                .header("Authorization", format!("Bearer {}", student.token))
                .send().await.unwrap();
            assert_eq!(r.status(), 200);

            // Verify it's now read
            let r = c.get(&format!("{}/api/users/notifications", backend_url()))
                .header("Authorization", format!("Bearer {}", student.token))
                .send().await.unwrap();
            let notifs: Vec<serde_json::Value> = r.json().await.unwrap();
            let target = notifs.iter().find(|n| n["id"].as_str().unwrap() == notif_id);
            if let Some(n) = target {
                assert!(n["is_read"].as_bool().unwrap(), "notification must now be read");
            }
        }
    }

    #[tokio::test]
    async fn e2e_mark_notification_read_requires_auth() {
        let c = client();
        let r = c.put(&format!("{}/api/users/notifications/fake/read", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
    }
}
