#[cfg(feature = "web-ui")]
use askama::Template;
#[cfg(feature = "web-ui")]
use askama_axum::IntoResponse;
#[cfg(feature = "web-ui")]
use axum::extract::Query;
#[cfg(feature = "web-ui")]
use axum::{
    body::Body,
    http::{header, Request, Response},
    middleware::Next,
};
#[cfg(feature = "web-ui")]
use serde::Deserialize;

// Asset version for cache busting
// Uses Cargo package version by default, but can be overridden via ASSET_VERSION env var at build time
// Example: ASSET_VERSION=1.2.3 cargo build --features web-ui
// This ensures CDN assets are properly versioned to avoid stale cache issues
#[cfg(feature = "web-ui")]
const ASSET_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "web-ui")]
fn get_asset_version() -> &'static str {
    // Allow override via environment variable for manual cache busting
    // This is useful for forcing cache invalidation without changing the package version
    option_env!("ASSET_VERSION").unwrap_or(ASSET_VERSION)
}

// Base template context with asset versioning
#[cfg(feature = "web-ui")]
pub struct BaseTemplateContext {
    pub asset_version: &'static str,
}

#[cfg(feature = "web-ui")]
impl Default for BaseTemplateContext {
    fn default() -> Self {
        Self {
            asset_version: get_asset_version(),
        }
    }
}

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    #[template(escape = "none")]
    pub asset_version: &'static str,
}

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    #[template(escape = "none")]
    pub asset_version: &'static str,
}

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "analytics.html")]
pub struct AnalyticsTemplate {
    pub symbol: Option<String>,
    #[template(escape = "none")]
    pub asset_version: &'static str,
}

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    #[template(escape = "none")]
    pub asset_version: &'static str,
}

#[cfg(feature = "web-ui")]
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub symbol: Option<String>,
}

#[cfg(feature = "web-ui")]
pub async fn dashboard() -> impl IntoResponse {
    DashboardTemplate {
        asset_version: get_asset_version(),
    }
}

#[cfg(feature = "web-ui")]
pub async fn search() -> impl IntoResponse {
    SearchTemplate {
        asset_version: get_asset_version(),
    }
}

#[cfg(feature = "web-ui")]
pub async fn analytics(Query(params): Query<AnalyticsQuery>) -> impl IntoResponse {
    AnalyticsTemplate {
        symbol: params.symbol,
        asset_version: get_asset_version(),
    }
}

#[cfg(feature = "web-ui")]
pub async fn login() -> impl IntoResponse {
    LoginTemplate {
        asset_version: get_asset_version(),
    }
}

/// Middleware to add cache headers for web UI responses
/// HTML pages: short cache (5 minutes) to allow updates
/// Static assets: long cache (1 year) with versioning for cache busting
#[cfg(feature = "web-ui")]
pub async fn cache_headers_middleware(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    
    // Set cache headers based on content type
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        if let Ok(content_type_str) = content_type.to_str() {
            if content_type_str.contains("text/html") {
                // HTML pages: short cache to allow quick updates
                // This prevents browsers from caching HTML too long, but allows CDN caching
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    "public, max-age=300, must-revalidate".parse().unwrap(),
                );
                // Add ETAG for conditional requests
                response.headers_mut().insert(
                    header::ETAG,
                    format!("\"{}\"", get_asset_version()).parse().unwrap(),
                );
            } else if content_type_str.contains("text/css")
                || content_type_str.contains("application/javascript")
                || content_type_str.contains("image/")
                || content_type_str.contains("font/")
            {
                // Static assets: long cache with versioning
                // Since we version assets, they can be cached forever
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable".parse().unwrap(),
                );
            }
        }
    }
    
    response
}

// Placeholder functions for when web-ui feature is disabled
#[cfg(not(feature = "web-ui"))]
#[allow(dead_code)]
pub async fn dashboard() -> Result<&'static str, axum::http::StatusCode> {
    Err(axum::http::StatusCode::NOT_FOUND)
}

#[cfg(not(feature = "web-ui"))]
#[allow(dead_code)]
pub async fn search() -> Result<&'static str, axum::http::StatusCode> {
    Err(axum::http::StatusCode::NOT_FOUND)
}

#[cfg(not(feature = "web-ui"))]
#[allow(dead_code)]
pub async fn analytics() -> Result<&'static str, axum::http::StatusCode> {
    Err(axum::http::StatusCode::NOT_FOUND)
} 