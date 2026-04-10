INSERT IGNORE INTO users (id, username, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at)
VALUES (
    'admin-0001-0001-0001-000000000001',
    'admin',
    'admin@meridian.edu',
    '$2b$12$mRZpl.mhwI7cvgCHUq8VteCUT2xegz9dV5.KnxLhfb49ZFKhBym/i',
    'System',
    'Administrator',
    'administrator',
    true,
    NOW(),
    NOW()
);

INSERT IGNORE INTO roles (id, name, description) VALUES
('role-student',        'student',        'Student role'),
('role-instructor',     'instructor',     'Instructor role'),
('role-academic-staff', 'academic_staff', 'Academic Staff role'),
('role-administrator',  'administrator',  'Administrator role');

INSERT IGNORE INTO permissions (id, name, description) VALUES
('perm-users-list',            'users.list',            'List all users'),
('perm-users-manage',          'users.manage',          'Create/update/deactivate users'),
('perm-users-role-change',     'users.role_change',     'Change user roles'),
('perm-submissions-create',    'submissions.create',    'Create submissions'),
('perm-submissions-list',      'submissions.list',      'List all submissions'),
('perm-submissions-review',    'submissions.review',    'Review/approve submissions'),
('perm-submissions-approve-blocked','submissions.approve_blocked','Approve blocked content'),
('perm-orders-create',         'orders.create',         'Create orders'),
('perm-orders-manage',         'orders.manage',         'Manage all orders, split/merge'),
('perm-orders-fulfillment',    'orders.fulfillment',    'Log fulfillment events'),
('perm-reviews-create',        'reviews.create',        'Create publication reviews'),
('perm-cases-create',          'cases.create',          'Create after-sales cases'),
('perm-cases-manage',          'cases.manage',          'Manage all cases, change status'),
('perm-payments-manage',       'payments.manage',       'Create/refund payments'),
('perm-content-manage',        'content.manage',        'Manage sensitive-word dictionary'),
('perm-admin-dashboard',       'admin.dashboard',       'View admin dashboard'),
('perm-admin-audit',           'admin.audit',           'View audit log'),
('perm-admin-settings',        'admin.settings',        'View/change system settings'),
('perm-admin-provision',       'admin.provision_users', 'Provision new user accounts'),
('perm-reset-tokens',          'auth.generate_reset',   'Generate password reset tokens');

INSERT IGNORE INTO role_permissions (id, role_id, permission_id) VALUES
('rp-s1',  'role-student', 'perm-submissions-create'),
('rp-s2',  'role-student', 'perm-orders-create'),
('rp-s3',  'role-student', 'perm-reviews-create'),
('rp-s4',  'role-student', 'perm-cases-create'),

('rp-i1',  'role-instructor', 'perm-submissions-create'),
('rp-i2',  'role-instructor', 'perm-submissions-review'),
('rp-i3',  'role-instructor', 'perm-orders-create'),
('rp-i4',  'role-instructor', 'perm-reviews-create'),
('rp-i5',  'role-instructor', 'perm-cases-create'),

('rp-as1', 'role-academic-staff', 'perm-users-list'),
('rp-as3', 'role-academic-staff', 'perm-submissions-list'),
('rp-as4', 'role-academic-staff', 'perm-submissions-review'),
('rp-as5', 'role-academic-staff', 'perm-submissions-approve-blocked'),
('rp-as6', 'role-academic-staff', 'perm-orders-manage'),
('rp-as7', 'role-academic-staff', 'perm-orders-fulfillment'),
('rp-as8', 'role-academic-staff', 'perm-cases-manage'),
('rp-as9', 'role-academic-staff', 'perm-payments-manage'),
('rp-as10','role-academic-staff', 'perm-orders-create'),
('rp-as11','role-academic-staff', 'perm-reviews-create'),
('rp-as12','role-academic-staff', 'perm-cases-create'),

('rp-a1',  'role-administrator', 'perm-users-list'),
('rp-a2',  'role-administrator', 'perm-users-manage'),
('rp-a3',  'role-administrator', 'perm-users-role-change'),
('rp-a4',  'role-administrator', 'perm-submissions-list'),
('rp-a5',  'role-administrator', 'perm-submissions-review'),
('rp-a6',  'role-administrator', 'perm-submissions-approve-blocked'),
('rp-a7',  'role-administrator', 'perm-orders-manage'),
('rp-a8',  'role-administrator', 'perm-orders-fulfillment'),
('rp-a9',  'role-administrator', 'perm-cases-manage'),
('rp-a10', 'role-administrator', 'perm-payments-manage'),
('rp-a11', 'role-administrator', 'perm-content-manage'),
('rp-a12', 'role-administrator', 'perm-admin-dashboard'),
('rp-a13', 'role-administrator', 'perm-admin-audit'),
('rp-a14', 'role-administrator', 'perm-admin-settings'),
('rp-a15', 'role-administrator', 'perm-admin-provision'),
('rp-a16', 'role-administrator', 'perm-reset-tokens'),
('rp-a17', 'role-administrator', 'perm-orders-create'),
('rp-a18', 'role-administrator', 'perm-reviews-create'),
('rp-a19', 'role-administrator', 'perm-cases-create');

INSERT IGNORE INTO sensitive_words (id, word, action, replacement, added_by, created_at) VALUES
('sw-0001', 'badword', 'replace', '***', 'admin-0001-0001-0001-000000000001', NOW()),
('sw-0002', 'forbidden', 'block', NULL, 'admin-0001-0001-0001-000000000001', NOW()),
('sw-0003', 'offensive', 'replace', '[redacted]', 'admin-0001-0001-0001-000000000001', NOW())
