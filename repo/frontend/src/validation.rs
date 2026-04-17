/// Frontend validation rules that mirror backend constraints.
/// These run client-side for fast feedback before form submission.
/// The authoritative validation is always on the backend.

/// Maximum allowed title length for submissions and reviews.
pub const MAX_TITLE_LEN: usize = 120;
/// Maximum allowed summary length for submissions.
pub const MAX_SUMMARY_LEN: usize = 500;
/// Maximum allowed file size in bytes (25 MB).
pub const MAX_FILE_SIZE: u64 = 25 * 1024 * 1024;
/// Maximum review images per review.
pub const MAX_REVIEW_IMAGES: usize = 6;
/// Maximum review image size in bytes (5 MB).
pub const MAX_REVIEW_IMAGE_SIZE: u64 = 5 * 1024 * 1024;
/// Maximum submission versions.
pub const MAX_SUBMISSION_VERSIONS: i32 = 10;
/// Review rating bounds (inclusive).
pub const RATING_MIN: i32 = 1;
pub const RATING_MAX: i32 = 5;

/// Allowed file extensions for submission uploads.
pub const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "docx", "png", "jpg", "jpeg"];
/// Allowed submission types.
pub const ALLOWED_SUBMISSION_TYPES: &[&str] = &["journal_article", "conference_paper", "thesis", "book_chapter"];
/// Allowed subscription periods.
pub const ALLOWED_SUBSCRIPTION_PERIODS: &[&str] = &["monthly", "quarterly", "annual"];
/// Allowed case types.
pub const ALLOWED_CASE_TYPES: &[&str] = &["return", "refund", "exchange"];
/// Allowed case priorities.
pub const ALLOWED_PRIORITIES: &[&str] = &["low", "medium", "high", "urgent"];
/// Allowed payment methods.
pub const ALLOWED_PAYMENT_METHODS: &[&str] = &["cash", "check", "on_account"];
/// Valid application roles.
pub const VALID_ROLES: &[&str] = &["student", "instructor", "academic_staff", "administrator"];

pub fn validate_title(title: &str) -> Result<(), &'static str> {
    if title.trim().is_empty() {
        return Err("Title is required");
    }
    if title.len() > MAX_TITLE_LEN {
        return Err("Title must be 120 characters or fewer");
    }
    Ok(())
}

pub fn validate_summary(summary: &str) -> Result<(), &'static str> {
    if summary.len() > MAX_SUMMARY_LEN {
        return Err("Summary must be 500 characters or fewer");
    }
    Ok(())
}

pub fn validate_rating(rating: i32) -> Result<(), &'static str> {
    if rating < RATING_MIN || rating > RATING_MAX {
        return Err("Rating must be between 1 and 5");
    }
    Ok(())
}

pub fn validate_file_extension(filename: &str) -> Result<(), String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    if ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        Ok(())
    } else {
        Err(format!("File type '{}' not allowed. Allowed: PDF, DOCX, PNG, JPG", ext))
    }
}

pub fn validate_file_size(size: u64) -> Result<(), &'static str> {
    if size > MAX_FILE_SIZE {
        return Err("File exceeds 25 MB limit");
    }
    Ok(())
}

pub fn validate_review_image_size(size: u64) -> Result<(), &'static str> {
    if size > MAX_REVIEW_IMAGE_SIZE {
        return Err("Image exceeds 5 MB limit");
    }
    Ok(())
}

pub fn validate_subscription_period(period: &str) -> Result<(), &'static str> {
    if ALLOWED_SUBSCRIPTION_PERIODS.contains(&period) {
        Ok(())
    } else {
        Err("Invalid subscription period")
    }
}

pub fn validate_case_type(case_type: &str) -> Result<(), &'static str> {
    if ALLOWED_CASE_TYPES.contains(&case_type) {
        Ok(())
    } else {
        Err("Invalid case type")
    }
}

pub fn validate_role(role: &str) -> Result<(), &'static str> {
    if VALID_ROLES.contains(&role) {
        Ok(())
    } else {
        Err("Invalid role")
    }
}

pub fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.is_empty() {
        return Err("Email is required");
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format");
    }
    if email.contains(' ') {
        return Err("Email must not contain spaces");
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    Ok(())
}

/// Returns true if all line items in an order are valid (non-empty, positive qty and price).
pub fn validate_line_items(items: &[(String, i32, f64)]) -> Result<(), &'static str> {
    if items.is_empty() {
        return Err("At least one line item is required");
    }
    for (title, qty, price) in items {
        if title.trim().is_empty() {
            return Err("Publication title is required");
        }
        if *qty <= 0 {
            return Err("Quantity must be positive");
        }
        if *price < 0.0 {
            return Err("Price cannot be negative");
        }
    }
    Ok(())
}
