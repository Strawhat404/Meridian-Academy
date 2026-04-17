//! Contract tests: verify that JSON the backend emits can be round-tripped
//! into the exact DTOs the frontend consumes, covering every model.
//! These are high-value tests because a serialization mismatch between
//! backend and frontend is silent at compile time but fatal at runtime.

#[cfg(test)]
mod tests {
    use backend::models::auth::{Claims, LoginResponse, ResetTokenResponse};
    use backend::models::case::{AfterSalesCase, CaseComment, CaseWithSla};
    use backend::models::content::{ContentCheckResult, SensitiveWord};
    use backend::models::order::{
        FulfillmentEvent, Order, OrderLineItem, OrderWithItems, ReconciliationRecord,
    };
    use backend::models::payment::{AbnormalOrderFlag, Payment, ReconciliationReport};
    use backend::models::review::{Review, ReviewImage};
    use backend::models::submission::{
        SubmissionTemplate, SubmissionVersion, SubmissionVersionResponse,
    };
    use backend::models::user::{NotificationItem, UserAddress, UserResponse};

    // ===== LOGIN RESPONSE =====

    #[test]
    fn test_login_response_serializes_token_and_user() {
        let resp = LoginResponse {
            token: "abc.def.ghi".into(),
            user: UserResponse {
                id: "u1".into(),
                username: "alice".into(),
                email: "a@b.c".into(),
                first_name: "A".into(),
                last_name: "B".into(),
                contact_info: None,
                role: "student".into(),
                is_active: true,
                invoice_title: None,
                notify_submissions: true,
                notify_orders: true,
                notify_reviews: true,
                notify_cases: true,
                created_at: None,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"token\":\"abc.def.ghi\""));
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"role\":\"student\""));
    }

    // ===== CLAIMS ROUND-TRIP =====

    #[test]
    fn test_claims_roundtrip_preserves_session_id() {
        let c = Claims {
            sub: "user-1".into(),
            username: "alice".into(),
            role: "administrator".into(),
            exp: 9999999999,
            iat: 1000000000,
            session_id: "sess-xyz".into(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let decoded: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.sub, "user-1");
        assert_eq!(decoded.session_id, "sess-xyz");
        assert_eq!(decoded.role, "administrator");
    }

    // ===== RESET TOKEN RESPONSE =====

    #[test]
    fn test_reset_token_response_shape() {
        let r = ResetTokenResponse {
            token: "0123456789abcdef".into(),
            expires_at: "04/15/2026, 10:00:00 AM".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"token\":\"0123456789abcdef\""));
        assert!(json.contains("\"expires_at\":\"04/15/2026"));
    }

    // ===== ORDER WITH ITEMS =====

    #[test]
    fn test_order_with_items_round_trips() {
        let owi = OrderWithItems {
            order: Order {
                id: "o".into(), user_id: "u".into(),
                order_number: "ORD-20260415-0042".into(),
                subscription_period: "annual".into(),
                shipping_address_id: Some("a1".into()),
                status: "pending".into(),
                payment_status: "unpaid".into(),
                total_amount: 100.00,
                parent_order_id: None,
                is_flagged: false, flag_reason: None,
                created_at: None, updated_at: None,
            },
            line_items: vec![OrderLineItem {
                id: "li1".into(), order_id: "o".into(),
                publication_title: "Pub".into(),
                series_name: Some("S1".into()),
                quantity: 2, unit_price: 50.0, line_total: 100.0,
            }],
        };
        let json = serde_json::to_string(&owi).unwrap();
        let decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded["order"]["order_number"], "ORD-20260415-0042");
        assert_eq!(decoded["line_items"][0]["publication_title"], "Pub");
    }

    // ===== ORDER LINE ITEM =====

    #[test]
    fn test_order_line_item_round_trips_numeric_fields() {
        let li = OrderLineItem {
            id: "li".into(), order_id: "o".into(),
            publication_title: "T".into(), series_name: None,
            quantity: 7, unit_price: 12.50, line_total: 87.50,
        };
        let json = serde_json::to_string(&li).unwrap();
        let back: OrderLineItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.quantity, 7);
        assert!((back.unit_price - 12.50).abs() < f64::EPSILON);
        assert!((back.line_total - 87.50).abs() < f64::EPSILON);
    }

    // ===== FULFILLMENT EVENT =====

    #[test]
    fn test_fulfillment_event_round_trip() {
        let fe = FulfillmentEvent {
            id: "fe".into(), order_id: "o".into(),
            line_item_id: Some("li".into()),
            event_type: "delay".into(),
            issue_identifier: Some("ISS-01".into()),
            reason: "late delivery".into(),
            expected_date: Some("2026-05-01".into()),
            actual_date: None,
            logged_by: "staff-1".into(),
            created_at: None,
        };
        let json = serde_json::to_string(&fe).unwrap();
        let back: FulfillmentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, "delay");
        assert_eq!(back.issue_identifier.unwrap(), "ISS-01");
        assert_eq!(back.reason, "late delivery");
    }

    // ===== RECONCILIATION RECORD =====

    #[test]
    fn test_reconciliation_record_deserializes_minimal_payload() {
        let json = r#"{
            "id": "rec",
            "order_id": "o",
            "line_item_id": null,
            "issue_identifier": "Vol 12",
            "expected_qty": 5,
            "received_qty": 0,
            "status": "pending",
            "notes": null
        }"#;
        let rec: ReconciliationRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.status, "pending");
        assert_eq!(rec.expected_qty, 5);
        assert!(rec.line_item_id.is_none());
    }

    // ===== PAYMENT =====

    #[test]
    fn test_payment_deserialize_with_all_fields() {
        let json = r#"{
            "id": "p1",
            "order_id": "o",
            "idempotency_key": "idem-1",
            "payment_method": "check",
            "amount": 99.99,
            "transaction_type": "charge",
            "reference_payment_id": null,
            "status": "completed",
            "check_number": "12345",
            "notes": "good",
            "processed_by": "staff-1",
            "created_at": null,
            "updated_at": null
        }"#;
        let p: Payment = serde_json::from_str(json).unwrap();
        assert_eq!(p.idempotency_key, "idem-1");
        assert_eq!(p.payment_method, "check");
        assert!((p.amount - 99.99).abs() < f64::EPSILON);
    }

    #[test]
    fn test_payment_refund_references_original() {
        let p = Payment {
            id: "r1".into(), order_id: "o".into(),
            idempotency_key: "r-idem".into(),
            payment_method: "on_account".into(),
            amount: 50.0, transaction_type: "refund".into(),
            reference_payment_id: Some("original-p".into()),
            status: "refunded".into(),
            check_number: None, notes: None,
            processed_by: None, created_at: None, updated_at: None,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Payment = serde_json::from_str(&json).unwrap();
        assert_eq!(back.reference_payment_id.unwrap(), "original-p");
    }

    // ===== ABNORMAL ORDER FLAG =====

    #[test]
    fn test_abnormal_flag_serialization() {
        let flag = AbnormalOrderFlag {
            id: "f1".into(),
            order_id: Some("o1".into()),
            user_id: Some("u1".into()),
            flag_type: "high_quantity".into(),
            reason: "Ordered 500".into(),
            is_cleared: false,
            cleared_by: None, cleared_at: None, created_at: None,
        };
        let json = serde_json::to_string(&flag).unwrap();
        assert!(json.contains("\"is_cleared\":false"));
        assert!(json.contains("\"flag_type\":\"high_quantity\""));
    }

    // ===== AFTER-SALES CASE =====

    #[test]
    fn test_case_with_sla_deserializes_all_fields() {
        let json = r#"{
            "case": {
                "id": "c1", "order_id": "o1", "reporter_id": "u1",
                "assigned_to": null, "case_type": "refund",
                "subject": "S", "description": "D",
                "status": "submitted", "priority": "medium",
                "submitted_at": null, "first_response_at": null,
                "first_response_due": null, "resolution_target": null,
                "resolved_at": null, "closed_at": null,
                "created_at": null, "updated_at": null
            },
            "first_response_overdue": false,
            "resolution_overdue": false,
            "hours_until_first_response": 45.5,
            "hours_until_resolution": 160.0
        }"#;
        let cws: CaseWithSla = serde_json::from_str(json).unwrap();
        assert_eq!(cws.case.priority, "medium");
        assert_eq!(cws.hours_until_first_response, Some(45.5));
        assert!(!cws.first_response_overdue);
    }

    // ===== CASE COMMENT =====

    #[test]
    fn test_case_comment_deserialize() {
        let json = r#"{
            "id": "cc1", "case_id": "c1", "author_id": "u1",
            "content": "Please process.", "created_at": null
        }"#;
        let comment: CaseComment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.content, "Please process.");
    }

    // ===== REVIEW + REVIEW IMAGE =====

    #[test]
    fn test_review_with_followup_metadata() {
        let r = Review {
            id: "r".into(), order_id: "o".into(),
            line_item_id: None, user_id: "u".into(),
            rating: 5, title: "Excellent".into(),
            body: "Loved it".into(),
            is_followup: true,
            parent_review_id: Some("parent".into()),
            created_at: None, updated_at: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: Review = serde_json::from_str(&json).unwrap();
        assert!(back.is_followup);
        assert_eq!(back.parent_review_id.unwrap(), "parent");
    }

    #[test]
    fn test_review_image_has_size_and_path() {
        let img = ReviewImage {
            id: "i1".into(), review_id: "r1".into(),
            file_name: "photo.png".into(),
            file_path: "uploads/reviews/r1/photo.png".into(),
            file_size: 42_000,
        };
        let json = serde_json::to_string(&img).unwrap();
        assert!(json.contains("\"file_size\":42000"));
        assert!(json.contains("uploads/reviews/r1/photo.png"));
    }

    // ===== SUBMISSION TEMPLATE =====

    #[test]
    fn test_submission_template_deserializes_from_json() {
        let json = r#"{
            "id": "tpl-journal",
            "name": "Journal Article",
            "submission_type": "journal_article",
            "required_fields": ["title", "abstract"],
            "optional_fields": ["funding_source"],
            "description": "Standard template"
        }"#;
        let tpl: SubmissionTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(tpl.name, "Journal Article");
        assert!(tpl.required_fields.contains(&"title".to_string()));
    }

    // ===== SUBMISSION VERSION RESPONSE =====

    #[test]
    fn test_submission_version_response_fields() {
        let sv = SubmissionVersion {
            id: "sv1".into(), submission_id: "sub1".into(),
            version_number: 5, file_name: "v5.pdf".into(),
            file_path: "uploads/sub1/v5.pdf".into(),
            file_size: 1_234_567,
            file_type: "pdf".into(),
            file_hash: "abc123".into(),
            magic_bytes: None,
            form_data: None,
            submitted_at: Some(
                chrono::NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()
                    .and_hms_opt(10, 30, 0).unwrap()
            ),
        };
        let resp = sv.to_response();
        assert_eq!(resp.version_number, 5);
        assert_eq!(resp.file_name, "v5.pdf");
        assert_eq!(resp.file_hash, "abc123");
        assert_eq!(resp.submitted_at.unwrap(), "04/15/2026, 10:30:00 AM");
    }

    #[test]
    fn test_submission_version_response_serializes() {
        let resp = SubmissionVersionResponse {
            id: "sv".into(), version_number: 2,
            file_name: "v2.pdf".into(), file_size: 2048,
            file_type: "pdf".into(), file_hash: "h".into(),
            submitted_at: Some("04/15/2026, 09:00:00 AM".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"version_number\":2"));
        assert!(json.contains("\"file_size\":2048"));
    }

    // ===== USER ADDRESS =====

    #[test]
    fn test_user_address_default_flag() {
        let a = UserAddress {
            id: "a1".into(), user_id: "u1".into(),
            label: "Home".into(),
            street_line1: "1 Main".into(), street_line2: None,
            city: "Springfield".into(), state: "IL".into(), zip_code: "62701".into(),
            is_default: true,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"is_default\":true"));
    }

    // ===== NOTIFICATION ITEM =====

    #[test]
    fn test_notification_with_unread_flag() {
        let n = NotificationItem {
            id: "n1".into(),
            title: "New case".into(),
            message: "Case c1 opened".into(),
            is_read: false,
            created_at: None,
        };
        let json = serde_json::to_string(&n).unwrap();
        assert!(json.contains("\"is_read\":false"));
    }

    // ===== CONTENT CHECK RESULT =====

    #[test]
    fn test_content_check_result_serializes_all_fields() {
        let r = ContentCheckResult {
            is_blocked: true,
            blocked_words: vec!["badword".into()],
            processed_text: "replaced".into(),
            replacements_made: vec![("foo".into(), "bar".into())],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"is_blocked\":true"));
        assert!(json.contains("\"badword\""));
        assert!(json.contains("\"replacements_made\""));
    }

    // ===== SENSITIVE WORD =====

    #[test]
    fn test_sensitive_word_null_replacement_serializes() {
        let sw = SensitiveWord {
            id: "1".into(), word: "bad".into(),
            action: "block".into(),
            replacement: None,
            added_by: "admin".into(),
        };
        let json = serde_json::to_string(&sw).unwrap();
        assert!(json.contains("\"replacement\":null"));
        assert!(json.contains("\"action\":\"block\""));
    }

    // ===== RECONCILIATION REPORT =====

    #[test]
    fn test_reconciliation_report_shape() {
        let rr = ReconciliationReport {
            id: "rr".into(),
            report_date: "2026-04-15".into(),
            expected_balance: 1000.0,
            actual_balance: 950.0,
            discrepancy: -50.0,
            details: Some("Missing 2 payments".into()),
        };
        let json = serde_json::to_string(&rr).unwrap();
        assert!(json.contains("\"discrepancy\":-50.0"));
        assert!(json.contains("\"actual_balance\":950.0"));
    }

    // ===== AfterSalesCase with dates =====

    #[test]
    fn test_case_all_date_fields_roundtrip() {
        let dt = chrono::NaiveDate::from_ymd_opt(2026, 4, 10).unwrap()
            .and_hms_opt(12, 0, 0).unwrap();
        let case = AfterSalesCase {
            id: "c".into(), order_id: "o".into(), reporter_id: "u".into(),
            assigned_to: Some("staff".into()),
            case_type: "refund".into(), subject: "S".into(),
            description: "D".into(), status: "arbitrated".into(),
            priority: "urgent".into(),
            submitted_at: Some(dt), first_response_at: Some(dt),
            first_response_due: Some(dt), resolution_target: Some(dt),
            resolved_at: None, closed_at: None,
            created_at: Some(dt), updated_at: Some(dt),
        };
        let json = serde_json::to_string(&case).unwrap();
        let back: AfterSalesCase = serde_json::from_str(&json).unwrap();
        assert_eq!(back.submitted_at, Some(dt));
        assert_eq!(back.first_response_at, Some(dt));
        assert_eq!(back.priority, "urgent");
    }

    // ===== Negative: invalid JSON shapes =====

    #[test]
    fn test_review_missing_required_field_fails() {
        let bad = r#"{ "id": "x" }"#;
        let r: Result<Review, _> = serde_json::from_str(bad);
        assert!(r.is_err(), "missing required fields must fail to deserialize");
    }

    #[test]
    fn test_order_invalid_total_amount_type_fails() {
        let bad = r#"{
            "id": "x", "user_id": "u", "order_number": "N",
            "subscription_period": "monthly", "shipping_address_id": null,
            "status": "pending", "payment_status": "unpaid",
            "total_amount": "not_a_number", "parent_order_id": null,
            "is_flagged": false, "flag_reason": null,
            "created_at": null, "updated_at": null
        }"#;
        let r: Result<Order, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }

    #[test]
    fn test_payment_invalid_amount_type_fails() {
        let bad = r#"{
            "id": "p", "order_id": "o", "idempotency_key": "i",
            "payment_method": "cash", "amount": "string_not_number",
            "transaction_type": "charge", "reference_payment_id": null,
            "status": "completed", "check_number": null,
            "notes": null, "processed_by": null,
            "created_at": null, "updated_at": null
        }"#;
        let r: Result<Payment, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }

    // ===== Order list with large dataset =====

    #[test]
    fn test_order_list_deserialization_many_orders() {
        let mut items = Vec::new();
        for i in 0..50 {
            items.push(format!(
                concat!(
                    r#"{{"id":"o{0}","user_id":"u{0}","order_number":"ORD-{0}","#,
                    r#""subscription_period":"monthly","shipping_address_id":null,"#,
                    r#""status":"pending","payment_status":"unpaid","#,
                    r#""total_amount":{1},"parent_order_id":null,"#,
                    r#""is_flagged":false,"flag_reason":null,"#,
                    r#""created_at":null,"updated_at":null}}"#,
                ),
                i,
                (i as f64) * 10.0,
            ));
        }
        let json = format!("[{}]", items.join(","));
        let orders: Vec<Order> = serde_json::from_str(&json).unwrap();
        assert_eq!(orders.len(), 50);
        assert!((orders[10].total_amount - 100.0).abs() < f64::EPSILON);
    }
}
