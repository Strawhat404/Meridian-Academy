//! Tests for `frontend::validation` — real frontend module imports.
//! These exercise the actual validation functions used by frontend form components.

#[cfg(test)]
mod tests {
    use frontend::validation::*;

    // ===== TITLE VALIDATION =====

    #[test]
    fn test_validate_title_empty_rejected() {
        assert!(validate_title("").is_err());
        assert!(validate_title("   ").is_err());
    }

    #[test]
    fn test_validate_title_at_limit() {
        assert!(validate_title(&"a".repeat(120)).is_ok());
    }

    #[test]
    fn test_validate_title_over_limit() {
        assert!(validate_title(&"a".repeat(121)).is_err());
    }

    #[test]
    fn test_validate_title_normal() {
        assert!(validate_title("My Research Paper").is_ok());
    }

    // ===== SUMMARY VALIDATION =====

    #[test]
    fn test_validate_summary_at_limit() {
        assert!(validate_summary(&"s".repeat(500)).is_ok());
    }

    #[test]
    fn test_validate_summary_over_limit() {
        assert!(validate_summary(&"s".repeat(501)).is_err());
    }

    #[test]
    fn test_validate_summary_empty_ok() {
        assert!(validate_summary("").is_ok());
    }

    // ===== RATING VALIDATION =====

    #[test]
    fn test_validate_rating_valid_range() {
        for r in 1..=5 {
            assert!(validate_rating(r).is_ok(), "rating {} should be valid", r);
        }
    }

    #[test]
    fn test_validate_rating_zero_rejected() {
        assert!(validate_rating(0).is_err());
    }

    #[test]
    fn test_validate_rating_six_rejected() {
        assert!(validate_rating(6).is_err());
    }

    #[test]
    fn test_validate_rating_negative_rejected() {
        assert!(validate_rating(-1).is_err());
    }

    // ===== FILE EXTENSION VALIDATION =====

    #[test]
    fn test_validate_file_extension_pdf() {
        assert!(validate_file_extension("paper.pdf").is_ok());
    }

    #[test]
    fn test_validate_file_extension_docx() {
        assert!(validate_file_extension("document.docx").is_ok());
    }

    #[test]
    fn test_validate_file_extension_png() {
        assert!(validate_file_extension("figure.png").is_ok());
    }

    #[test]
    fn test_validate_file_extension_jpg() {
        assert!(validate_file_extension("photo.jpg").is_ok());
    }

    #[test]
    fn test_validate_file_extension_jpeg() {
        assert!(validate_file_extension("photo.jpeg").is_ok());
    }

    #[test]
    fn test_validate_file_extension_case_insensitive() {
        assert!(validate_file_extension("PAPER.PDF").is_ok());
        assert!(validate_file_extension("doc.DOCX").is_ok());
    }

    #[test]
    fn test_validate_file_extension_exe_rejected() {
        assert!(validate_file_extension("malware.exe").is_err());
    }

    #[test]
    fn test_validate_file_extension_txt_rejected() {
        assert!(validate_file_extension("notes.txt").is_err());
    }

    #[test]
    fn test_validate_file_extension_svg_rejected() {
        assert!(validate_file_extension("logo.svg").is_err());
    }

    #[test]
    fn test_validate_file_extension_no_extension() {
        assert!(validate_file_extension("README").is_err());
    }

    // ===== FILE SIZE VALIDATION =====

    #[test]
    fn test_validate_file_size_within_limit() {
        assert!(validate_file_size(1024).is_ok());
        assert!(validate_file_size(MAX_FILE_SIZE).is_ok());
    }

    #[test]
    fn test_validate_file_size_over_limit() {
        assert!(validate_file_size(MAX_FILE_SIZE + 1).is_err());
    }

    #[test]
    fn test_validate_review_image_within_limit() {
        assert!(validate_review_image_size(MAX_REVIEW_IMAGE_SIZE).is_ok());
    }

    #[test]
    fn test_validate_review_image_over_limit() {
        assert!(validate_review_image_size(MAX_REVIEW_IMAGE_SIZE + 1).is_err());
    }

    // ===== SUBSCRIPTION PERIOD =====

    #[test]
    fn test_validate_subscription_period_valid() {
        assert!(validate_subscription_period("monthly").is_ok());
        assert!(validate_subscription_period("quarterly").is_ok());
        assert!(validate_subscription_period("annual").is_ok());
    }

    #[test]
    fn test_validate_subscription_period_invalid() {
        assert!(validate_subscription_period("weekly").is_err());
        assert!(validate_subscription_period("").is_err());
    }

    // ===== CASE TYPE =====

    #[test]
    fn test_validate_case_type_valid() {
        assert!(validate_case_type("return").is_ok());
        assert!(validate_case_type("refund").is_ok());
        assert!(validate_case_type("exchange").is_ok());
    }

    #[test]
    fn test_validate_case_type_invalid() {
        assert!(validate_case_type("complaint").is_err());
    }

    // ===== ROLE =====

    #[test]
    fn test_validate_role_all_valid() {
        for role in VALID_ROLES {
            assert!(validate_role(role).is_ok(), "role '{}' should be valid", role);
        }
    }

    #[test]
    fn test_validate_role_invalid() {
        assert!(validate_role("superadmin").is_err());
        assert!(validate_role("").is_err());
    }

    // ===== EMAIL =====

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("a@b.c").is_ok());
    }

    #[test]
    fn test_validate_email_empty_rejected() {
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_email_no_at_rejected() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn test_validate_email_no_dot_rejected() {
        assert!(validate_email("user@example").is_err());
    }

    #[test]
    fn test_validate_email_space_rejected() {
        assert!(validate_email("user @example.com").is_err());
    }

    // ===== PASSWORD =====

    #[test]
    fn test_validate_password_too_short() {
        assert!(validate_password("1234567").is_err());
    }

    #[test]
    fn test_validate_password_min_length() {
        assert!(validate_password("12345678").is_ok());
    }

    // ===== LINE ITEMS =====

    #[test]
    fn test_validate_line_items_empty_rejected() {
        assert!(validate_line_items(&[]).is_err());
    }

    #[test]
    fn test_validate_line_items_valid() {
        let items = vec![("Journal".to_string(), 2, 10.0)];
        assert!(validate_line_items(&items).is_ok());
    }

    #[test]
    fn test_validate_line_items_zero_quantity_rejected() {
        let items = vec![("Journal".to_string(), 0, 10.0)];
        assert!(validate_line_items(&items).is_err());
    }

    #[test]
    fn test_validate_line_items_negative_price_rejected() {
        let items = vec![("Journal".to_string(), 1, -5.0)];
        assert!(validate_line_items(&items).is_err());
    }

    #[test]
    fn test_validate_line_items_empty_title_rejected() {
        let items = vec![("".to_string(), 1, 10.0)];
        assert!(validate_line_items(&items).is_err());
    }

    // ===== CONSTANTS MATCH BACKEND =====

    #[test]
    fn test_constants_match_backend() {
        use backend::models;
        assert_eq!(MAX_TITLE_LEN, 120);
        assert_eq!(MAX_SUMMARY_LEN, 500);
        assert_eq!(MAX_FILE_SIZE, models::MAX_FILE_SIZE);
        assert_eq!(MAX_REVIEW_IMAGES, models::MAX_REVIEW_IMAGES);
        assert_eq!(MAX_REVIEW_IMAGE_SIZE, models::MAX_REVIEW_IMAGE_SIZE);
        assert_eq!(MAX_SUBMISSION_VERSIONS, models::MAX_SUBMISSION_VERSIONS);
    }

    #[test]
    fn test_allowed_extensions_match_backend_file_type_list() {
        // Backend accepts: pdf, docx, png, jpg, jpeg
        assert!(ALLOWED_EXTENSIONS.contains(&"pdf"));
        assert!(ALLOWED_EXTENSIONS.contains(&"docx"));
        assert!(ALLOWED_EXTENSIONS.contains(&"png"));
        assert!(ALLOWED_EXTENSIONS.contains(&"jpg"));
        assert!(ALLOWED_EXTENSIONS.contains(&"jpeg"));
        assert_eq!(ALLOWED_EXTENSIONS.len(), 5);
    }
}
