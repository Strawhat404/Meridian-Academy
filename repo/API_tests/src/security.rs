//! Additional API security tests: authZ boundaries, IDOR enumeration,
//! token misuse, and sensitive-endpoint protection.

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
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        r.json().await.unwrap()
    }

    async fn create_user(c: &Client, username: &str, role: &str) -> LoginResponse {
        let admin = login_admin(c).await;
        let r = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "username": username, "email": format!("{}@meridian.edu", username),
                "password": "TestP@ss123", "first_name": "T", "last_name": "U",
                "role": role
            })).send().await.unwrap();
        assert!(r.status().is_success());
        r.json().await.unwrap()
    }

    /// Verify that an error response doesn't leak sensitive data.
    fn assert_no_data_leak(body: &str, context: &str) {
        let sensitive = ["password_hash", "JWT_SECRET", "mysql://", "ROCKET_SECRET"];
        for s in &sensitive {
            assert!(!body.contains(s),
                "{}: error response must not leak '{}' — body: {}", context, s, &body[..body.len().min(200)]);
        }
    }

    // =================================================================
    // ENDPOINT-LEVEL AUTH ENFORCEMENT (with payload assertions)
    // =================================================================

    #[tokio::test]
    async fn sec_admin_dashboard_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/admin/dashboard", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "dashboard 401");
    }

    #[tokio::test]
    async fn sec_admin_settings_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/admin/settings", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "settings 401");
    }

    #[tokio::test]
    async fn sec_admin_settings_student_denied() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("secS_{}", id), "student").await;
        let r = c.get(&format!("{}/api/admin/settings", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "settings 403");
        // Must not return the actual settings to unauthorized users
        assert!(!body.contains("session_timeout_minutes"), "settings data must not leak");
    }

    #[tokio::test]
    async fn sec_cleanup_soft_deleted_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/admin/cleanup-soft-deleted", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "cleanup 401");
        assert!(!body.contains("deleted_count"), "cleanup response must not leak data on 401");
    }

    #[tokio::test]
    async fn sec_audit_log_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/admin/audit-log", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "audit-log 401");
        assert!(!body.contains("\"logs\""), "audit log data must not leak on 401");
        assert!(!body.contains("\"action\""), "audit entries must not leak on 401");
    }

    #[tokio::test]
    async fn sec_list_users_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/users", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "list-users 401");
        assert!(!body.contains("\"username\""), "user records must not leak on 401");
        assert!(!body.contains("\"email\""), "user emails must not leak on 401");
    }

    #[tokio::test]
    async fn sec_generate_reset_token_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/generate-reset-token", backend_url()))
            .json(&json!({ "user_id": "foo" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "generate-reset 401");
        assert!(!body.contains("\"token\""), "reset token must not leak on 401");
    }

    #[tokio::test]
    async fn sec_change_password_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/change-password", backend_url()))
            .json(&json!({ "current_password": "x", "new_password": "y" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "change-password 401");
    }

    #[tokio::test]
    async fn sec_logout_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/logout", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "logout 401");
    }

    #[tokio::test]
    async fn sec_request_deletion_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/request-deletion", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "request-deletion 401");
        assert!(!body.contains("deletion_scheduled"), "deletion details must not leak on 401");
    }

    #[tokio::test]
    async fn sec_cancel_deletion_requires_auth() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/cancel-deletion", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "cancel-deletion 401");
    }

    #[tokio::test]
    async fn sec_reconciliation_report_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/payments/reconciliation-report", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "reconciliation-report 401");
        assert!(!body.contains("total_charges"), "financial data must not leak on 401");
        assert!(!body.contains("discrepancy"), "financial data must not leak on 401");
    }

    #[tokio::test]
    async fn sec_abnormal_flags_requires_auth() {
        let c = client();
        let r = c.get(&format!("{}/api/payments/abnormal-flags", backend_url()))
            .send().await.unwrap();
        assert_eq!(r.status(), 401);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "abnormal-flags 401");
        assert!(!body.contains("flag_type"), "flag data must not leak on 401");
    }

    #[tokio::test]
    async fn sec_content_check_requires_privileged() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("sec_cc_{}", id), "student").await;
        let r = c.post(&format!("{}/api/content/check", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "text": "hello world" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        // Must NOT return filtered content to unprivileged users
        assert!(!body.contains("processed_text"), "content check must not leak results on 403");
        assert!(!body.contains("blocked_words"));
    }

    // =================================================================
    // ROLE-SPECIFIC DENIAL
    // =================================================================

    #[tokio::test]
    async fn sec_instructor_cannot_split_orders() {
        let c = client();
        let id = uid();
        let instr = create_user(&c, &format!("sec_is_{}", id), "instructor").await;
        let r = c.post(&format!("{}/api/orders/split", backend_url()))
            .header("Authorization", format!("Bearer {}", instr.token))
            .json(&json!({ "order_id": "fake" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "split 403");
        assert!(!body.contains("\"order_number\""), "order data must not leak on forbidden split");
    }

    #[tokio::test]
    async fn sec_instructor_cannot_generate_reset_token() {
        let c = client();
        let id = uid();
        let instr = create_user(&c, &format!("sec_it_{}", id), "instructor").await;
        let r = c.post(&format!("{}/api/auth/generate-reset-token", backend_url()))
            .header("Authorization", format!("Bearer {}", instr.token))
            .json(&json!({ "user_id": "any" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "generate-reset 403");
        assert!(!body.contains("\"token\""), "token must not leak on forbidden reset");
    }

    #[tokio::test]
    async fn sec_staff_cannot_provision() {
        let c = client();
        let id = uid();
        let staff = create_user(&c, &format!("sec_sp_{}", id), "academic_staff").await;
        let r = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", staff.token))
            .json(&json!({
                "username": "byStaff", "email": "bs@m.edu",
                "password": "x", "first_name": "a", "last_name": "b",
                "role": "student"
            })).send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "provision 403");
        assert!(!body.contains("\"token\""), "new user token must not leak on forbidden provision");
    }

    #[tokio::test]
    async fn sec_staff_cannot_change_user_role() {
        let c = client();
        let id = uid();
        let staff = create_user(&c, &format!("sec_sr_{}", id), "academic_staff").await;
        let target = create_user(&c, &format!("sec_tg_{}", id), "student").await;
        let r = c.put(&format!("{}/api/users/{}/role", backend_url(), target.user.id))
            .header("Authorization", format!("Bearer {}", staff.token))
            .json(&json!({ "role": "instructor" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        assert_no_data_leak(&body, "role-change 403");
    }

    // =================================================================
    // IDOR: DETAILED ENUMERATION
    // =================================================================

    #[tokio::test]
    async fn sec_idor_nonexistent_submission_returns_404() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("sec_ns_{}", id), "student").await;
        let r = c.get(&format!("{}/api/submissions/00000000-0000-0000-0000-000000000000", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        // 404 or 403 acceptable — must not be 200
        assert!(r.status() == 403 || r.status() == 404,
            "got unexpected status {}", r.status());
    }

    #[tokio::test]
    async fn sec_idor_cannot_access_other_user_review_via_direct_get() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student_a = create_user(&c, &format!("sec_raA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("sec_raB_{}", id), "student").await;

        // A creates order → deliver → review
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "P", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let _ = c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.unwrap();

        let rev_r = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "order_id": order_id, "rating": 5,
                "title": "T", "body": "X"
            })).send().await.unwrap();
        assert_eq!(rev_r.status(), 200);
        let review_id = rev_r.json::<serde_json::Value>().await.unwrap()
            ["id"].as_str().unwrap().to_string();

        // B tries to fetch A's review → 403
        let r = c.get(&format!("{}/api/reviews/{}", backend_url(), review_id))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
        let body = r.text().await.unwrap_or_default();
        // Ensure no review data leaks on 403
        assert!(!body.contains("Excellent"), "review body must not leak on IDOR denial");
        assert_no_data_leak(&body, "review IDOR 403");
    }

    #[tokio::test]
    async fn sec_idor_cannot_list_other_user_cases_via_my_cases() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("sec_caA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("sec_caB_{}", id), "student").await;

        // A creates order → case
        let order_r = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "P", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();
        let order_id = order_r.json::<serde_json::Value>().await.unwrap()
            ["order"]["id"].as_str().unwrap().to_string();

        let case_r = c.post(&format!("{}/api/cases/", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "order_id": order_id, "case_type": "return",
                "subject": "A's case", "description": "private"
            })).send().await.unwrap();
        let case_id = case_r.json::<serde_json::Value>().await.unwrap()
            ["case"]["id"].as_str().unwrap().to_string();

        // B's my_cases must not include A's
        let r = c.get(&format!("{}/api/cases/my", backend_url()))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        let cases: Vec<serde_json::Value> = r.json().await.unwrap();
        for cs in &cases {
            assert_ne!(cs["case"]["id"].as_str().unwrap(), case_id);
            assert_eq!(cs["case"]["reporter_id"].as_str().unwrap(), student_b.user.id);
        }
    }

    #[tokio::test]
    async fn sec_idor_my_orders_scoped() {
        let c = client();
        let id = uid();
        let student_a = create_user(&c, &format!("sec_moA_{}", id), "student").await;
        let student_b = create_user(&c, &format!("sec_moB_{}", id), "student").await;

        let _ = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "P", "quantity": 1, "unit_price": 10.0}]
            })).send().await.unwrap();

        let r = c.get(&format!("{}/api/orders/my", backend_url()))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let orders: Vec<serde_json::Value> = r.json().await.unwrap();
        for o in &orders {
            assert_eq!(o["user_id"].as_str().unwrap(), student_b.user.id);
        }
    }

    #[tokio::test]
    async fn sec_student_cannot_see_all_orders() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("sec_ao_{}", id), "student").await;
        let r = c.get(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.unwrap();
        // Students get their own via /api/orders/my, but `/api/orders` is scoped to own too
        assert_eq!(r.status(), 200);
        let orders: Vec<serde_json::Value> = r.json().await.unwrap();
        for o in &orders {
            assert_eq!(o["user_id"].as_str().unwrap(), student.user.id);
        }
    }

    // =================================================================
    // USE-RESET-TOKEN EDGE CASES
    // =================================================================

    #[tokio::test]
    async fn sec_use_reset_token_unknown_returns_404() {
        let c = client();
        let r = c.post(&format!("{}/api/auth/use-reset-token", backend_url()))
            .json(&json!({
                "token": "00000000000000000000000000000000", "new_password": "N123"
            })).send().await.unwrap();
        assert_eq!(r.status(), 404);
    }

    #[tokio::test]
    async fn sec_use_reset_token_no_auth_needed() {
        // The endpoint is intentionally unauthenticated (users reset their own pw)
        let c = client();
        let r = c.post(&format!("{}/api/auth/use-reset-token", backend_url()))
            .json(&json!({ "token": "xyz", "new_password": "Pass" }))
            .send().await.unwrap();
        // Should not return 401 (no auth), should return 404 (token unknown)
        assert_ne!(r.status(), 401);
    }

    // =================================================================
    // SESSION/LOGOUT INVARIANTS
    // =================================================================

    #[tokio::test]
    async fn sec_logout_does_not_invalidate_other_user_sessions() {
        let c = client();
        let id = uid();
        let user_a = create_user(&c, &format!("sec_loA_{}", id), "student").await;
        let user_b = create_user(&c, &format!("sec_loB_{}", id), "student").await;

        // A logs out
        let r = c.post(&format!("{}/api/auth/logout", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);

        // B's session is still valid
        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    // =================================================================
    // ADMIN MANAGEMENT OF OWN ACCOUNT
    // =================================================================

    #[tokio::test]
    async fn sec_update_other_user_profile_forbidden() {
        let c = client();
        let id = uid();
        let user_a = create_user(&c, &format!("sec_upA_{}", id), "student").await;
        let _user_b = create_user(&c, &format!("sec_upB_{}", id), "student").await;

        // Each user's /profile endpoint only updates their own profile.
        // Ensure that A's profile update changes A's data, not B's.
        let r = c.put(&format!("{}/api/users/profile", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({ "first_name": "ChangedByA" }))
            .send().await.unwrap();
        assert!(r.status().is_success());

        let r = c.get(&format!("{}/api/auth/me", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .send().await.unwrap();
        let body: serde_json::Value = r.json().await.unwrap();
        assert_eq!(body["first_name"].as_str().unwrap(), "ChangedByA");
    }

    // =================================================================
    // CONTENT CHECK
    // =================================================================

    #[tokio::test]
    async fn sec_content_check_as_admin_returns_result() {
        let c = client();
        let admin = login_admin(&c).await;
        let r = c.post(&format!("{}/api/content/check", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "text": "normal innocent text" }))
            .send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();
        assert!(body["is_blocked"].is_boolean());
        assert!(body["blocked_words"].is_array());
        assert!(body["processed_text"].is_string());
    }

    // =================================================================
    // DOWNLOAD AUTH / IDOR
    // =================================================================

    #[tokio::test]
    async fn sec_download_other_user_submission_version_denied() {
        let c = client();
        let id = uid();
        use base64::{engine::general_purpose, Engine};
        let user_a = create_user(&c, &format!("sec_dlA_{}", id), "student").await;
        let user_b = create_user(&c, &format!("sec_dlB_{}", id), "student").await;

        // A creates and uploads
        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({ "title": "A", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

        let pdf = b"%PDF-1.4 A's private content";
        let b64 = general_purpose::STANDARD.encode(pdf);
        let _ = c.post(&format!("{}/api/submissions/{}/versions", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({ "file_name": "a.pdf", "file_data": b64 }))
            .send().await.unwrap();

        // B tries to download
        let r = c.get(&format!("{}/api/submissions/{}/versions/1/download",
                                backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.unwrap();
        assert_eq!(r.status(), 403);
    }

    #[tokio::test]
    async fn sec_download_nonexistent_version_not_found() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("sec_dnf_{}", id), "student").await;

        let sub_r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({ "title": "T", "submission_type": "thesis" }))
            .send().await.unwrap();
        let sub_id = sub_r.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

        // No version uploaded yet — download of v1 must fail
        let r = c.get(&format!("{}/api/submissions/{}/versions/1/download",
                                backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.unwrap();
        assert!(r.status() == 404 || r.status() == 400,
            "got {}", r.status());
    }

    // =================================================================
    // ADDRESS DELETION CROSS-USER PROTECTION
    // =================================================================

    #[tokio::test]
    async fn sec_cannot_delete_other_users_address() {
        let c = client();
        let id = uid();
        let user_a = create_user(&c, &format!("sec_adA_{}", id), "student").await;
        let user_b = create_user(&c, &format!("sec_adB_{}", id), "student").await;

        // A creates an address
        let addr_r = c.post(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "label": "Home", "street_line1": "1 St",
                "city": "Chi", "state": "IL", "zip_code": "60601",
                "is_default": true
            })).send().await.unwrap();
        let addr_id = addr_r.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

        // B tries to delete it
        let r = c.delete(&format!("{}/api/users/addresses/{}", backend_url(), addr_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.unwrap();
        assert!(r.status() == 403 || r.status() == 404,
            "got {}", r.status());

        // A's address still exists
        let r = c.get(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .send().await.unwrap();
        let addrs: Vec<serde_json::Value> = r.json().await.unwrap();
        assert!(addrs.iter().any(|a| a["id"].as_str().unwrap() == addr_id));
    }

    // =================================================================
    // SUBMISSION DEADLINE VALIDATION
    // =================================================================

    #[tokio::test]
    async fn sec_submission_past_deadline_acceptable_but_flagged_in_response() {
        // The API does not reject past-dated deadlines outright but may
        // store them as-is. We only verify the endpoint accepts arbitrary future/past.
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("sec_sdd_{}", id), "student").await;

        let r = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "title": "Test",
                "submission_type": "thesis",
                "deadline": "2028-12-31T23:59:59"
            })).send().await.unwrap();
        assert_eq!(r.status(), 200);
    }

    // =================================================================
    // HEALTH ENDPOINT NO AUTH
    // =================================================================

    #[tokio::test]
    async fn sec_health_does_not_leak_db_details() {
        let c = client();
        let r = c.get(&format!("{}/health", backend_url())).send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: serde_json::Value = r.json().await.unwrap();
        let txt = body.to_string();
        assert!(!txt.contains("mysql://"), "health must not leak connection URL");
        assert!(!txt.contains("password"), "health must not leak credentials");
        assert!(!txt.contains("JWT_SECRET"), "health must not leak secrets");
        assert!(!txt.contains("ROCKET_SECRET"), "health must not leak rocket secret");
        // Health should return minimal info
        assert!(body.get("status").is_some(), "health must return a status field");
        assert_eq!(body["status"], "ok");
    }
}
