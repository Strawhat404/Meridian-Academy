# Test Coverage Audit

## Project Type Detection
- Declared in `repo/README.md`: **Full-Stack Web Application (Rust backend + Rust/WASM frontend)**.
- Effective strict audit type: **fullstack**.

## Backend Endpoint Inventory
Source basis:
- Route mounts: `repo/backend/src/main.rs`.
- Route handlers: `repo/backend/src/routes/*.rs`.

Total unique endpoints (`METHOD + resolved PATH`): **81**.

## API Test Mapping Table
| Endpoint | Covered | Test Type | Test Files | Evidence (file:test_fn) |
|---|---|---|---|---|
| DELETE /api/content/sensitive-words/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_sensitive_words_admin_add_and_remove |
| DELETE /api/users/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_academic_staff_cannot_deactivate_users; repo/API_tests/src/lib.rs:test_deactivated_user_cannot_create_case; repo/API_tests/src/lib.rs:test_deactivated_user_cannot_create_order |
| DELETE /api/users/addresses/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_address_create_set_default_delete; repo/API_tests/src/security.rs:sec_cannot_delete_other_users_address |
| GET /api/admin/audit-log | yes | true no-mock HTTP (static inference) | repo/API_tests/src/security.rs | repo/API_tests/src/security.rs:sec_audit_log_requires_auth |
| GET /api/admin/audit-logs | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_audit_log_plural_endpoint_admin_only |
| GET /api/admin/dashboard | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_admin_dashboard_returns_expected_keys; repo/API_tests/src/flows.rs:e2e_dashboard_denied_for_instructor; repo/API_tests/src/flows.rs:e2e_dashboard_denied_for_student |
| GET /api/admin/settings | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/flows.rs:e2e_system_settings_exposes_constants; repo/API_tests/src/security.rs:sec_admin_settings_requires_auth; repo/API_tests/src/security.rs:sec_admin_settings_student_denied |
| GET /api/auth/export-my-data | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_export_includes_submitted_content_and_order; repo/API_tests/src/flows.rs:e2e_export_user_profile_does_not_contain_password_hash; repo/API_tests/src/lib.rs:test_export_my_data_returns_scoped_data |
| GET /api/auth/me | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_bearer_with_garbage_token_rejected; repo/API_tests/src/flows.rs:e2e_empty_authorization_rejected; repo/API_tests/src/flows.rs:e2e_malformed_bearer_rejected |
| GET /api/cases | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_list_cases_admin_sees_all; repo/API_tests/src/flows.rs:e2e_list_cases_student_sees_only_own |
| GET /api/cases/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_admin_case_status_transitions_through_arbitration; repo/API_tests/src/flows.rs:e2e_assign_case_to_staff; repo/API_tests/src/flows.rs:e2e_student_case_happy_path_with_comments |
| GET /api/cases/:param/comments | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_student_case_happy_path_with_comments |
| GET /api/cases/my | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/flows.rs:e2e_student_case_happy_path_with_comments; repo/API_tests/src/security.rs:sec_idor_cannot_list_other_user_cases_via_my_cases |
| GET /api/content/sensitive-words | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_sensitive_words_admin_add_and_remove; repo/API_tests/src/lib.rs:test_student_cannot_manage_sensitive_words |
| GET /api/orders | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_staff_list_orders_returns_all_users_orders; repo/API_tests/src/security.rs:sec_student_cannot_see_all_orders |
| GET /api/orders/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_clear_flag_clears_order_flag; repo/API_tests/src/lib.rs:test_idor_user_b_cannot_access_user_a_order; repo/API_tests/src/flows.rs:e2e_list_flagged_orders_admin_can_see |
| GET /api/orders/:param/fulfillment | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_fulfillment_list_foreign_order_denied; repo/API_tests/src/flows.rs:e2e_fulfillment_list_owner_can_see |
| GET /api/orders/:param/reconciliation | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_reconciliation_delivery_updates_pending_record; repo/API_tests/src/lib.rs:test_reconciliation_manual_update_status_transitions; repo/API_tests/src/lib.rs:test_reconciliation_non_delivery_event_creates_pending_record |
| GET /api/orders/flagged | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_list_flagged_orders_admin_can_see; repo/API_tests/src/flows.rs:e2e_list_flagged_orders_student_denied |
| GET /api/orders/my | yes | true no-mock HTTP (static inference) | repo/API_tests/src/security.rs | repo/API_tests/src/security.rs:sec_idor_my_orders_scoped |
| GET /api/payments/abnormal-flags | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_abnormal_flag_can_be_cleared_by_admin; repo/API_tests/src/flows.rs:e2e_abnormal_flags_denied_for_student; repo/API_tests/src/flows.rs:e2e_high_quantity_order_is_flagged_and_listed |
| GET /api/payments/order/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_payment_list_non_owner_non_staff_forbidden; repo/API_tests/src/flows.rs:e2e_payment_list_order_owner_can_see_their_payments |
| GET /api/payments/reconciliation-report | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/flows.rs:e2e_payment_reconciliation_report_admin_only; repo/API_tests/src/security.rs:sec_reconciliation_report_requires_auth |
| GET /api/reviews | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_idor_review_list_scoped_to_user |
| GET /api/reviews/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/security.rs:sec_idor_cannot_access_other_user_review_via_direct_get; repo/API_tests/src/flows.rs:e2e_student_orders_then_reviews_delivered_item |
| GET /api/reviews/my | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_student_orders_then_reviews_delivered_item |
| GET /api/submissions | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_list_submissions_admin_sees_all; repo/API_tests/src/flows.rs:e2e_list_submissions_student_sees_only_own |
| GET /api/submissions/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/security.rs:sec_idor_nonexistent_submission_returns_404; repo/API_tests/src/flows.rs:e2e_approve_blocked_submission; repo/API_tests/src/flows.rs:e2e_content_reject_by_staff |
| GET /api/submissions/:param/versions | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_idor_user_b_cannot_access_user_a_submission |
| GET /api/submissions/:param/versions/:param/download | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_download_png_returns_native_png_not_zip; repo/API_tests/src/lib.rs:test_download_returns_native_content_type_with_watermark; repo/API_tests/src/security.rs:sec_download_nonexistent_version_not_found |
| GET /api/submissions/my | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_my_submissions_only_returns_own |
| GET /api/submissions/templates | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_templates_endpoint_returns_templates |
| GET /api/users | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_student_cannot_list_users; repo/API_tests/src/security.rs:sec_list_users_requires_auth |
| GET /api/users/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_address_create_set_default_delete; repo/API_tests/src/security.rs:sec_cannot_delete_other_users_address; repo/API_tests/src/flows.rs:e2e_mark_notification_read |
| GET /api/users/addresses | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_address_create_set_default_delete; repo/API_tests/src/security.rs:sec_cannot_delete_other_users_address |
| GET /api/users/notifications | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_mark_notification_read; repo/API_tests/src/flows.rs:e2e_notifications_generated_on_order_create |
| GET /health | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/flows.rs:e2e_health_no_auth_required; repo/API_tests/src/lib.rs:test_health_check; repo/API_tests/src/security.rs:sec_health_does_not_leak_db_details |
| POST /api/admin/cleanup-soft-deleted | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_cleanup_soft_deleted_admin_only; repo/API_tests/src/security.rs:sec_cleanup_soft_deleted_requires_auth |
| POST /api/auth/cancel-deletion | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_soft_delete_and_cancel; repo/API_tests/src/security.rs:sec_cancel_deletion_requires_auth |
| POST /api/auth/change-password | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_change_password_success; repo/API_tests/src/flows.rs:e2e_change_password_unauthenticated_rejected; repo/API_tests/src/flows.rs:e2e_change_password_wrong_current_rejected |
| POST /api/auth/generate-reset-token | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_reset_token_cannot_be_reused; repo/API_tests/src/lib.rs:test_reset_token_generate_and_use; repo/API_tests/src/lib.rs:test_student_cannot_generate_reset_token |
| POST /api/auth/login | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_change_password_success; repo/API_tests/src/flows.rs:login_admin; repo/API_tests/src/lib.rs:login_admin |
| POST /api/auth/logout | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_logout_invalidates_session; repo/API_tests/src/security.rs:sec_logout_does_not_invalidate_other_user_sessions; repo/API_tests/src/security.rs:sec_logout_requires_auth |
| POST /api/auth/provision | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:create_user; repo/API_tests/src/flows.rs:e2e_provision_duplicate_username_rejected; repo/API_tests/src/flows.rs:e2e_provision_invalid_role_rejected |
| POST /api/auth/request-deletion | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_soft_delete_and_cancel; repo/API_tests/src/security.rs:sec_request_deletion_requires_auth |
| POST /api/auth/use-reset-token | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_reset_token_cannot_be_reused; repo/API_tests/src/lib.rs:test_reset_token_generate_and_use; repo/API_tests/src/security.rs:sec_use_reset_token_no_auth_needed |
| POST /api/cases | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_admin_case_status_transitions_through_arbitration; repo/API_tests/src/flows.rs:e2e_assign_case_to_staff; repo/API_tests/src/flows.rs:e2e_case_invalid_transition_rejected |
| POST /api/cases/:param/comments | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_student_case_happy_path_with_comments; repo/API_tests/src/lib.rs:test_case_comment_owner_allowed; repo/API_tests/src/lib.rs:test_case_comment_requires_involvement |
| POST /api/content/check | yes | true no-mock HTTP (static inference) | repo/API_tests/src/security.rs | repo/API_tests/src/security.rs:sec_content_check_as_admin_returns_result; repo/API_tests/src/security.rs:sec_content_check_requires_privileged |
| POST /api/content/items/:param/approve | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_content_approve_requires_reviewer_role; repo/API_tests/src/flows.rs:e2e_submission_lifecycle_draft_to_published |
| POST /api/content/items/:param/publish | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_submission_lifecycle_draft_to_published |
| POST /api/content/items/:param/reject | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_content_reject_by_staff; repo/API_tests/src/flows.rs:e2e_content_reject_student_denied |
| POST /api/content/items/:param/request-revision | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_submission_rejected_then_revision_requested |
| POST /api/content/items/:param/rollback/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_content_rollback_version; repo/API_tests/src/flows.rs:e2e_content_rollback_nonexistent_version_fails |
| POST /api/content/items/:param/submit | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_content_reject_by_staff; repo/API_tests/src/flows.rs:e2e_submission_lifecycle_draft_to_published; repo/API_tests/src/flows.rs:e2e_submission_rejected_then_revision_requested |
| POST /api/content/sensitive-words | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_sensitive_words_admin_add_and_remove; repo/API_tests/src/flows.rs:e2e_sensitive_words_invalid_action_rejected |
| POST /api/orders | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_abnormal_flag_can_be_cleared_by_admin; repo/API_tests/src/flows.rs:e2e_admin_case_status_transitions_through_arbitration; repo/API_tests/src/flows.rs:e2e_assign_case_to_staff |
| POST /api/orders/clear-flag | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_clear_flag_clears_order_flag; repo/API_tests/src/flows.rs:e2e_clear_flag_student_denied |
| POST /api/orders/fulfillment | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_fulfillment_invalid_event_type_rejected; repo/API_tests/src/flows.rs:e2e_fulfillment_list_owner_can_see; repo/API_tests/src/lib.rs:test_fulfillment_event_requires_reason |
| POST /api/orders/merge | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_merge_cross_user_rejected; repo/API_tests/src/flows.rs:e2e_merge_orders_requires_minimum_two; repo/API_tests/src/lib.rs:test_student_cannot_merge_orders |
| POST /api/orders/split | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_student_cannot_split_order; repo/API_tests/src/security.rs:sec_instructor_cannot_split_orders |
| POST /api/payments | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_payment_list_order_owner_can_see_their_payments; repo/API_tests/src/lib.rs:test_payment_idempotency_no_double_charge; repo/API_tests/src/lib.rs:test_refund_cannot_exceed_original_amount |
| POST /api/payments/abnormal-flags/:param/clear | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_abnormal_flag_can_be_cleared_by_admin |
| POST /api/payments/refund | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_refund_cannot_exceed_original_amount |
| POST /api/reviews | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_duplicate_review_on_same_order_conflict; repo/API_tests/src/flows.rs:e2e_review_on_undelivered_order_rejected; repo/API_tests/src/flows.rs:e2e_review_rating_six_rejected |
| POST /api/reviews/:param/images | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_review_image_max_6_enforced; repo/API_tests/src/lib.rs:test_review_image_upload_validates_file_type |
| POST /api/reviews/followup | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_followup_creation_requires_authentication; repo/API_tests/src/lib.rs:test_review_followup_on_followup_rejected; repo/API_tests/src/lib.rs:test_review_followup_only_one_allowed |
| POST /api/submissions | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_approve_blocked_submission; repo/API_tests/src/flows.rs:e2e_content_approve_requires_reviewer_role; repo/API_tests/src/flows.rs:e2e_content_reject_by_staff |
| POST /api/submissions/:param/approve | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_approve_blocked_submission; repo/API_tests/src/flows.rs:e2e_approve_blocked_student_denied |
| POST /api/submissions/:param/versions | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/lib.rs | repo/API_tests/src/flows.rs:e2e_content_rollback_version; repo/API_tests/src/lib.rs:test_download_png_returns_native_png_not_zip; repo/API_tests/src/lib.rs:test_download_returns_native_content_type_with_watermark |
| POST /api/users/addresses | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_address_create_set_default_delete; repo/API_tests/src/lib.rs:test_order_create_foreign_address_rejected; repo/API_tests/src/security.rs:sec_cannot_delete_other_users_address |
| PUT /api/cases/:param/assign | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_assign_case_to_staff; repo/API_tests/src/flows.rs:e2e_assign_case_student_denied |
| PUT /api/cases/:param/status | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_admin_case_status_transitions_through_arbitration; repo/API_tests/src/flows.rs:e2e_case_invalid_transition_rejected |
| PUT /api/orders/:param/status | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_duplicate_review_on_same_order_conflict; repo/API_tests/src/flows.rs:e2e_order_status_invalid_rejected_by_admin; repo/API_tests/src/flows.rs:e2e_review_rating_six_rejected |
| PUT /api/orders/reconciliation/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_reconciliation_manual_update_status_transitions; repo/API_tests/src/lib.rs:test_student_cannot_update_reconciliation |
| PUT /api/submissions/:param | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_approve_blocked_submission; repo/API_tests/src/flows.rs:e2e_update_submission_non_owner_denied; repo/API_tests/src/flows.rs:e2e_update_submission_owner_can_edit_title |
| PUT /api/users/:param/role | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs, repo/API_tests/src/security.rs | repo/API_tests/src/lib.rs:test_role_change_invalidates_session; repo/API_tests/src/security.rs:sec_staff_cannot_change_user_role |
| PUT /api/users/addresses/default | yes | true no-mock HTTP (static inference) | repo/API_tests/src/lib.rs | repo/API_tests/src/lib.rs:test_address_create_set_default_delete; repo/API_tests/src/lib.rs:test_set_default_address_invalid_id_returns_not_found |
| PUT /api/users/notification-prefs | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_notification_prefs_update_persists |
| PUT /api/users/notifications/:param/read | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs | repo/API_tests/src/flows.rs:e2e_mark_notification_read; repo/API_tests/src/flows.rs:e2e_mark_notification_read_requires_auth |
| PUT /api/users/profile | yes | true no-mock HTTP (static inference) | repo/API_tests/src/flows.rs, repo/API_tests/src/security.rs | repo/API_tests/src/flows.rs:e2e_profile_update_changes_are_reflected_in_me; repo/API_tests/src/security.rs:sec_update_other_user_profile_forbidden |

## API Test Classification
1. True No-Mock HTTP
- `repo/API_tests/src/lib.rs` (`#[tokio::test]` = 64)
- `repo/API_tests/src/flows.rs` (`#[tokio::test]` = 70)
- `repo/API_tests/src/security.rs` (`#[tokio::test]` = 33)
- Evidence: `reqwest::Client` + direct HTTP calls to backend URL.

2. HTTP with Mocking
- **None detected (static scan)**.

3. Non-HTTP (unit/integration without HTTP)
- **None detected** in API test crate.

## Mock Detection
- Result: **No mocking/stubbing evidence found** in API tests.
- Scan scope: `repo/API_tests`, `repo/unit_tests`, `repo/frontend_tests`.
- Patterns checked: `jest.mock`, `vi.mock`, `sinon.stub`, `mockall`, `mockito`, mock symbol patterns.

## Coverage Summary
- Total endpoints: **81**
- Endpoints with HTTP tests: **81**
- Endpoints with TRUE no-mock HTTP tests (static inference): **81**
- HTTP coverage %: **100.00%**
- True API coverage %: **100.00%**

Additional strict note:
- Unmatched API test call found: `POST /api/auth/register` in `repo/API_tests/src/lib.rs:test_self_registration_disabled`.
- Backend route declaration is `POST /api/auth/provision` (`repo/backend/src/routes/auth.rs:register`).

## Unit Test Summary
### Backend Unit Tests
Test files:
- `repo/unit_tests/src/lib.rs`
- `repo/unit_tests/src/domain_rules.rs`
- `repo/unit_tests/src/security.rs`

Modules covered:
- Domain/model logic (`backend::models::*`), validation, constants, transitions
- Auth/security primitives (bcrypt/JWT behavior, permission rules)

Important backend modules NOT unit tested directly:
- `repo/backend/src/routes/*.rs`
- `repo/backend/src/middleware/auth_guard.rs`
- `repo/backend/src/notifications.rs`
- `repo/backend/src/main.rs`

### Frontend Unit Tests (STRICT REQUIREMENT)
Frontend test files:
- `repo/frontend_tests/src/frontend_validation_tests.rs`
- `repo/frontend_tests/src/frontend_formatting_tests.rs`
- `repo/frontend_tests/src/frontend_nav_tests.rs`
- `repo/frontend_tests/src/frontend_status_tests.rs`
- `repo/frontend_tests/src/frontend_page_integration_tests.rs`
- `repo/frontend_tests/src/ui_logic.rs`
- `repo/frontend_tests/src/contracts.rs`
- `repo/frontend_tests/src/lib.rs`

Framework/tools detected:
- Rust built-in test harness (`#[test]`, `cargo test`)

Components/modules covered:
- Real frontend module imports verified: `frontend::validation`, `frontend::formatting`, `frontend::nav_logic`, `frontend::status_display`
- Page-flow simulation tests present in `frontend_page_integration_tests.rs`

Important frontend components/modules NOT tested directly in browser/runtime:
- `repo/frontend/src/components/layout.rs`
- `repo/frontend/src/components/nav.rs`
- `repo/frontend/src/pages/*.rs` runtime rendering paths
- `repo/frontend/src/services/api.rs`, `repo/frontend/src/services/auth.rs` network/runtime integration behavior

**Mandatory Verdict:** **Frontend unit tests: PRESENT**

Cross-layer observation:
- Backend API tests are exhaustive.
- Frontend has logic + page-flow simulation coverage, but no browser-driven FE↔BE E2E automation.

## API Observability Check
- Overall: **Strong**.
- Endpoint/method/path visibility: clear in tests.
- Request payload visibility: broadly explicit via `.json(...)` and path params.
- Response assertions: status + body field assertions widely present.
- Weak subset: some auth tests primarily assert status code only.

## Tests Check
- Success paths: strong coverage.
- Failure/negative paths: strong coverage (RBAC, unauthenticated, validation, IDOR).
- Edge cases: present (idempotency, follow-up limits, media/file constraints, reconciliation states).
- Assertion depth: mostly meaningful, low superficial ratio.
- Integration boundaries: real HTTP boundary exercised by API test crate.

`run_tests.sh` check:
- Docker-based default execution path: **OK**.
- Local toolchain path exists only when `USE_LOCAL_CARGO=1` is explicitly set.

## End-to-End Expectations (Fullstack)
- Missing dedicated browser FE↔BE E2E suite (e.g., Playwright/Cypress).
- Current compensation: strong API tests + frontend logic/page-flow simulation tests.

## Test Coverage Score (0–100)
- **91/100**

## Score Rationale
- + 81/81 endpoint HTTP coverage with no mock inflation evidence.
- + Strong negative-path and security coverage.
- + Frontend unit and page-flow integration simulation tests are present.
- - No browser/runtime FE↔BE automated E2E coverage.
- - Backend unit tests do not directly isolate route/middleware modules.

## Key Gaps
1. No browser-level FE↔BE end-to-end automation.
2. Middleware/route orchestration lacks direct unit-level tests.
3. Test path drift exists for `POST /api/auth/register` (test targets non-declared endpoint).

## Confidence & Assumptions
- Confidence: **High**.
- Assumptions:
  - No-mock classification is static-inferred from source patterns only.
  - Endpoint resolution uses route attributes + mount prefixes from backend bootstrap.

---

# README Audit

## Target File
- `repo/README.md`: **Present**.

## Hard Gate Evaluation
### Formatting
- PASS

### Startup Instructions (backend/fullstack)
- PASS: includes `docker-compose up`.

### Access Method
- PASS: backend/frontend URLs and ports declared.

### Verification Method
- PASS: includes API checks (`curl`) and UI verification flow.

### Environment Rules (STRICT)
- PASS: no runtime install commands or manual DB setup instructions.

### Demo Credentials (Conditional)
- PASS: auth exists and README includes direct credentials for all declared roles:
  - administrator
  - academic_staff
  - instructor
  - student

## Engineering Quality
- Tech stack clarity: strong.
- Architecture explanation: strong.
- Testing instructions: clear and operational.
- Security/roles/workflow documentation: strong.
- Presentation quality: clean and actionable.

## High Priority Issues
- None.

## Medium Priority Issues
1. `run_tests.sh` permits optional local cargo path (`USE_LOCAL_CARGO=1`), which may reduce reproducibility if enabled manually.

## Low Priority Issues
1. Project-structure tree naming (`Meridian_Academy/`) may differ from actual workspace path naming context.

## Hard Gate Failures
- None.

## README Verdict
- **PASS**
