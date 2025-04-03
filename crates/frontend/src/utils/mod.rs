use gloo::storage::{LocalStorage, Storage};

/// Storage key for auth token
pub const AUTH_TOKEN_KEY: &str = "easm_auth_token";

/// Get the auth token from local storage
pub fn get_auth_token() -> Option<String> {
    LocalStorage::get(AUTH_TOKEN_KEY).ok()
}

/// Save the auth token to local storage
pub fn save_auth_token(token: &str) -> Result<(), String> {
    LocalStorage::set(AUTH_TOKEN_KEY, token).map_err(|e| e.to_string())
}

/// Clear the auth token from local storage
pub fn clear_auth_token() -> Result<(), String> {
    LocalStorage::delete(AUTH_TOKEN_KEY);
    Ok(())
}

/// Format a date string for display
pub fn format_date(date_string: &str) -> String {
    // In a real app, we would use a proper date formatting library
    // For now, just return as is
    date_string.to_string()
}

/// Format severity for display
pub fn format_severity(severity: &str) -> String {
    match severity.to_lowercase().as_str() {
        "critical" => "Critical".to_string(),
        "high" => "High".to_string(),
        "medium" => "Medium".to_string(),
        "low" => "Low".to_string(),
        "info" => "Info".to_string(),
        _ => severity.to_string(),
    }
}

/// Get CSS class for severity
pub fn severity_class(severity: &str) -> String {
    match severity.to_lowercase().as_str() {
        "critical" => "severity-critical".to_string(),
        "high" => "severity-high".to_string(),
        "medium" => "severity-medium".to_string(),
        "low" => "severity-low".to_string(),
        "info" => "severity-info".to_string(),
        _ => "".to_string(),
    }
}

/// Truncate a string if it's too long
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() > max_chars {
        format!("{}...", &s[0..max_chars])
    } else {
        s.to_string()
    }
}
