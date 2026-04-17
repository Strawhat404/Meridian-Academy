//! Tests for `frontend::nav_logic` — real frontend module imports.
//! Verifies the navigation visibility and API path routing logic
//! extracted from components/nav.rs and pages/*.rs.

#[cfg(test)]
mod tests {
    use frontend::nav_logic::*;

    // ===== SUBMISSION VISIBILITY =====

    #[test]
    fn test_student_sees_submissions() {
        assert!(show_submissions("student"));
    }

    #[test]
    fn test_instructor_sees_submissions() {
        assert!(show_submissions("instructor"));
    }

    #[test]
    fn test_staff_does_not_see_submissions_link() {
        assert!(!show_submissions("academic_staff"));
    }

    #[test]
    fn test_admin_does_not_see_submissions_link() {
        assert!(!show_submissions("administrator"));
    }

    // ===== ADMIN VISIBILITY =====

    #[test]
    fn test_student_no_admin_link() {
        assert!(!show_admin("student"));
    }

    #[test]
    fn test_instructor_no_admin_link() {
        assert!(!show_admin("instructor"));
    }

    #[test]
    fn test_staff_sees_admin_link() {
        assert!(show_admin("academic_staff"));
    }

    #[test]
    fn test_admin_sees_admin_link() {
        assert!(show_admin("administrator"));
    }

    // ===== IS_STAFF =====

    #[test]
    fn test_is_staff_true_for_admin() {
        assert!(is_staff("administrator"));
    }

    #[test]
    fn test_is_staff_true_for_academic_staff() {
        assert!(is_staff("academic_staff"));
    }

    #[test]
    fn test_is_staff_false_for_student() {
        assert!(!is_staff("student"));
    }

    #[test]
    fn test_is_staff_false_for_instructor() {
        assert!(!is_staff("instructor"));
    }

    // ===== API PATH ROUTING =====

    #[test]
    fn test_orders_path_staff() {
        assert_eq!(orders_api_path("administrator"), "/api/orders");
        assert_eq!(orders_api_path("academic_staff"), "/api/orders");
    }

    #[test]
    fn test_orders_path_student() {
        assert_eq!(orders_api_path("student"), "/api/orders/my");
        assert_eq!(orders_api_path("instructor"), "/api/orders/my");
    }

    #[test]
    fn test_cases_path_staff() {
        assert_eq!(cases_api_path("administrator"), "/api/cases");
    }

    #[test]
    fn test_cases_path_student() {
        assert_eq!(cases_api_path("student"), "/api/cases/my");
    }

    #[test]
    fn test_submissions_path_staff() {
        assert_eq!(submissions_api_path("academic_staff"), "/api/submissions");
    }

    #[test]
    fn test_submissions_path_student() {
        assert_eq!(submissions_api_path("student"), "/api/submissions/my");
    }

    // ===== MENU ITEMS =====

    #[test]
    fn test_student_menu_items() {
        let items = menu_items("student");
        assert!(items.contains(&"Dashboard"));
        assert!(items.contains(&"Submissions"));
        assert!(items.contains(&"Orders"));
        assert!(items.contains(&"Reviews"));
        assert!(items.contains(&"Cases"));
        assert!(items.contains(&"Profile"));
        assert!(!items.contains(&"Admin"));
    }

    #[test]
    fn test_instructor_menu_items() {
        let items = menu_items("instructor");
        assert!(items.contains(&"Submissions"));
        assert!(!items.contains(&"Admin"));
    }

    #[test]
    fn test_staff_menu_items() {
        let items = menu_items("academic_staff");
        assert!(!items.contains(&"Submissions"));
        assert!(items.contains(&"Admin"));
        assert!(items.contains(&"Orders"));
        assert!(items.contains(&"Profile"));
    }

    #[test]
    fn test_admin_menu_items() {
        let items = menu_items("administrator");
        assert!(items.contains(&"Admin"));
        assert!(items.contains(&"Dashboard"));
        assert!(items.contains(&"Profile"));
    }

    #[test]
    fn test_all_roles_have_profile_in_menu() {
        for role in &["student", "instructor", "academic_staff", "administrator"] {
            assert!(menu_items(role).contains(&"Profile"),
                "role '{}' must have Profile", role);
        }
    }

    #[test]
    fn test_all_roles_have_dashboard_in_menu() {
        for role in &["student", "instructor", "academic_staff", "administrator"] {
            assert!(menu_items(role).contains(&"Dashboard"),
                "role '{}' must have Dashboard", role);
        }
    }

    #[test]
    fn test_all_roles_have_orders_in_menu() {
        for role in &["student", "instructor", "academic_staff", "administrator"] {
            assert!(menu_items(role).contains(&"Orders"),
                "role '{}' must have Orders", role);
        }
    }
}
