use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveWord {
    pub id: String,
    pub word: String,
    pub action: String,
    pub replacement: Option<String>,
    pub added_by: String,
}

#[derive(Debug, Deserialize)]
pub struct AddSensitiveWordRequest {
    pub word: String,
    pub action: String,
    pub replacement: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveSensitiveWordRequest {
    pub word_id: String,
}

#[derive(Debug, Serialize)]
pub struct ContentCheckResult {
    pub is_blocked: bool,
    pub blocked_words: Vec<String>,
    pub processed_text: String,
    pub replacements_made: Vec<(String, String)>,
}

/// Check text against sensitive word dictionary.
/// Returns (is_blocked, blocked_words, processed_text, replacements)
pub fn check_sensitive_words(text: &str, words: &[SensitiveWord]) -> ContentCheckResult {
    let mut blocked_words = Vec::new();
    let mut replacements_made = Vec::new();
    let mut processed = text.to_string();
    let text_lower = text.to_lowercase();

    for sw in words {
        let word_lower = sw.word.to_lowercase();
        if text_lower.contains(&word_lower) {
            match sw.action.as_str() {
                "block" => {
                    blocked_words.push(sw.word.clone());
                }
                "replace" => {
                    let replacement = sw.replacement.clone().unwrap_or_else(|| "***".to_string());
                    // Case-insensitive replacement
                    let mut result = String::new();
                    let mut remaining = processed.as_str();
                    while let Some(pos) = remaining.to_lowercase().find(&word_lower) {
                        result.push_str(&remaining[..pos]);
                        result.push_str(&replacement);
                        remaining = &remaining[pos + sw.word.len()..];
                        replacements_made.push((sw.word.clone(), replacement.clone()));
                    }
                    result.push_str(remaining);
                    processed = result;
                }
                _ => {}
            }
        }
    }

    ContentCheckResult {
        is_blocked: !blocked_words.is_empty(),
        blocked_words,
        processed_text: processed,
        replacements_made,
    }
}
