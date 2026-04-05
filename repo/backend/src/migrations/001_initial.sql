CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    contact_info TEXT,
    role ENUM('student', 'instructor', 'academic_staff', 'administrator') NOT NULL DEFAULT 'student',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    soft_deleted_at DATETIME NULL,
    deletion_scheduled_at DATETIME NULL,
    invoice_title VARCHAR(255) NULL,
    notify_submissions BOOLEAN NOT NULL DEFAULT TRUE,
    notify_orders BOOLEAN NOT NULL DEFAULT TRUE,
    notify_reviews BOOLEAN NOT NULL DEFAULT TRUE,
    notify_cases BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_users_email (email),
    INDEX idx_users_username (username),
    INDEX idx_users_role (role)
);

CREATE TABLE IF NOT EXISTS user_addresses (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    label VARCHAR(100) NOT NULL,
    street_line1 VARCHAR(255) NOT NULL,
    street_line2 VARCHAR(255) NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(2) NOT NULL,
    zip_code VARCHAR(10) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_addresses_user (user_id)
);

CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    token VARCHAR(512) NOT NULL UNIQUE,
    last_activity DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_sessions_token (token),
    INDEX idx_sessions_user (user_id)
);

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at DATETIME NOT NULL,
    created_by VARCHAR(36) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id),
    INDEX idx_reset_token (token)
);

CREATE TABLE IF NOT EXISTS audit_log (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NULL,
    action VARCHAR(200) NOT NULL,
    target_type VARCHAR(100) NULL,
    target_id VARCHAR(36) NULL,
    details TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_audit_log_user (user_id),
    INDEX idx_audit_log_action (action),
    INDEX idx_audit_log_created (created_at)
);

CREATE TABLE IF NOT EXISTS notifications (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_notifications_user (user_id),
    INDEX idx_notifications_read (is_read)
);

CREATE TABLE IF NOT EXISTS submissions (
    id VARCHAR(36) PRIMARY KEY,
    author_id VARCHAR(36) NOT NULL,
    title VARCHAR(120) NOT NULL,
    summary VARCHAR(500) NULL,
    submission_type VARCHAR(50) NOT NULL,
    status ENUM('draft', 'submitted', 'in_review', 'revision_requested', 'accepted', 'rejected', 'published', 'blocked') NOT NULL DEFAULT 'draft',
    deadline DATETIME NULL,
    current_version INT NOT NULL DEFAULT 0,
    max_versions INT NOT NULL DEFAULT 10,
    meta_title VARCHAR(120) NULL,
    meta_description VARCHAR(255) NULL,
    slug VARCHAR(255) NULL,
    tags TEXT NULL,
    keywords TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_submissions_author (author_id),
    INDEX idx_submissions_status (status)
);

CREATE TABLE IF NOT EXISTS submission_versions (
    id VARCHAR(36) PRIMARY KEY,
    submission_id VARCHAR(36) NOT NULL,
    version_number INT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(20) NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    magic_bytes VARCHAR(20) NULL,
    form_data TEXT NULL,
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (submission_id) REFERENCES submissions(id) ON DELETE CASCADE,
    INDEX idx_sv_submission (submission_id),
    UNIQUE KEY uq_sv_version (submission_id, version_number)
);

CREATE TABLE IF NOT EXISTS orders (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    order_number VARCHAR(50) NOT NULL UNIQUE,
    subscription_period ENUM('monthly', 'quarterly', 'annual') NOT NULL,
    shipping_address_id VARCHAR(36) NULL,
    status ENUM('pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled', 'split', 'merged') NOT NULL DEFAULT 'pending',
    payment_status ENUM('unpaid', 'held', 'paid', 'refunded', 'partial_refund') NOT NULL DEFAULT 'unpaid',
    total_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    parent_order_id VARCHAR(36) NULL,
    is_flagged BOOLEAN NOT NULL DEFAULT FALSE,
    flag_reason VARCHAR(500) NULL,
    flag_cleared_by VARCHAR(36) NULL,
    flag_cleared_at DATETIME NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_orders_user (user_id),
    INDEX idx_orders_status (status),
    INDEX idx_orders_flagged (is_flagged)
);

CREATE TABLE IF NOT EXISTS order_line_items (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    publication_title VARCHAR(255) NOT NULL,
    series_name VARCHAR(255) NULL,
    quantity INT NOT NULL DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    line_total DECIMAL(10,2) NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    INDEX idx_oli_order (order_id)
);

CREATE TABLE IF NOT EXISTS fulfillment_events (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    line_item_id VARCHAR(36) NULL,
    event_type ENUM('missing_issue', 'reshipment', 'delay', 'discontinuation', 'edition_change', 'delivered') NOT NULL,
    issue_identifier VARCHAR(255) NULL,
    reason TEXT NOT NULL,
    expected_date DATE NULL,
    actual_date DATE NULL,
    logged_by VARCHAR(36) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (logged_by) REFERENCES users(id),
    INDEX idx_fe_order (order_id)
);

CREATE TABLE IF NOT EXISTS reconciliation_records (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    line_item_id VARCHAR(36) NULL,
    issue_identifier VARCHAR(255) NOT NULL,
    expected_qty INT NOT NULL,
    received_qty INT NOT NULL DEFAULT 0,
    status ENUM('pending', 'matched', 'discrepancy') NOT NULL DEFAULT 'pending',
    notes TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    INDEX idx_rr_order (order_id)
);

CREATE TABLE IF NOT EXISTS reviews (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    line_item_id VARCHAR(36) NULL,
    user_id VARCHAR(36) NOT NULL,
    rating INT NOT NULL,
    title VARCHAR(120) NOT NULL,
    body TEXT NOT NULL,
    is_followup BOOLEAN NOT NULL DEFAULT FALSE,
    parent_review_id VARCHAR(36) NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_reviews_order (order_id),
    INDEX idx_reviews_user (user_id)
);

CREATE TABLE IF NOT EXISTS review_images (
    id VARCHAR(36) PRIMARY KEY,
    review_id VARCHAR(36) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(20) NOT NULL DEFAULT '',
    file_hash VARCHAR(64) NOT NULL DEFAULT '',
    image_data MEDIUMBLOB NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (review_id) REFERENCES reviews(id) ON DELETE CASCADE,
    INDEX idx_ri_review (review_id)
);

CREATE TABLE IF NOT EXISTS after_sales_cases (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    reporter_id VARCHAR(36) NOT NULL,
    assigned_to VARCHAR(36) NULL,
    case_type ENUM('return', 'refund', 'exchange') NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    status ENUM('submitted', 'in_review', 'awaiting_evidence', 'arbitrated', 'approved', 'denied', 'closed') NOT NULL DEFAULT 'submitted',
    priority ENUM('low', 'medium', 'high', 'urgent') NOT NULL DEFAULT 'medium',
    submitted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    first_response_at DATETIME NULL,
    first_response_due DATETIME NULL,
    resolution_target DATETIME NULL,
    resolved_at DATETIME NULL,
    closed_at DATETIME NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (reporter_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_to) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_asc_order (order_id),
    INDEX idx_asc_reporter (reporter_id),
    INDEX idx_asc_status (status)
);

CREATE TABLE IF NOT EXISTS case_comments (
    id VARCHAR(36) PRIMARY KEY,
    case_id VARCHAR(36) NOT NULL,
    author_id VARCHAR(36) NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (case_id) REFERENCES after_sales_cases(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_cc_case (case_id)
);

CREATE TABLE IF NOT EXISTS case_evidence (
    id VARCHAR(36) PRIMARY KEY,
    case_id VARCHAR(36) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    uploaded_by VARCHAR(36) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (case_id) REFERENCES after_sales_cases(id) ON DELETE CASCADE,
    FOREIGN KEY (uploaded_by) REFERENCES users(id),
    INDEX idx_ce_case (case_id)
);

CREATE TABLE IF NOT EXISTS sensitive_words (
    id VARCHAR(36) PRIMARY KEY,
    word VARCHAR(255) NOT NULL UNIQUE,
    action ENUM('replace', 'block') NOT NULL DEFAULT 'replace',
    replacement VARCHAR(255) NULL,
    added_by VARCHAR(36) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (added_by) REFERENCES users(id),
    INDEX idx_sw_word (word)
);

CREATE TABLE IF NOT EXISTS payments (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    idempotency_key VARCHAR(255) NOT NULL UNIQUE,
    payment_method ENUM('cash', 'check', 'on_account') NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    transaction_type ENUM('charge', 'hold', 'release', 'refund') NOT NULL,
    reference_payment_id VARCHAR(36) NULL,
    status ENUM('pending', 'completed', 'held', 'released', 'refunded', 'failed') NOT NULL DEFAULT 'pending',
    check_number VARCHAR(100) NULL,
    notes TEXT NULL,
    processed_by VARCHAR(36) NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (reference_payment_id) REFERENCES payments(id),
    INDEX idx_payments_order (order_id),
    INDEX idx_payments_idempotency (idempotency_key),
    INDEX idx_payments_status (status)
);

CREATE TABLE IF NOT EXISTS reconciliation_reports (
    id VARCHAR(36) PRIMARY KEY,
    report_date DATE NOT NULL,
    expected_balance DECIMAL(12,2) NOT NULL,
    actual_balance DECIMAL(12,2) NOT NULL,
    discrepancy DECIMAL(12,2) NOT NULL,
    details TEXT NULL,
    generated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_rr_date (report_date)
);

CREATE TABLE IF NOT EXISTS abnormal_order_flags (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NULL,
    user_id VARCHAR(36) NULL,
    flag_type ENUM('high_quantity', 'repeated_refunds', 'manual') NOT NULL,
    reason TEXT NOT NULL,
    is_cleared BOOLEAN NOT NULL DEFAULT FALSE,
    cleared_by VARCHAR(36) NULL,
    cleared_at DATETIME NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_aof_cleared (is_cleared)
);

CREATE TABLE IF NOT EXISTS payment_gateway_config (
    id VARCHAR(36) PRIMARY KEY,
    gateway_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    config_json TEXT NULL,
    installed_by VARCHAR(36) NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description VARCHAR(255) NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description VARCHAR(255) NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS role_permissions (
    id VARCHAR(36) PRIMARY KEY,
    role_id VARCHAR(36) NOT NULL,
    permission_id VARCHAR(36) NOT NULL,
    granted_by VARCHAR(36) NULL,
    granted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE,
    UNIQUE KEY uq_role_perm (role_id, permission_id),
    INDEX idx_rp_role (role_id)
)

;

ALTER TABLE submission_versions ADD COLUMN file_data MEDIUMBLOB NULL;

ALTER TABLE review_images ADD COLUMN file_type VARCHAR(20) NOT NULL DEFAULT '';
ALTER TABLE review_images ADD COLUMN file_hash VARCHAR(64) NOT NULL DEFAULT '';
ALTER TABLE review_images ADD COLUMN image_data MEDIUMBLOB NULL
