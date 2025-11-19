use thiserror::Error;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;

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

/// AppError for authentication and general app errors
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        
        let body = Json(serde_json::json!({
            "success": false,
            "error": error_message,
        }));
        
        (status, body).into_response()
    }
}

