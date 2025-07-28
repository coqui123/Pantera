#[cfg(feature = "web-ui")]
use askama::Template;
#[cfg(feature = "web-ui")]
use askama_axum::IntoResponse;
#[cfg(feature = "web-ui")]
use axum::{
    extract::Query,
    response::Html,
};
#[cfg(feature = "web-ui")]
use serde::Deserialize;

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate;

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate;

#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "analytics.html")]
pub struct AnalyticsTemplate {
    pub symbol: Option<String>,
}

#[cfg(feature = "web-ui")]
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub symbol: Option<String>,
}

#[cfg(feature = "web-ui")]
pub async fn dashboard() -> impl IntoResponse {
    DashboardTemplate
}

#[cfg(feature = "web-ui")]
pub async fn search() -> impl IntoResponse {
    SearchTemplate
}

#[cfg(feature = "web-ui")]
pub async fn analytics(Query(params): Query<AnalyticsQuery>) -> impl IntoResponse {
    AnalyticsTemplate {
        symbol: params.symbol,
    }
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