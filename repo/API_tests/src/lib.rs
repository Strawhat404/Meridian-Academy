#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use base64;

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

    #[derive(Debug, Serialize, Deserialize)]
    struct Submission {
        id: String,
        author_id: String,
        title: String,
        status: String,
    }

    fn client() -> Client {
        Client::new()
    }

    /// Login as the seed admin. Panics if backend is unreachable.
    async fn login_admin(c: &Client) -> LoginResponse {
        let resp = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": "admin", "password": "admin123" }))
            .send().await
            .expect("Backend must be reachable for integration tests");
        assert_eq!(resp.status(), 200, "Admin login must succeed");
        resp.json::<LoginResponse>().await.expect("Valid login JSON")
    }

    /// Admin provisions a user. Panics on failure.
    async fn provision(c: &Client, token: &str, username: &str, email: &str, role: &str) -> LoginResponse {
        let resp = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "username": username, "email": email, "password": "TestP@ss123",
                "first_name": "Test", "last_name": "User", "role": role
            }))
            .send().await
            .expect("Backend must be reachable");
        assert!(resp.status().is_success(), "Provision must succeed, got {}", resp.status());
        resp.json::<LoginResponse>().await.expect("Valid provision JSON")
    }

    /// Login as a provisioned user. Panics on failure.
    async fn login(c: &Client, username: &str) -> LoginResponse {
        let resp = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": username, "password": "TestP@ss123" }))
            .send().await
            .expect("Backend must be reachable");
        assert!(resp.status().is_success(), "Login must succeed for {}", username);
        resp.json::<LoginResponse>().await.expect("Valid login JSON")
    }

    /// Provision + login helper.
    async fn create_user(c: &Client, username: &str, email: &str, role: &str) -> LoginResponse {
        let admin = login_admin(c).await;
        let _ = provision(c, &admin.token, username, email, role).await;
        login(c, username).await
    }

    fn uid() -> String { uuid::Uuid::new_v4().to_string()[..8].to_string() }

    // ===== HEALTH =====

    #[tokio::test]
    async fn test_health_check() {
        let c = client();
        let resp = c.get(&format!("{}/health", backend_url())).send().await
            .expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"], "ok");
    }

    // ===== AUTH =====

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let c = client();
        let resp = c.post(&format!("{}/api/auth/login", backend_url()))
            .json(&json!({ "username": "nonexistent", "password": "wrong" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn test_unauthorized_access() {
        let c = client();
        let resp = c.get(&format!("{}/api/auth/me", backend_url())).send().await
            .expect("Backend must be reachable");
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn test_self_registration_disabled() {
        let c = client();
        let resp = c.post(&format!("{}/api/auth/register", backend_url()))
            .json(&json!({
                "username": "selfregister", "email": "self@m.edu", "password": "Test123",
                "first_name": "S", "last_name": "R", "role": "student"
            }))
            .send().await.expect("Backend must be reachable");
        assert_ne!(resp.status(), 200, "Self-registration must be disabled");
    }

    #[tokio::test]
    async fn test_provision_requires_admin() {
        let c = client();
        let resp = c.post(&format!("{}/api/auth/provision", backend_url()))
            .json(&json!({
                "username": "noadmin", "email": "na@m.edu", "password": "T",
                "first_name": "N", "last_name": "A", "role": "student"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn test_student_cannot_provision() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("sprov_{}", id), &format!("sp{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/auth/provision", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "username": format!("v_{}", id), "email": format!("v{}@m.edu", id),
                "password": "T", "first_name": "V", "last_name": "U", "role": "student"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "Student must not provision accounts");
    }

    // ===== RBAC =====

    #[tokio::test]
    async fn test_student_cannot_access_admin() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("rbac_{}", id), &format!("rb{}@m.edu", id), "student").await;
        let resp = c.get(&format!("{}/api/admin/dashboard", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403);
    }

    #[tokio::test]
    async fn test_student_cannot_list_users() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("rbac2_{}", id), &format!("r2{}@m.edu", id), "student").await;
        let resp = c.get(&format!("{}/api/users", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403);
    }

    #[tokio::test]
    async fn test_student_cannot_manage_payments() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("pay_{}", id), &format!("py{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/payments", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({
                "order_id": "fake", "idempotency_key": "k1", "payment_method": "cash",
                "amount": 10.0, "transaction_type": "charge"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403);
    }

    #[tokio::test]
    async fn test_student_cannot_manage_sensitive_words() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("sw_{}", id), &format!("sw{}@m.edu", id), "student").await;
        let resp = c.get(&format!("{}/api/content/sensitive-words", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403);
    }

    #[tokio::test]
    async fn test_staff_cannot_create_submission() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("stf_{}", id), &format!("st{}@m.edu", id), "academic_staff").await;
        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({ "title": "Test", "submission_type": "thesis" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403);
    }

    // ===== VALIDATION =====

    #[tokio::test]
    async fn test_submission_title_too_long() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("long_{}", id), &format!("lg{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({ "title": "a".repeat(121), "submission_type": "thesis" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 422);
    }

    #[tokio::test]
    async fn test_case_invalid_type() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("cas_{}", id), &format!("cs{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({
                "order_id": "fake", "case_type": "complaint",
                "subject": "Test", "description": "Test"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 400);
    }

    // ===== IDOR: Cross-user access denied =====

    #[tokio::test]
    async fn test_idor_user_b_cannot_access_user_a_submission() {
        let c = client();
        let id = uid();

        let user_a = create_user(&c, &format!("idA_{}", id), &format!("idA{}@m.edu", id), "student").await;
        let user_b = create_user(&c, &format!("idB_{}", id), &format!("idB{}@m.edu", id), "student").await;

        // User A creates a submission
        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "title": "User A Private Paper",
                "summary": "Private research.",
                "submission_type": "journal_article"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        let submission_id = body["id"].as_str().expect("submission must have id");

        // User B tries to GET User A's submission — must be 403
        let resp = c.get(&format!("{}/api/submissions/{}", backend_url(), submission_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "IDOR: User B must not access User A's submission");

        // User B tries to GET User A's version history — must be 403
        let resp = c.get(&format!("{}/api/submissions/{}/versions", backend_url(), submission_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "IDOR: User B must not access User A's version history");
    }

    #[tokio::test]
    async fn test_idor_user_b_cannot_access_user_a_order() {
        let c = client();
        let id = uid();

        let user_a = create_user(&c, &format!("ioA_{}", id), &format!("ioA{}@m.edu", id), "student").await;
        let user_b = create_user(&c, &format!("ioB_{}", id), &format!("ioB{}@m.edu", id), "student").await;

        let resp = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Private", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        let order_id = body["order"]["id"].as_str().expect("order must have id");

        let resp = c.get(&format!("{}/api/orders/{}", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "IDOR: User B must not access User A's order");
    }

    #[tokio::test]
    async fn test_idor_review_list_scoped_to_user() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("rvs_{}", id), &format!("rvs{}@m.edu", id), "student").await;

        // Student list_reviews should only return their own
        let resp = c.get(&format!("{}/api/reviews", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let reviews: Vec<serde_json::Value> = resp.json().await.unwrap();
        // All returned reviews must belong to this user
        for rev in &reviews {
            assert_eq!(rev["user_id"].as_str().unwrap(), s.user.id, "List reviews must only return own reviews");
        }
    }

    // ===== CASE CREATE: order ownership binding =====

    #[tokio::test]
    async fn test_case_create_requires_own_order() {
        let c = client();
        let id = uid();

        // User A creates an order
        let user_a = create_user(&c, &format!("caA_{}", id), &format!("caA{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Test Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        let order_id = body["order"]["id"].as_str().expect("order must have id");

        // User B tries to open a case against User A's order — must be 403 or 404
        let user_b = create_user(&c, &format!("caB_{}", id), &format!("caB{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .json(&json!({
                "order_id": order_id,
                "case_type": "refund",
                "subject": "Unauthorized case",
                "description": "Should be rejected"
            }))
            .send().await.expect("Backend must be reachable");
        assert!(
            resp.status() == 403 || resp.status() == 404,
            "User B must not open a case against User A's order, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn test_case_create_nonexistent_order_rejected() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("cne_{}", id), &format!("cne{}@m.edu", id), "student").await;

        let resp = c.post(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({
                "order_id": "00000000-0000-0000-0000-000000000000",
                "case_type": "return",
                "subject": "Ghost order",
                "description": "Order does not exist"
            }))
            .send().await.expect("Backend must be reachable");
        assert!(
            resp.status() == 404 || resp.status() == 403,
            "Nonexistent order must be rejected, got {}",
            resp.status()
        );
    }

    // ===== SHIPPING ADDRESS OWNERSHIP =====

    #[tokio::test]
    async fn test_order_create_foreign_address_rejected() {
        let c = client();
        let id = uid();

        // User A creates an address using the correct route
        let user_a = create_user(&c, &format!("adA_{}", id), &format!("adA{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "label": "Home",
                "street_line1": "123 Main St",
                "city": "Springfield",
                "state": "IL",
                "zip_code": "62701",
                "is_default": true
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Address creation must succeed, got {}", resp.status());
        let addr: serde_json::Value = resp.json().await.unwrap();
        let addr_id = addr["id"].as_str().expect("address must have id");

        // User B tries to use User A's address_id in their own order — must be 403 or 404
        let user_b = create_user(&c, &format!("adB_{}", id), &format!("adB{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .json(&json!({
                "subscription_period": "monthly",
                "shipping_address_id": addr_id,
                "line_items": [{"publication_title": "Test", "quantity": 1, "unit_price": 5.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert!(
            resp.status() == 403 || resp.status() == 404,
            "User B must not use User A's address, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn test_address_create_set_default_delete() {
        let c = client();
        let id = uid();
        let user = create_user(&c, &format!("addr_{}", id), &format!("addr{}@m.edu", id), "student").await;

        // Create first address
        let resp = c.post(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "label": "Home", "street_line1": "1 Main St",
                "city": "Chicago", "state": "IL", "zip_code": "60601", "is_default": true
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Create address 1 must succeed");
        let addr1: serde_json::Value = resp.json().await.unwrap();
        let addr1_id = addr1["id"].as_str().unwrap().to_string();

        // Create second address (not default)
        let resp = c.post(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "label": "Office", "street_line1": "2 Work Ave",
                "city": "Chicago", "state": "IL", "zip_code": "60602", "is_default": false
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Create address 2 must succeed");
        let addr2: serde_json::Value = resp.json().await.unwrap();
        let addr2_id = addr2["id"].as_str().unwrap().to_string();

        // Set address 2 as default
        let resp = c.put(&format!("{}/api/users/addresses/default", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({ "address_id": addr2_id }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Set default must succeed");

        // Verify only one default: list addresses and check
        let resp = c.get(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let addrs: Vec<serde_json::Value> = resp.json().await.unwrap();
        let defaults: Vec<_> = addrs.iter().filter(|a| a["is_default"].as_bool().unwrap_or(false)).collect();
        assert_eq!(defaults.len(), 1, "Exactly one address must be default");
        assert_eq!(defaults[0]["id"].as_str().unwrap(), addr2_id, "addr2 must be the default");

        // Delete address 1
        let resp = c.delete(&format!("{}/api/users/addresses/{}", backend_url(), addr1_id))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 204, "Delete address must return 204");

        // Verify address 1 is gone
        let resp = c.get(&format!("{}/api/users/addresses", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .send().await.expect("Backend must be reachable");
        let addrs: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(!addrs.iter().any(|a| a["id"].as_str().unwrap() == addr1_id), "Deleted address must not appear in list");
    }

    // ===== CASE COMMENT AUTHORIZATION =====

    #[tokio::test]
    async fn test_case_comment_requires_involvement() {
        let c = client();
        let id = uid();

        // User A creates an order and a case
        let user_a = create_user(&c, &format!("ccA_{}", id), &format!("ccA{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(order_resp.status(), 200);
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap();

        let case_resp = c.post(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", user_a.token))
            .json(&json!({
                "order_id": order_id,
                "case_type": "refund",
                "subject": "Test case",
                "description": "Need refund"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(case_resp.status(), 200);
        let case_body: serde_json::Value = case_resp.json().await.unwrap();
        let case_id = case_body["case"]["id"].as_str().expect("case must have id");

        // Unrelated User B tries to comment — must be 403
        let user_b = create_user(&c, &format!("ccB_{}", id), &format!("ccB{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/cases/{}/comments", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", user_b.token))
            .json(&json!({ "content": "Unauthorized comment" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "Unrelated user must not comment on another user's case");
    }

    #[tokio::test]
    async fn test_case_comment_owner_allowed() {
        let c = client();
        let id = uid();

        // User creates order + case + comment — all must succeed
        let user = create_user(&c, &format!("cco_{}", id), &format!("cco{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(order_resp.status(), 200);
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap();

        let case_resp = c.post(&format!("{}/api/cases", backend_url()))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({
                "order_id": order_id,
                "case_type": "exchange",
                "subject": "Wrong edition",
                "description": "Received wrong edition"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(case_resp.status(), 200);
        let case_body: serde_json::Value = case_resp.json().await.unwrap();
        let case_id = case_body["case"]["id"].as_str().expect("case must have id");

        let comment_resp = c.post(&format!("{}/api/cases/{}/comments", backend_url(), case_id))
            .header("Authorization", format!("Bearer {}", user.token))
            .json(&json!({ "content": "Please process my exchange" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(comment_resp.status(), 200, "Case owner must be able to comment");
    }

    // ===== SUBMISSION BOUNDARY: summary and tags =====

    #[tokio::test]
    async fn test_submission_summary_too_long() {
        let c = client();
        let id = uid();
        let s = create_user(&c, &format!("smry_{}", id), &format!("sm{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", s.token))
            .json(&json!({ "title": "Valid Title", "submission_type": "thesis", "summary": "a".repeat(501) }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 422, "Summary over 500 chars must be rejected");
    }

    // ===== ORDER SPLIT/MERGE/FULFILLMENT/RECONCILIATION =====

    #[tokio::test]
    async fn test_student_cannot_split_order() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("spl_{}", id), &format!("spl{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/orders/split", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": "fake-id" }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "Student must not split orders");
    }

    #[tokio::test]
    async fn test_student_cannot_merge_orders() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("mrg_{}", id), &format!("mrg{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/orders/merge", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_ids": ["id1", "id2"] }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "Student must not merge orders");
    }

    #[tokio::test]
    async fn test_student_cannot_log_fulfillment_event() {
        let c = client();
        let id = uid();
        let student = create_user(&c, &format!("ful_{}", id), &format!("ful{}@m.edu", id), "student").await;
        let resp = c.post(&format!("{}/api/orders/fulfillment", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": "fake", "event_type": "missing_issue",
                "reason": "Test", "issue_identifier": null,
                "expected_date": null, "actual_date": null, "line_item_id": null
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 403, "Student must not log fulfillment events");
    }

    #[tokio::test]
    async fn test_fulfillment_event_requires_reason() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        // Create an order as a student first
        let student = create_user(&c, &format!("fre_{}", id), &format!("fre{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Test", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(order_resp.status(), 200);
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        // Admin tries to log fulfillment with empty reason — must be rejected
        let resp = c.post(&format!("{}/api/orders/fulfillment", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "event_type": "missing_issue",
                "reason": "   ", "issue_identifier": null,
                "expected_date": null, "actual_date": null, "line_item_id": null
            }))
            .send().await.expect("Backend must be reachable");
        assert!(
            resp.status() == 422 || resp.status() == 400,
            "Empty reason must be rejected, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn test_reconciliation_requires_auth() {
        let c = client();
        let resp = c.get(&format!("{}/api/orders/fake-id/reconciliation", backend_url()))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 401, "Reconciliation must require auth");
    }

    // ===== PAYMENT IDEMPOTENCY =====

    #[tokio::test]
    async fn test_payment_idempotency_no_double_charge() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("pay2_{}", id), &format!("pay2{}@m.edu", id), "student").await;

        // Create an order
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 50.0}]
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(order_resp.status(), 200);
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        let idem_key = format!("idem-{}", id);

        // First charge
        let r1 = c.post(&format!("{}/api/payments/", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "idempotency_key": idem_key,
                "payment_method": "cash", "amount": 50.0, "transaction_type": "charge"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(r1.status(), 200, "First charge must succeed");
        let p1: serde_json::Value = r1.json().await.unwrap();

        // Second charge with same idempotency key — must return same payment, not create new
        let r2 = c.post(&format!("{}/api/payments/", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "idempotency_key": idem_key,
                "payment_method": "cash", "amount": 50.0, "transaction_type": "charge"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(r2.status(), 200, "Idempotent charge must return 200");
        let p2: serde_json::Value = r2.json().await.unwrap();

        assert_eq!(p1["id"], p2["id"], "Idempotent calls must return the same payment ID");
    }

    #[tokio::test]
    async fn test_refund_cannot_exceed_original_amount() {
        let c = client();
        let id = uid();
        let admin = login_admin(&c).await;
        let student = create_user(&c, &format!("rfnd_{}", id), &format!("rfnd{}@m.edu", id), "student").await;

        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 30.0}]
            }))
            .send().await.expect("Backend must be reachable");
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        // Charge
        let charge_resp = c.post(&format!("{}/api/payments/", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "order_id": order_id, "idempotency_key": format!("chg-{}", id),
                "payment_method": "cash", "amount": 30.0, "transaction_type": "charge"
            }))
            .send().await.expect("Backend must be reachable");
        let charge: serde_json::Value = charge_resp.json().await.unwrap();
        let payment_id = charge["id"].as_str().unwrap().to_string();

        // Refund more than original — must be rejected
        let refund_resp = c.post(&format!("{}/api/payments/refund", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({
                "original_payment_id": payment_id,
                "idempotency_key": format!("rfnd-{}", id),
                "amount": 999.0,
                "reason": "Over-refund test"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(refund_resp.status(), 400, "Refund exceeding original must be rejected");
    }

    // ===== DOWNLOAD ARTIFACT CONTRACT =====
    // Verify that the download endpoint returns the native file type (not ZIP)
    // and includes watermark evidence in the response headers.

    #[tokio::test]
    async fn test_download_returns_native_content_type_with_watermark() {
        let c = client();
        let id = uid();

        // Create a student and a submission
        let student = create_user(&c, &format!("dl_{}", id), &format!("dl{}@m.edu", id), "student").await;

        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "title": "Download Test Paper",
                "summary": "Testing watermark contract.",
                "submission_type": "journal_article"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let sub: serde_json::Value = resp.json().await.unwrap();
        let sub_id = sub["id"].as_str().expect("submission id");

        // Submit a version with a minimal valid PDF (magic bytes %PDF)
        let fake_pdf = b"%PDF-1.4 fake content for watermark test";
        let b64_pdf = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, fake_pdf);

        let resp = c.post(&format!("{}/api/submissions/{}/versions", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "file_name": "test_paper.pdf",
                "file_data": b64_pdf
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Version upload must succeed");

        // Download the version
        let resp = c.get(&format!("{}/api/submissions/{}/versions/1/download", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "Download must succeed");

        // Verify content type is native PDF, NOT application/zip
        let content_type = resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("application/pdf"),
            "Download must return native PDF content type, got: {}",
            content_type
        );
        assert!(
            !content_type.contains("zip"),
            "Download must NOT return ZIP content type, got: {}",
            content_type
        );

        // Verify watermark headers are present
        let wm_header = resp.headers().get("x-watermark");
        assert!(wm_header.is_some(), "X-Watermark header must be present");
        let wm_text = wm_header.unwrap().to_str().unwrap();
        assert!(
            wm_text.contains("Downloaded by:"),
            "Watermark must contain requester identity, got: {}",
            wm_text
        );

        let wm_hash_header = resp.headers().get("x-watermark-hash");
        assert!(wm_hash_header.is_some(), "X-Watermark-Hash header must be present");
        let wm_hash = wm_hash_header.unwrap().to_str().unwrap();
        assert_eq!(wm_hash.len(), 64, "Watermark hash must be SHA-256 (64 hex chars)");

        // Verify the response body is NOT a ZIP (does not start with PK\x03\x04)
        let body = resp.bytes().await.unwrap();
        assert!(body.len() > 0, "Response body must not be empty");
        assert!(
            !(body.len() >= 4 && body[0] == 0x50 && body[1] == 0x4B && body[2] == 0x03 && body[3] == 0x04),
            "Response body must NOT be a ZIP archive"
        );
        // Verify it starts with %PDF (watermarked PDF still starts with PDF magic)
        assert!(
            body.starts_with(b"%PDF"),
            "Watermarked PDF must still start with %PDF magic bytes"
        );
    }

    #[tokio::test]
    async fn test_download_png_returns_native_png_not_zip() {
        let c = client();
        let id = uid();

        let student = create_user(&c, &format!("dlp_{}", id), &format!("dlp{}@m.edu", id), "student").await;

        let resp = c.post(&format!("{}/api/submissions", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "title": "PNG Download Test",
                "submission_type": "thesis"
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let sub: serde_json::Value = resp.json().await.unwrap();
        let sub_id = sub["id"].as_str().expect("submission id");

        // Minimal valid PNG: 8-byte signature + IHDR chunk (minimal) + IEND chunk
        let mut fake_png: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
        // IHDR chunk (13 bytes data): 1x1, 8-bit RGB
        let ihdr_data: [u8; 13] = [0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0];
        let ihdr_type = b"IHDR";
        let mut ihdr_crc_input = Vec::new();
        ihdr_crc_input.extend_from_slice(ihdr_type);
        ihdr_crc_input.extend_from_slice(&ihdr_data);
        let ihdr_crc = crc32(&ihdr_crc_input);
        fake_png.extend_from_slice(&(13u32).to_be_bytes()); // length
        fake_png.extend_from_slice(ihdr_type);
        fake_png.extend_from_slice(&ihdr_data);
        fake_png.extend_from_slice(&ihdr_crc.to_be_bytes());
        // IEND chunk
        let iend_type = b"IEND";
        let iend_crc = crc32(iend_type);
        fake_png.extend_from_slice(&(0u32).to_be_bytes());
        fake_png.extend_from_slice(iend_type);
        fake_png.extend_from_slice(&iend_crc.to_be_bytes());

        let b64_png = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &fake_png);

        let resp = c.post(&format!("{}/api/submissions/{}/versions", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "file_name": "figure.png",
                "file_data": b64_png
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "PNG version upload must succeed");

        let resp = c.get(&format!("{}/api/submissions/{}/versions/1/download", backend_url(), sub_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200, "PNG download must succeed");

        let content_type = resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("image/png"),
            "Download must return image/png, got: {}",
            content_type
        );

        let body = resp.bytes().await.unwrap();
        // PNG signature must still be present (watermark inserted AFTER signature)
        assert!(
            body.len() >= 8 && body[0] == 0x89 && body[1] == 0x50 && body[2] == 0x4E && body[3] == 0x47,
            "Watermarked PNG must still have PNG signature"
        );
    }

    /// Simple CRC-32 for PNG test construction
    fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; } else { crc >>= 1; }
            }
        }
        !crc
    }

    // ===== STAFF ORDERS LIST: all orders, not just /my =====

    #[tokio::test]
    async fn test_staff_list_orders_returns_all_users_orders() {
        let c = client();
        let id = uid();

        // Create two students, each creates an order
        let student_a = create_user(&c, &format!("olA_{}", id), &format!("olA{}@m.edu", id), "student").await;
        let student_b = create_user(&c, &format!("olB_{}", id), &format!("olB{}@m.edu", id), "student").await;

        c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_a.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal A", "quantity": 1, "unit_price": 5.0}]
            }))
            .send().await.expect("Backend must be reachable");

        c.post(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", student_b.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal B", "quantity": 1, "unit_price": 5.0}]
            }))
            .send().await.expect("Backend must be reachable");

        // Admin fetches /api/orders (all orders)
        let admin = login_admin(&c).await;
        let resp = c.get(&format!("{}/api/orders", backend_url()))
            .header("Authorization", format!("Bearer {}", admin.token))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 200);
        let all_orders: Vec<serde_json::Value> = resp.json().await.unwrap();

        // Must contain orders from both students
        let user_ids: std::collections::HashSet<&str> = all_orders.iter()
            .filter_map(|o| o["user_id"].as_str())
            .collect();
        assert!(
            user_ids.len() >= 2,
            "Admin /api/orders must return orders from multiple users, found {} distinct user_ids",
            user_ids.len()
        );
    }

    // ===== REVIEW FOLLOW-UP RULES =====

    #[tokio::test]
    async fn test_review_followup_only_one_allowed() {
        let c = client();
        let id = uid();

        // Create student + order (mark as delivered so review is allowed)
        let student = create_user(&c, &format!("rfu_{}", id), &format!("rfu{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        // Mark order as delivered (admin)
        let admin = login_admin(&c).await;
        c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.expect("Backend must be reachable");

        // Create initial review
        let rev_resp = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "order_id": order_id, "rating": 4,
                "title": "Good journal", "body": "Really enjoyed it."
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(rev_resp.status(), 200, "Initial review must succeed");
        let rev: serde_json::Value = rev_resp.json().await.unwrap();
        let review_id = rev["id"].as_str().unwrap().to_string();

        // First follow-up — must succeed
        let fu1 = c.post(&format!("{}/api/reviews/followup", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "parent_review_id": review_id, "rating": 5,
                "title": "Update", "body": "Even better on second read."
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(fu1.status(), 200, "First follow-up must succeed");

        // Second follow-up on same review — must be rejected (409 Conflict)
        let fu2 = c.post(&format!("{}/api/reviews/followup", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "parent_review_id": review_id, "rating": 3,
                "title": "Another update", "body": "Changed my mind again."
            }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(fu2.status(), 409, "Second follow-up on same review must be rejected with 409");
    }

    #[tokio::test]
    async fn test_review_followup_on_followup_rejected() {
        let c = client();
        let id = uid();

        let student = create_user(&c, &format!("rfuf_{}", id), &format!("rfuf{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        let admin = login_admin(&c).await;
        c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.expect("Backend must be reachable");

        let rev_resp = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": order_id, "rating": 4, "title": "Good", "body": "Nice." }))
            .send().await.expect("Backend must be reachable");
        let rev: serde_json::Value = rev_resp.json().await.unwrap();
        let review_id = rev["id"].as_str().unwrap().to_string();

        let fu_resp = c.post(&format!("{}/api/reviews/followup", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "parent_review_id": review_id, "rating": 5, "title": "Update", "body": "Better." }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(fu_resp.status(), 200);
        let fu: serde_json::Value = fu_resp.json().await.unwrap();
        let followup_id = fu["id"].as_str().unwrap().to_string();

        // Follow-up on a follow-up — must be rejected (400)
        let fu2 = c.post(&format!("{}/api/reviews/followup", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "parent_review_id": followup_id, "rating": 3, "title": "Meta", "body": "Nesting." }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(fu2.status(), 400, "Follow-up on a follow-up must be rejected with 400");
    }

    #[tokio::test]
    async fn test_review_image_upload_validates_file_type() {
        let c = client();
        let id = uid();

        let student = create_user(&c, &format!("rimg_{}", id), &format!("rimg{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        let admin = login_admin(&c).await;
        c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.expect("Backend must be reachable");

        let rev_resp = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": order_id, "rating": 4, "title": "Good", "body": "Nice." }))
            .send().await.expect("Backend must be reachable");
        let rev: serde_json::Value = rev_resp.json().await.unwrap();
        let review_id = rev["id"].as_str().unwrap().to_string();

        // Upload a valid PNG (magic bytes 0x89 PNG)
        let valid_png: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D];
        let b64_png = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &valid_png);
        let resp = c.post(&format!("{}/api/reviews/{}/images", backend_url(), review_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "file_name": "photo.png", "file_data": b64_png }))
            .send().await.expect("Backend must be reachable");
        assert!(resp.status() == 201 || resp.status() == 200, "Valid PNG upload must succeed, got {}", resp.status());

        // Upload a PDF disguised as PNG — must be rejected (415)
        let fake_png = b"%PDF-1.4 not a real png";
        let b64_fake = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, fake_png);
        let resp = c.post(&format!("{}/api/reviews/{}/images", backend_url(), review_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "file_name": "fake.png", "file_data": b64_fake }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 415, "PDF disguised as PNG must be rejected with 415");
    }

    #[tokio::test]
    async fn test_review_image_max_6_enforced() {
        let c = client();
        let id = uid();

        let student = create_user(&c, &format!("rmax_{}", id), &format!("rmax{}@m.edu", id), "student").await;
        let order_resp = c.post(&format!("{}/api/orders/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({
                "subscription_period": "monthly",
                "line_items": [{"publication_title": "Journal", "quantity": 1, "unit_price": 10.0}]
            }))
            .send().await.expect("Backend must be reachable");
        let order_body: serde_json::Value = order_resp.json().await.unwrap();
        let order_id = order_body["order"]["id"].as_str().unwrap().to_string();

        let admin = login_admin(&c).await;
        c.put(&format!("{}/api/orders/{}/status", backend_url(), order_id))
            .header("Authorization", format!("Bearer {}", admin.token))
            .json(&json!({ "status": "delivered" }))
            .send().await.expect("Backend must be reachable");

        let rev_resp = c.post(&format!("{}/api/reviews/", backend_url()))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "order_id": order_id, "rating": 4, "title": "Good", "body": "Nice." }))
            .send().await.expect("Backend must be reachable");
        let rev: serde_json::Value = rev_resp.json().await.unwrap();
        let review_id = rev["id"].as_str().unwrap().to_string();

        // Valid minimal PNG bytes
        let valid_png: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D];
        let b64_png = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &valid_png);

        // Upload 6 images — all must succeed
        for i in 0..6 {
            let resp = c.post(&format!("{}/api/reviews/{}/images", backend_url(), review_id))
                .header("Authorization", format!("Bearer {}", student.token))
                .json(&json!({ "file_name": format!("photo{}.png", i), "file_data": b64_png }))
                .send().await.expect("Backend must be reachable");
            assert!(resp.status() == 201 || resp.status() == 200, "Image {} upload must succeed, got {}", i, resp.status());
        }

        // 7th image — must be rejected (422)
        let resp = c.post(&format!("{}/api/reviews/{}/images", backend_url(), review_id))
            .header("Authorization", format!("Bearer {}", student.token))
            .json(&json!({ "file_name": "photo7.png", "file_data": b64_png }))
            .send().await.expect("Backend must be reachable");
        assert_eq!(resp.status(), 422, "7th image must be rejected with 422");
    }
}
