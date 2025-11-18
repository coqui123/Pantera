use thiserror::Error;

/// Internal error types - detailed for logging and debugging
#[derive(Debug, Error)]
#[allow(dead_code)] // Some variants are reserved for future use
pub enum InternalError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Yahoo Finance API error: {0}")]
    YahooApi(String),

    #[error("Rate limit exceeded for client: {client_id}")]
    RateLimitExceeded { client_id: String },

    #[error("Invalid symbol: {symbol}")]
    InvalidSymbol { symbol: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Insufficient data: {message}")]
    InsufficientData { message: String },

    #[error("Calculation error: {message}")]
    CalculationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

/// External error types - safe to return to clients
#[derive(Debug, Error)]
pub enum ExternalError {
    #[error("Invalid request")]
    InvalidRequest,

    #[error("Symbol not found")]
    SymbolNotFound,

    #[error("Rate limit exceeded. Please try again later")]
    RateLimitExceeded,

    #[error("Insufficient data available")]
    InsufficientData,

    #[error("Internal server error")]
    InternalError,
}

impl From<InternalError> for ExternalError {
    fn from(err: InternalError) -> Self {
        match err {
            InternalError::InvalidSymbol { .. } => ExternalError::SymbolNotFound,
            InternalError::RateLimitExceeded { .. } => ExternalError::RateLimitExceeded,
            InternalError::InsufficientData { .. } => ExternalError::InsufficientData,
            InternalError::InvalidInput { .. } => ExternalError::InvalidRequest,
            _ => ExternalError::InternalError,
        }
    }
}

impl From<YahooServiceError> for InternalError {
    fn from(err: YahooServiceError) -> Self {
        match err {
            YahooServiceError::DatabaseError(e) => InternalError::Database(e),
            YahooServiceError::RateLimitExceeded => InternalError::RateLimitExceeded {
                client_id: "unknown".to_string(),
            },
        }
    }
}

// Re-export YahooServiceError for backward compatibility
use crate::yahoo_service::YahooServiceError;

