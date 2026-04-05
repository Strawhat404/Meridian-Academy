pub mod admin;
pub mod auth;
pub mod cases;
pub mod content;
pub mod health;
pub mod orders;
pub mod payments;
pub mod reviews;
pub mod submissions;
pub mod users;

pub fn health_routes() -> Vec<rocket::Route> {
    routes![health::health_check]
}

pub fn auth_routes() -> Vec<rocket::Route> {
    routes![
        auth::login,
        auth::register,           // POST /api/auth/provision (admin-only)
        auth::me,
        auth::change_password,
        auth::generate_reset_token,
        auth::use_reset_token,
        auth::request_account_deletion,
        auth::cancel_deletion,
        auth::export_my_data,
        auth::logout
    ]
}

pub fn user_routes() -> Vec<rocket::Route> {
    routes![
        users::list_users,
        users::get_user,
        users::update_profile,
        users::update_notification_prefs,
        users::update_user_role,
        users::deactivate_user,
        users::list_addresses,
        users::create_address,
        users::set_default_address,
        users::delete_address,
        users::get_notifications,
        users::mark_notification_read
    ]
}

pub fn submission_routes() -> Vec<rocket::Route> {
    routes![
        submissions::create_submission,
        submissions::list_submissions,
        submissions::get_submission,
        submissions::update_submission,
        submissions::submit_version,
        submissions::list_versions,
        submissions::download_version,
        submissions::my_submissions,
        submissions::approve_blocked
    ]
}

pub fn order_routes() -> Vec<rocket::Route> {
    routes![
        orders::create_order,
        orders::list_orders,
        orders::get_order,
        orders::update_order_status,
        orders::my_orders,
        orders::split_order,
        orders::merge_orders,
        orders::log_fulfillment_event,
        orders::list_fulfillment_events,
        orders::get_reconciliation,
        orders::update_reconciliation,
        orders::clear_flag,
        orders::list_flagged_orders
    ]
}

pub fn review_routes() -> Vec<rocket::Route> {
    routes![
        reviews::create_review,
        reviews::create_followup,
        reviews::list_reviews,
        reviews::get_review,
        reviews::add_review_image,
        reviews::my_reviews
    ]
}

pub fn case_routes() -> Vec<rocket::Route> {
    routes![
        cases::create_case,
        cases::list_cases,
        cases::get_case,
        cases::update_case_status,
        cases::assign_case,
        cases::add_comment,
        cases::get_comments,
        cases::my_cases
    ]
}

pub fn payment_routes() -> Vec<rocket::Route> {
    routes![
        payments::create_payment,
        payments::refund_payment,
        payments::list_payments,
        payments::get_reconciliation_report,
        payments::list_abnormal_flags,
        payments::clear_abnormal_flag
    ]
}

pub fn admin_routes() -> Vec<rocket::Route> {
    routes![
        admin::dashboard_stats,
        admin::audit_log,
        admin::audit_logs,
        admin::system_settings,
        admin::cleanup_soft_deleted
    ]
}

pub fn content_routes() -> Vec<rocket::Route> {
    routes![
        content::list_sensitive_words,
        content::add_sensitive_word,
        content::remove_sensitive_word,
        content::check_content,
        content::submit_item,
        content::approve_item,
        content::reject_item,
        content::request_revision,
        content::publish_item,
        content::rollback_version
    ]
}
