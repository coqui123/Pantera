use crate::errors::InternalError;
use crate::config::{MAX_SYMBOL_LENGTH, MAX_SEARCH_QUERY_LENGTH};

/// Validate a stock symbol
pub fn validate_symbol(symbol: &str) -> Result<(), InternalError> {
    if symbol.is_empty() {
        return Err(InternalError::InvalidInput {
            message: "Symbol cannot be empty".to_string(),
        });
    }

    if symbol.len() > MAX_SYMBOL_LENGTH {
        return Err(InternalError::InvalidInput {
            message: format!("Symbol too long (max {} characters)", MAX_SYMBOL_LENGTH),
        });
    }

    // Allow alphanumeric characters, dots, and hyphens
    if !symbol.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return Err(InternalError::InvalidInput {
            message: "Symbol contains invalid characters. Only alphanumeric, dots, and hyphens are allowed".to_string(),
        });
    }

    Ok(())
}

/// Validate and sanitize search query
pub fn validate_search_query(query: &str) -> Result<String, InternalError> {
    let trimmed = query.trim();
    
    if trimmed.is_empty() {
        return Err(InternalError::InvalidInput {
            message: "Search query cannot be empty".to_string(),
        });
    }

    if trimmed.len() > MAX_SEARCH_QUERY_LENGTH {
        return Err(InternalError::InvalidInput {
            message: format!("Search query too long (max {} characters)", MAX_SEARCH_QUERY_LENGTH),
        });
    }

    // Remove any potentially dangerous characters but keep basic punctuation
    let sanitized: String = trimmed
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '.' | '-' | ',' | '&'))
        .take(MAX_SEARCH_QUERY_LENGTH)
        .collect();

    if sanitized.is_empty() {
        return Err(InternalError::InvalidInput {
            message: "Search query contains only invalid characters".to_string(),
        });
    }

    Ok(sanitized)
}

/// Validate date range
pub fn validate_date_range(
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), InternalError> {
    if let (Some(start), Some(end)) = (start_date, end_date) {
        if start > end {
            return Err(InternalError::InvalidInput {
                message: "Start date must be before end date".to_string(),
            });
        }
        
        // Check if range is too large (e.g., more than 10 years)
        let duration = end.signed_duration_since(start);
        if duration.num_days() > 3650 {
            return Err(InternalError::InvalidInput {
                message: "Date range cannot exceed 10 years".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate limit parameter
pub fn validate_limit(limit: Option<i32>, max: i32, default: i32) -> i32 {
    limit.unwrap_or(default).clamp(1, max)
}

