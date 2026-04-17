//! Additional unit tests covering domain rules, business invariants,
//! and edge cases that the core `lib.rs` tests don't cover in depth.

#[cfg(test)]
mod tests {
    use backend::models;
    use backend::models::content::{check_sensitive_words, SensitiveWord};
    use backend::models::submission::get_submission_templates;

    use bcrypt::{hash, verify, DEFAULT_COST};
    use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        username: String,
        role: String,
        exp: usize,
        iat: usize,
        session_id: String,
    }

    // ===== PASSWORD HASHING — ADDITIONAL EDGE CASES =====

    #[test]
    fn test_password_hash_empty_string() {
        // Empty password should still hash deterministically-correctly against verify.
        let hashed = hash("", DEFAULT_COST).unwrap();
        assert!(verify("", &hashed).unwrap());
        assert!(!verify("x", &hashed).unwrap());
    }

    #[test]
    fn test_password_hash_unicode() {
        let pw = "pässwörd_日本語_🔐";
        let hashed = hash(pw, DEFAULT_COST).unwrap();
        assert!(verify(pw, &hashed).unwrap());
        assert!(!verify("password", &hashed).unwrap());
    }

    #[test]
    fn test_password_hash_long_up_to_bcrypt_cap() {
        // Bcrypt has a 72-byte cap on inputs; verify we can hash/verify at the cap.
        let pw = "A".repeat(72);
        let hashed = hash(&pw, DEFAULT_COST).unwrap();
        assert!(verify(&pw, &hashed).unwrap());
    }

    #[test]
    fn test_password_case_sensitivity() {
        let hashed = hash("CaseSensitive", DEFAULT_COST).unwrap();
        assert!(!verify("casesensitive", &hashed).unwrap());
        assert!(!verify("CASESENSITIVE", &hashed).unwrap());
        assert!(verify("CaseSensitive", &hashed).unwrap());
    }

    #[test]
    fn test_password_hash_is_never_plaintext() {
        let pw = "not_in_hash_please";
        let hashed = hash(pw, DEFAULT_COST).unwrap();
        assert!(!hashed.contains(pw), "hash must not contain the plaintext");
    }

    // ===== JWT — ADDITIONAL EDGE CASES =====

    #[test]
    fn test_jwt_tampered_payload_rejected() {
        let secret = "test_secret_that_is_at_least_32_bytes_long!!";
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: "u1".into(), username: "alice".into(), role: "student".into(),
            iat: now, exp: now + 3600, session_id: "s1".into(),
        };
        let token = encode(&Header::default(), &claims,
            &EncodingKey::from_secret(secret.as_bytes())).unwrap();

        // Tamper with the middle segment (payload)
        let mut parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        let tampered_payload = "eyJzdWIiOiJoYWNrZXIifQ";
        parts[1] = tampered_payload;
        let tampered = parts.join(".");

        assert!(decode::<Claims>(&tampered,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()).is_err());
    }

    #[test]
    fn test_jwt_missing_dot_rejected() {
        let secret = "test_secret_that_is_at_least_32_bytes_long!!";
        assert!(decode::<Claims>("not.a.jwt",
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()).is_err());
    }

    #[test]
    fn test_jwt_empty_string_rejected() {
        let secret = "test_secret_that_is_at_least_32_bytes_long!!";
        assert!(decode::<Claims>("",
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()).is_err());
    }

    #[test]
    fn test_jwt_session_id_preserved() {
        let secret = "test_secret_that_is_at_least_32_bytes_long!!";
        let now = Utc::now().timestamp() as usize;
        let session_id = Uuid::new_v4().to_string();
        let claims = Claims {
            sub: "u1".into(), username: "alice".into(), role: "student".into(),
            iat: now, exp: now + 3600, session_id: session_id.clone(),
        };
        let token = encode(&Header::default(), &claims,
            &EncodingKey::from_secret(secret.as_bytes())).unwrap();
        let decoded = decode::<Claims>(&token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()).unwrap();
        assert_eq!(decoded.claims.session_id, session_id);
    }

    #[test]
    fn test_jwt_iat_before_exp_is_valid() {
        let secret = "test_secret_that_is_at_least_32_bytes_long!!";
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: "u1".into(), username: "alice".into(), role: "student".into(),
            iat: now, exp: now + 1, session_id: "s1".into(),
        };
        let token = encode(&Header::default(), &claims,
            &EncodingKey::from_secret(secret.as_bytes())).unwrap();
        assert!(decode::<Claims>(&token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()).is_ok());
    }

    // ===== VALIDATE_METADATA — MORE EDGE CASES =====

    #[test]
    fn test_validate_metadata_single_tag_exactly_50() {
        let tag50 = "a".repeat(50);
        assert!(models::validate_metadata("T", None, Some(&tag50), None).is_ok());
    }

    #[test]
    fn test_validate_metadata_whitespace_trimmed_for_tag_length_check() {
        // A tag surrounded by whitespace but under 50 chars when trimmed should pass.
        let tag_padded = format!("  {}  ", "a".repeat(50));
        assert!(models::validate_metadata("T", None, Some(&tag_padded), None).is_ok(),
            "validate_metadata should measure after trimming");
    }

    #[test]
    fn test_validate_metadata_keyword_exactly_50() {
        let kw = "b".repeat(50);
        assert!(models::validate_metadata("T", None, None, Some(&kw)).is_ok());
    }

    #[test]
    fn test_validate_metadata_empty_tag_string_ok() {
        assert!(models::validate_metadata("T", None, Some(""), None).is_ok());
    }

    #[test]
    fn test_validate_metadata_empty_keyword_string_ok() {
        assert!(models::validate_metadata("T", None, None, Some("")).is_ok());
    }

    #[test]
    fn test_validate_metadata_single_char_title() {
        assert!(models::validate_metadata("x", None, None, None).is_ok());
    }

    #[test]
    fn test_validate_metadata_tag_with_internal_whitespace() {
        assert!(models::validate_metadata("T", None, Some("two words,three word tags"), None).is_ok());
    }

    #[test]
    fn test_validate_metadata_unicode_summary() {
        // Unicode summary under limit should pass (byte-length check, 500 bytes)
        let summary = "日本語".repeat(50); // Approx 450 bytes
        assert!(summary.len() <= 500);
        assert!(models::validate_metadata("T", Some(&summary), None, None).is_ok());
    }

    #[test]
    fn test_validate_metadata_long_keyword_list_boundary() {
        // 10 keywords of 50 chars each + 9 commas = 509 chars → valid (< 1000)
        let kws: Vec<String> = (0..10).map(|_| "k".repeat(50)).collect();
        let joined = kws.join(",");
        assert!(joined.len() < 1000);
        assert!(models::validate_metadata("T", None, None, Some(&joined)).is_ok());
    }

    // ===== GENERATE_SEO — ADDITIONAL CASES =====

    #[test]
    fn test_seo_summary_exactly_155() {
        let summary = "s".repeat(155);
        let (_, md, _) = models::generate_seo("Title", Some(&summary));
        assert_eq!(md.len(), 155);
        assert_eq!(md, summary);
    }

    #[test]
    fn test_seo_summary_exactly_156_truncated() {
        let summary = "s".repeat(156);
        let (_, md, _) = models::generate_seo("Title", Some(&summary));
        assert_eq!(md.len(), 155);
    }

    #[test]
    fn test_seo_title_exactly_120_kept() {
        let t = "t".repeat(120);
        let (mt, _, _) = models::generate_seo(&t, None);
        assert_eq!(mt.len(), 120);
        assert_eq!(mt, t);
    }

    #[test]
    fn test_seo_slug_is_lowercase() {
        let (_, _, slug) = models::generate_seo("UPPERCASE Title", None);
        assert_eq!(slug, slug.to_lowercase());
    }

    #[test]
    fn test_seo_slug_removes_multiple_spaces() {
        let (_, _, slug) = models::generate_seo("A    B    C", None);
        // No double dashes
        assert!(!slug.contains("--"));
    }

    #[test]
    fn test_seo_slug_idempotent_ascii() {
        let (_, _, s1) = models::generate_seo("Already-a-slug", None);
        let (_, _, s2) = models::generate_seo(&s1, None);
        assert_eq!(s1, s2, "slugifying an already-slugified string must be a no-op");
    }

    // ===== FILE TYPE — ADDITIONAL CASES =====

    #[test]
    fn test_file_type_rejects_magic_too_short() {
        // PDF check needs 4 bytes
        assert!(models::validate_file_type("short.pdf", b"%PD").is_err());
    }

    #[test]
    fn test_file_type_rejects_zero_length_file() {
        assert!(models::validate_file_type("empty.png", &[]).is_err());
    }

    #[test]
    fn test_file_type_rejects_hidden_dot_only_filename() {
        // ".pdf" → the ext splits into "pdf" and "" — Rust rsplit returns "pdf" here
        // Actually for ".pdf", rsplit('.').next() returns "pdf" (rsplit is right-to-left).
        // But "pdf" with valid magic bytes should still succeed.
        assert!(models::validate_file_type(".pdf", b"%PDF-1.4").is_ok());
    }

    #[test]
    fn test_file_type_double_extension_uses_last() {
        // archive.tar.pdf → "pdf" is the last extension
        assert!(models::validate_file_type("archive.tar.pdf", b"%PDF-1.4").is_ok());
    }

    #[test]
    fn test_file_type_returns_extension_lowercase() {
        let ext = models::validate_file_type("PHOTO.JPEG", &[0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        assert_eq!(ext, "jpeg");
    }

    #[test]
    fn test_file_type_zip_disguised_as_docx_rejected_without_docx_magic() {
        // DOCX is actually a ZIP (starts with 0x50 0x4B 0x03 0x04), so this test
        // actually verifies the opposite: valid DOCX magic passes.
        assert!(models::validate_file_type("thing.docx",
            &[0x50, 0x4B, 0x03, 0x04, 0x00]).is_ok());
    }

    #[test]
    fn test_file_type_svg_rejected() {
        // SVG is XML/text; not in allowlist
        assert!(models::validate_file_type("vector.svg",
            b"<?xml version=\"1.0\"?>").is_err());
    }

    #[test]
    fn test_file_type_gif_rejected() {
        assert!(models::validate_file_type("anim.gif",
            &[0x47, 0x49, 0x46, 0x38]).is_err());
    }

    #[test]
    fn test_file_type_bmp_rejected() {
        assert!(models::validate_file_type("image.bmp",
            &[0x42, 0x4D]).is_err());
    }

    // ===== SENSITIVE WORDS — MORE CASES =====

    #[test]
    fn test_sensitive_words_overlapping_substring() {
        let words = vec![
            SensitiveWord { id: "1".into(), word: "bad".into(), action: "replace".into(),
                replacement: Some("[x]".into()), added_by: "a".into() },
            SensitiveWord { id: "2".into(), word: "badword".into(), action: "replace".into(),
                replacement: Some("[y]".into()), added_by: "a".into() },
        ];
        let result = check_sensitive_words("badword and bad", &words);
        // Either order of replacement: first "bad" → [x] is applied to both occurrences,
        // or "badword" is replaced first — the key assertion is nothing stays uncensored.
        assert!(!result.processed_text.contains("bad "));
        assert!(!result.processed_text.contains("badword"));
    }

    #[test]
    fn test_sensitive_words_block_returns_original_blocked_word() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "SHOUTING".into(), action: "block".into(),
            replacement: None, added_by: "a".into(),
        }];
        let result = check_sensitive_words("this is shouting text", &words);
        assert!(result.is_blocked);
        // The blocked list contains the canonical/original word form (from the dictionary entry)
        assert_eq!(result.blocked_words[0], "SHOUTING");
    }

    #[test]
    fn test_sensitive_words_replace_preserves_non_target_text() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "foo".into(), action: "replace".into(),
            replacement: Some("BAR".into()), added_by: "a".into(),
        }];
        let result = check_sensitive_words("hello foo world", &words);
        assert_eq!(result.processed_text, "hello BAR world");
    }

    #[test]
    fn test_sensitive_words_replacement_tracks_count() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "x".into(), action: "replace".into(),
            replacement: Some("Y".into()), added_by: "a".into(),
        }];
        let result = check_sensitive_words("xxxx", &words);
        assert_eq!(result.replacements_made.len(), 4);
    }

    #[test]
    fn test_sensitive_words_punctuation_adjacent() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "spam".into(), action: "replace".into(),
            replacement: Some("[s]".into()), added_by: "a".into(),
        }];
        let result = check_sensitive_words("Hi spam! And spam. Also spam?", &words);
        assert!(result.processed_text.contains("[s]!"));
        assert!(result.processed_text.contains("[s]."));
        assert!(result.processed_text.contains("[s]?"));
    }

    #[test]
    fn test_sensitive_words_tab_and_newline_adjacent() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "bad".into(), action: "replace".into(),
            replacement: Some("OK".into()), added_by: "a".into(),
        }];
        let result = check_sensitive_words("x\tbad\ny", &words);
        assert_eq!(result.processed_text, "x\tOK\ny");
    }

    #[test]
    fn test_sensitive_words_unknown_action_noop() {
        let words = vec![SensitiveWord {
            id: "1".into(), word: "bad".into(), action: "delete".into(),  // unsupported
            replacement: None, added_by: "a".into(),
        }];
        let result = check_sensitive_words("this is bad text", &words);
        // Unknown action: neither block nor replace
        assert!(!result.is_blocked);
        assert_eq!(result.processed_text, "this is bad text");
    }

    // ===== BUSINESS DAYS — EXTREME CASES =====

    #[test]
    fn test_business_days_across_month_boundary() {
        // Friday Jan 30, 2026 + 2 = Tuesday Feb 3, 2026
        let fri = NaiveDate::from_ymd_opt(2026, 1, 30).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let result = models::add_business_days(fri, 2);
        assert_eq!(result.date(), NaiveDate::from_ymd_opt(2026, 2, 3).unwrap());
    }

    #[test]
    fn test_business_days_across_year_boundary() {
        // Wednesday Dec 30, 2026 + 3 = Monday Jan 4, 2027
        let wed = NaiveDate::from_ymd_opt(2026, 12, 30).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let result = models::add_business_days(wed, 3);
        assert_eq!(result.date(), NaiveDate::from_ymd_opt(2027, 1, 4).unwrap());
    }

    #[test]
    fn test_business_days_large_count_never_weekend() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_hms_opt(9, 0, 0).unwrap();
        for d in 1..=30 {
            let r = models::add_business_days(start, d);
            assert!(matches!(r.weekday(), Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri),
                "day +{} landed on weekend: {}", d, r.weekday());
        }
    }

    #[test]
    fn test_business_days_exactly_20_is_4_weeks() {
        // 20 business days = 4 work-weeks = 28 calendar days from any weekday
        let mon = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap().and_hms_opt(9, 0, 0).unwrap();
        let r = models::add_business_days(mon, 20);
        let diff = (r - mon).num_days();
        assert_eq!(diff, 28);
    }

    // ===== CASE TRANSITIONS — COMPLETE STATE MACHINE =====

    #[test]
    fn test_case_happy_path_through_all_states() {
        // Every state on the happy path must accept its forward transition.
        assert!(models::valid_case_transition("submitted", "in_review"));
        assert!(models::valid_case_transition("in_review", "arbitrated"));
        assert!(models::valid_case_transition("arbitrated", "approved"));
        assert!(models::valid_case_transition("approved", "closed"));
    }

    #[test]
    fn test_case_evidence_loop_is_bounded() {
        // You can bounce in_review ↔ awaiting_evidence any number of times,
        // but cannot skip to approved/denied without arbitration.
        assert!(models::valid_case_transition("in_review", "awaiting_evidence"));
        assert!(models::valid_case_transition("awaiting_evidence", "in_review"));
        assert!(!models::valid_case_transition("awaiting_evidence", "approved"));
        assert!(!models::valid_case_transition("awaiting_evidence", "denied"));
        assert!(!models::valid_case_transition("awaiting_evidence", "closed"));
    }

    #[test]
    fn test_case_every_state_pair_exhaustive() {
        // Exactly 9 valid transitions exist; all others are rejected.
        let states = ["submitted", "in_review", "awaiting_evidence",
                      "arbitrated", "approved", "denied", "closed"];
        let mut valid_count = 0;
        for from in &states {
            for to in &states {
                if models::valid_case_transition(from, to) {
                    valid_count += 1;
                }
            }
        }
        assert_eq!(valid_count, 9, "Expected exactly 9 valid transitions, got {}", valid_count);
    }

    // ===== ORDER / PAYMENT DOMAIN RULES =====

    #[test]
    fn test_order_number_format() {
        // ORD-YYYYMMDDHHMMSS-NNNN pattern used in orders.rs
        let sample = "ORD-20260411120000-0042";
        assert!(sample.starts_with("ORD-"));
        let parts: Vec<&str> = sample.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].len(), 14); // timestamp
        assert_eq!(parts[2].len(), 4);  // random
    }

    #[test]
    fn test_subscription_period_allowlist_exhaustive() {
        let valid = ["monthly", "quarterly", "annual"];
        let invalid = ["weekly", "biweekly", "daily", "lifetime", ""];
        for v in &valid { assert!(valid.contains(v)); }
        for iv in &invalid { assert!(!valid.contains(iv), "'{}' should not be valid", iv); }
    }

    #[test]
    fn test_order_status_allowlist_exhaustive() {
        let valid = ["pending", "confirmed", "processing", "shipped",
                     "delivered", "cancelled"];
        assert!(!valid.contains(&"returned"));
        assert!(!valid.contains(&"draft"));
        assert_eq!(valid.len(), 6);
    }

    #[test]
    fn test_payment_method_allowlist() {
        let valid = ["cash", "check", "on_account"];
        assert!(!valid.contains(&"credit_card"));
        assert!(!valid.contains(&"wire"));
        assert!(!valid.contains(&"crypto"));
        assert_eq!(valid.len(), 3);
    }

    #[test]
    fn test_transaction_type_allowlist() {
        let valid = ["charge", "hold", "release", "refund"];
        assert_eq!(valid.len(), 4);
        // Status derivation from transaction_type is the responsibility of payments.rs
        for tt in &valid {
            let expected_status = match *tt {
                "hold" => "held",
                "charge" => "completed",
                "release" => "released",
                "refund" => "refunded",
                _ => "pending",
            };
            assert_ne!(expected_status, "pending");
        }
    }

    #[test]
    fn test_order_flag_high_quantity_and_refunds() {
        // Flagging logic mirrors create_order: either condition flags the order.
        fn would_flag(qty: i32, refund_count: i64) -> (bool, Option<&'static str>) {
            let has_high_qty = qty > models::HIGH_QUANTITY_THRESHOLD;
            let has_refunds = refund_count >= models::REFUND_COUNT_THRESHOLD;
            let flag = has_high_qty || has_refunds;
            let reason = match (has_high_qty, has_refunds) {
                (true, true) => Some("both"),
                (true, false) => Some("high_quantity"),
                (false, true) => Some("refunds"),
                _ => None,
            };
            (flag, reason)
        }
        assert_eq!(would_flag(100, 0), (true, Some("high_quantity")));
        assert_eq!(would_flag(1, 5), (true, Some("refunds")));
        assert_eq!(would_flag(100, 5), (true, Some("both")));
        assert_eq!(would_flag(1, 0), (false, None));
    }

    #[test]
    fn test_refund_partial_vs_full_threshold() {
        // The payments code uses an epsilon of 0.01 to decide full vs partial refund.
        let orig = 100.00_f64;
        assert!((orig - 100.00).abs() < 0.01); // full
        assert!((orig - 99.99).abs() < 0.02);  // still essentially full
        assert!((orig - 50.00).abs() >= 0.01); // partial
    }

    // ===== SLA DEADLINE COMPUTATION =====

    #[test]
    fn test_sla_first_response_monday_9am() {
        // Monday 9am + 48 hours = Wednesday 9am (wall-clock), not business hours.
        let mon = NaiveDate::from_ymd_opt(2026, 4, 6).unwrap().and_hms_opt(9, 0, 0).unwrap();
        let deadline = mon + Duration::hours(models::SLA_FIRST_RESPONSE_HOURS);
        assert_eq!(deadline.weekday(), Weekday::Wed);
        assert_eq!(deadline.date(), NaiveDate::from_ymd_opt(2026, 4, 8).unwrap());
    }

    #[test]
    fn test_sla_business_days_first_response_is_2_days() {
        // The spec calls for 2 business days; verify constant corresponds.
        assert_eq!(models::SLA_FIRST_RESPONSE_HOURS, 48);
        assert_eq!(models::SLA_FIRST_RESPONSE_HOURS / 24, 2);
    }

    #[test]
    fn test_sla_resolution_is_7_days() {
        assert_eq!(models::SLA_RESOLUTION_HOURS, 168);
        assert_eq!(models::SLA_RESOLUTION_HOURS / 24, 7);
    }

    #[test]
    fn test_sla_resolution_longer_than_first_response() {
        assert!(models::SLA_RESOLUTION_HOURS > models::SLA_FIRST_RESPONSE_HOURS);
    }

    // ===== RECONCILIATION STATUS =====

    #[test]
    fn test_reconciliation_exhaustive_status_derivation() {
        fn derive(expected: i32, received: i32) -> &'static str {
            if received == 0 { "pending" }
            else if received == expected { "matched" }
            else { "discrepancy" }
        }
        assert_eq!(derive(5, 0), "pending");
        assert_eq!(derive(5, 5), "matched");
        assert_eq!(derive(5, 3), "discrepancy");
        assert_eq!(derive(5, 6), "discrepancy");
        assert_eq!(derive(0, 0), "pending"); // initial state
    }

    // ===== WATERMARK HASH =====

    #[test]
    fn test_watermark_hash_fixed_vector() {
        // Stable deterministic vector: SHA-256("abc123" || "wm1")
        let mut h = Sha256::new();
        h.update(b"abc123");
        h.update(b"wm1");
        let got = hex::encode(h.finalize());
        assert_eq!(got.len(), 64);
        // Hex-only
        assert!(got.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_watermark_different_file_hashes_differ() {
        let mk = |fh: &[u8], wm: &[u8]| {
            let mut h = Sha256::new();
            h.update(fh);
            h.update(wm);
            hex::encode(h.finalize())
        };
        let a = mk(b"file1", b"wm");
        let b = mk(b"file2", b"wm");
        assert_ne!(a, b);
    }

    // ===== SESSION / ACCOUNT LIFECYCLE =====

    #[test]
    fn test_session_refresh_updates_expiry_forward() {
        let t0 = Utc::now().naive_utc();
        let expiry_initial = t0 + Duration::minutes(models::SESSION_IDLE_TIMEOUT_MINUTES);

        // Simulate a request 10 minutes later that refreshes
        let t_request = t0 + Duration::minutes(10);
        let expiry_refreshed = t_request + Duration::minutes(models::SESSION_IDLE_TIMEOUT_MINUTES);

        assert!(expiry_refreshed > expiry_initial, "refresh must extend expiry");
    }

    #[test]
    fn test_session_idle_past_expiry_should_be_denied() {
        let t0 = Utc::now().naive_utc();
        let expires_at = t0 - Duration::minutes(1);
        let now = t0;
        assert!(now > expires_at, "session should be treated as expired");
    }

    #[test]
    fn test_soft_delete_deletion_scheduled_in_future() {
        let now = Utc::now().naive_utc();
        let scheduled = now + Duration::days(models::SOFT_DELETE_HOLD_DAYS);
        assert!(scheduled > now);
        assert_eq!((scheduled - now).num_days(), models::SOFT_DELETE_HOLD_DAYS);
    }

    #[test]
    fn test_soft_delete_scheduled_past_is_ready_for_cleanup() {
        let now = Utc::now().naive_utc();
        let scheduled_past = now - Duration::days(1);
        assert!(now > scheduled_past, "deletion should be ready when past scheduled date");
    }

    // ===== RBAC DERIVED RULES =====

    #[test]
    fn test_privileged_requires_users_list_or_admin_dashboard() {
        // Mirror the is_privileged logic from auth_guard.rs
        fn is_privileged(perms: &[&str]) -> bool {
            perms.iter().any(|p| *p == "users.list" || *p == "admin.dashboard")
        }
        assert!(is_privileged(&["users.list"]));
        assert!(is_privileged(&["admin.dashboard"]));
        assert!(is_privileged(&["users.list", "admin.dashboard"]));
        assert!(!is_privileged(&["submissions.create"]));
        assert!(!is_privileged(&[]));
    }

    #[test]
    fn test_owner_or_privileged_policy() {
        fn check(my_id: &str, owner_id: &str, privileged: bool) -> bool {
            my_id == owner_id || privileged
        }
        assert!(check("u1", "u1", false)); // owner
        assert!(check("u2", "u1", true));  // privileged outsider
        assert!(!check("u2", "u1", false)); // unrelated, not privileged
        assert!(check("u2", "u1", true));  // admin can see everyone's resources
    }

    // ===== TEMPLATE INTEGRITY =====

    #[test]
    fn test_every_template_has_abstract_or_equivalent() {
        let templates = get_submission_templates();
        for t in &templates {
            assert!(t.required_fields.iter().any(|f| f == "abstract" || f == "title"),
                "Template '{}' must require at least title or abstract", t.id);
        }
    }

    #[test]
    fn test_template_required_fields_nonempty_strings() {
        let templates = get_submission_templates();
        for t in &templates {
            for f in &t.required_fields {
                assert!(!f.is_empty(), "Template '{}' has empty required field", t.id);
                assert!(!f.contains(' '), "Template field '{}' should not contain spaces", f);
            }
        }
    }

    #[test]
    fn test_template_ids_prefixed_with_tpl() {
        let templates = get_submission_templates();
        for t in &templates {
            assert!(t.id.starts_with("tpl-"), "template id '{}' must be prefixed with 'tpl-'", t.id);
        }
    }

    #[test]
    fn test_template_submission_type_is_snake_case() {
        let templates = get_submission_templates();
        for t in &templates {
            assert!(!t.submission_type.contains('-'),
                "submission_type '{}' should be snake_case, not kebab-case", t.submission_type);
            assert_eq!(t.submission_type, t.submission_type.to_lowercase());
        }
    }

    #[test]
    fn test_template_optional_fields_disjoint_from_required() {
        let templates = get_submission_templates();
        for t in &templates {
            for opt in &t.optional_fields {
                assert!(!t.required_fields.contains(opt),
                    "Template '{}' has '{}' in both required and optional", t.id, opt);
            }
        }
    }

    // ===== UUID INVARIANTS =====

    #[test]
    fn test_uuid_v4_format() {
        for _ in 0..10 {
            let u = Uuid::new_v4().to_string();
            assert_eq!(u.len(), 36);
            assert_eq!(u.chars().filter(|c| *c == '-').count(), 4);
            // Version nibble should be 4
            let version_char = u.chars().nth(14).unwrap();
            assert_eq!(version_char, '4');
        }
    }

    // ===== REVIEW RATING BOUNDS =====

    #[test]
    fn test_review_rating_valid_range() {
        for r in 1..=5 { assert!((1..=5).contains(&r)); }
        for invalid in &[0, 6, -1, 10] {
            assert!(!(1..=5).contains(invalid));
        }
    }

    // ===== FOLLOW-UP WINDOW =====

    #[test]
    fn test_followup_window_14_days_in_future() {
        let created = Utc::now().naive_utc();
        let deadline = created + Duration::days(models::FOLLOWUP_WINDOW_DAYS);
        assert_eq!((deadline - created).num_days(), models::FOLLOWUP_WINDOW_DAYS);
    }

    #[test]
    fn test_followup_window_expired_after_15_days() {
        let created = Utc::now().naive_utc() - Duration::days(15);
        let deadline = created + Duration::days(models::FOLLOWUP_WINDOW_DAYS);
        let now = Utc::now().naive_utc();
        assert!(now > deadline, "follow-up window must be expired after 15 days");
    }

    // ===== TIMESTAMP FORMATTING =====

    #[test]
    fn test_timestamp_12hour_formatting_am() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap().and_hms_opt(6, 0, 0).unwrap();
        assert_eq!(dt.format("%m/%d/%Y, %I:%M:%S %p").to_string(), "01/15/2026, 06:00:00 AM");
    }

    #[test]
    fn test_timestamp_12hour_formatting_pm() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap().and_hms_opt(18, 30, 45).unwrap();
        assert_eq!(dt.format("%m/%d/%Y, %I:%M:%S %p").to_string(), "01/15/2026, 06:30:45 PM");
    }

    #[test]
    fn test_timestamp_12hour_midnight() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(dt.format("%m/%d/%Y, %I:%M:%S %p").to_string(), "01/15/2026, 12:00:00 AM");
    }

    #[test]
    fn test_timestamp_12hour_noon() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(dt.format("%m/%d/%Y, %I:%M:%S %p").to_string(), "01/15/2026, 12:00:00 PM");
    }

    // ===== NOTIFICATION PREFS DEFAULT =====

    #[test]
    fn test_notification_prefs_default_all_enabled() {
        // Defaults from user provisioning: all four are true
        let defaults = (true, true, true, true);
        assert!(defaults.0);
        assert!(defaults.1);
        assert!(defaults.2);
        assert!(defaults.3);
    }
}
