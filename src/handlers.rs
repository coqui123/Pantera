use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::models::{ApiResponse, HistoricalResponse, ProfileResponse, QuoteResponse, Symbol};
use crate::yahoo_service::{YahooFinanceService, YahooServiceError};

type AppState = Arc<YahooFinanceService>;

#[derive(Debug, Deserialize)]
pub struct HistoricalParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub interval: Option<String>,
    pub limit: Option<i32>,
    pub force_refresh: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BulkParams {
    pub symbols: String, // comma-separated symbols
    pub interval: Option<String>,
    pub max_concurrent: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisParams {
    pub limit: Option<i32>,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<i32>,
}

// Helper function to extract client identifier for rate limiting
fn get_client_id() -> String {
    "default_client".to_string() // Simplified for web UI compatibility
}

// Health check endpoint
pub async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    let health_data = serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now(),
        "features": ["rate_limiting", "caching", "cow_optimization"]
    });
    Json(ApiResponse::success(health_data))
}

// Get all symbols with rate limiting
pub async fn get_symbols(
    State(service): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Symbol>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    match service.db.get_all_symbols().await {
        Ok(symbols) => Ok(Json(ApiResponse::success(symbols))),
        Err(e) => {
            error!("Failed to get symbols: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Search symbols with optimized string handling
pub async fn search_symbols(
    State(service): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<Symbol>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let query = params.q.trim();
    if query.is_empty() {
        return Ok(Json(ApiResponse::success(vec![])));
    }

    let limit = params.limit.unwrap_or(10).min(50); // Cap at 50 results

    match service.db.search_symbols(query, limit).await {
        Ok(symbols) => {
            debug!("Found {} symbols matching '{}'", symbols.len(), query);
            Ok(Json(ApiResponse::success(symbols)))
        }
        Err(e) => {
            error!("Failed to search symbols: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Validate symbol with caching
pub async fn validate_symbol(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    
    match service.validate_symbol(&symbol).await {
        Ok(is_valid) => {
            let response = serde_json::json!({
                "symbol": symbol,
                "valid": is_valid,
                "timestamp": Utc::now()
            });
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to validate symbol {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get historical data with Cow optimization
pub async fn get_historical_data(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<HistoricalParams>,
) -> Result<Json<ApiResponse<HistoricalResponse<'static>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    let symbol_cow = Cow::Owned(symbol.clone());
    
    // Parse dates
    let start_date = params.start_date;
    let end_date = params.end_date;
    let force_refresh = params.force_refresh.unwrap_or(false);

    // If force refresh or limit is provided, fetch fresh data
    if force_refresh || (params.limit.unwrap_or(0) > 0 && params.interval.is_some()) {
        if let Some(ref interval) = params.interval {
            if let Err(e) = service
                .fetch_historical_data(&symbol, interval, force_refresh)
                .await
            {
                warn!(
                    "Failed to fetch fresh historical data for {}: {}",
                    symbol, e
                );
            }
        }
    }

    match service
        .get_historical_data(
            &symbol,
            start_date,
            end_date,
            params.interval.as_deref(),
            params.limit,
        )
        .await
    {
        Ok(data) => {
            let count = data.len();
            let response = HistoricalResponse {
                symbol: symbol_cow,
                data,
                count,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get historical data for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Fetch historical data (POST endpoint)
pub async fn fetch_historical_data(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<HistoricalParams>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    let interval = params.interval.unwrap_or_else(|| "1d".to_string());

    match service
        .fetch_historical_data(&symbol, &interval, true)
        .await
    {
        Ok(data) => {
            let message = format!(
                "Successfully fetched {} historical records for {}",
                data.len(),
                symbol
            );
            info!("{}", message);
            Ok(Json(ApiResponse::success(message)))
        }
        Err(e) => {
            error!("Failed to fetch historical data for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get real-time quote with optimized response
pub async fn get_real_time_quote(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiResponse<Option<QuoteResponse<'static>>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();

    match service.get_latest_quote(&symbol).await {
        Ok(quote) => {
            let response = quote.map(|q| QuoteResponse {
                symbol: Cow::Owned(q.symbol),
                price: q.price,
                change: q.change,
                change_percent: q.change_percent,
                volume: q.volume,
                market_time: q.market_time,
                trading_session: Cow::Owned(q.trading_session),
            });
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get latest quote for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get company profile with Cow optimization
pub async fn get_company_profile(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiResponse<ProfileResponse<'static>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();

    match service.fetch_company_profile(&symbol, false).await {
        Ok(profile) => {
            let response = ProfileResponse {
                symbol: Cow::Owned(symbol),
                profile,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get company profile for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get comprehensive symbol overview
pub async fn get_symbol_overview(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiResponse<crate::yahoo_service::SymbolOverview>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();

    match service.get_symbol_overview(&symbol).await {
        Ok(overview) => Ok(Json(ApiResponse::success(overview))),
        Err(e) => {
            error!("Failed to get symbol overview for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Bulk fetch historical data with improved concurrency control
pub async fn bulk_fetch_historical(
    State(service): State<AppState>,
    Query(params): Query<BulkParams>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbols: Vec<&str> = params
        .symbols
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if symbols.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Limit the number of symbols to prevent abuse
    if symbols.len() > 20 {
        let error_msg = format!(
            "Too many symbols requested: {}. Maximum allowed: 20",
            symbols.len()
        );
        return Ok(Json(ApiResponse::error(error_msg)));
    }

    let interval = params.interval.unwrap_or_else(|| "1d".to_string());
    let max_concurrent = params.max_concurrent.unwrap_or(5).clamp(1, 10) as usize;

    match service
        .bulk_fetch_historical(symbols, &interval, max_concurrent)
        .await
    {
        Ok(results) => {
            let response: Vec<serde_json::Value> = results
                .into_iter()
                .map(|(symbol, result)| match result {
                        Ok(data) => serde_json::json!({
                            "symbol": symbol,
                            "success": true,
                            "count": data.len(),
                            "data": data
                        }),
                        Err(e) => serde_json::json!({
                            "symbol": symbol,
                            "success": false,
                            "error": e.to_string()
                        }),
                })
                .collect();
            
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to bulk fetch historical data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get price analysis with optimized calculations
pub async fn get_price_analysis(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisParams>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    let limit = params.days.or(params.limit).unwrap_or(30).clamp(1, 365);

    match service
        .get_historical_data(&symbol, None, None, Some("1d"), Some(limit))
        .await
    {
        Ok(data) => {
            if data.is_empty() {
                let response = serde_json::json!({
                    "symbol": symbol,
                    "error": "No historical data available",
                    "analysis": null
                });
                return Ok(Json(ApiResponse::success(response)));
            }

            // Calculate analytics using iterator methods for better performance
            let prices: Vec<_> = data.iter().map(|p| p.close).collect();
            let volumes: Vec<_> = data.iter().map(|p| p.volume).collect();

            let latest_price = prices[0];
            let oldest_price = *prices.last().unwrap();
            let min_price = *prices.iter().min().unwrap();
            let max_price = *prices.iter().max().unwrap();
            
            let price_change = latest_price - oldest_price;
            let price_change_percent = if oldest_price != rust_decimal::Decimal::ZERO {
                (price_change / oldest_price) * rust_decimal::Decimal::from(100)
            } else {
                rust_decimal::Decimal::ZERO
            };

            // Calculate average price
            let avg_price = prices.iter().sum::<rust_decimal::Decimal>()
                / rust_decimal::Decimal::from(prices.len());

            let avg_volume = volumes.iter().sum::<i64>() / volumes.len() as i64;
            let max_volume = *volumes.iter().max().unwrap_or(&0);
            let min_volume = *volumes.iter().min().unwrap_or(&0);

            // Calculate volatility (standard deviation of price changes)
            let price_changes: Vec<_> = prices
                .windows(2)
                .map(|w| ((w[0] - w[1]) / w[1]).to_f64().unwrap_or(0.0))
                .collect();
            
            let mean_change = price_changes.iter().sum::<f64>() / price_changes.len() as f64;
            let variance = price_changes
                .iter()
                .map(|&x| (x - mean_change).powi(2))
                .sum::<f64>()
                / price_changes.len() as f64;
            let volatility = variance.sqrt();

            let response = serde_json::json!({
                "symbol": symbol,
                "period_days": limit,
                "data_points": data.len(),
                // Top-level fields that the test expects
                "min_price": min_price,
                "max_price": max_price,
                "avg_price": avg_price,
                "volatility": volatility,
                "price_change_percent": price_change_percent,
                // Detailed analysis
                "price_analysis": {
                    "latest_price": latest_price,
                    "oldest_price": oldest_price,
                    "min_price": min_price,
                    "max_price": max_price,
                    "avg_price": avg_price,
                    "price_change": price_change,
                    "price_change_percent": price_change_percent,
                    "volatility": volatility,
                    "high_52w": prices.iter().max(),
                    "low_52w": prices.iter().min(),
                },
                "volume_analysis": {
                    "avg_volume": avg_volume,
                    "max_volume": max_volume,
                    "min_volume": min_volume,
                    "latest_volume": volumes[0],
                },
                "timestamp": Utc::now()
            });

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get price analysis for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get database statistics with cache info
pub async fn get_database_stats(
    State(service): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    match service.get_stats().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            error!("Failed to get database stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Comprehensive quote with rate limiting
pub async fn get_comprehensive_quote(
    Path(symbol): Path<String>,
    State(yahoo_service): State<Arc<YahooFinanceService>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) =
        yahoo_service.check_api_rate_limit(&client_id)
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    
    match yahoo_service.get_comprehensive_quote(&symbol).await {
        Ok(data) => Ok(Json(ApiResponse::success(data))),
        Err(e) => {
            error!("Failed to get comprehensive quote for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Extended quote data with rate limiting
pub async fn get_extended_quote_data(
    Path(symbol): Path<String>,
    State(yahoo_service): State<Arc<YahooFinanceService>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) =
        yahoo_service.check_api_rate_limit(&client_id)
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    
    match yahoo_service.get_extended_quote_data(&symbol).await {
        Ok(data) => Ok(Json(ApiResponse::success(data))),
        Err(e) => {
            error!("Failed to get extended quote data for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get technical indicators for a symbol
pub async fn get_technical_indicators(
    State(service): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisParams>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbol = symbol.to_uppercase();
    let limit = params.days.or(params.limit).unwrap_or(100).clamp(20, 500);

    info!("Fetching technical indicators for {} with limit {}", symbol, limit);
    
    match service
        .get_historical_data(&symbol, None, None, Some("1d"), Some(limit))
        .await
    {
        Ok(data) => {
            info!("Got {} data points for technical analysis of {}", data.len(), symbol);
            
            if data.len() < 20 {
                let error_msg = format!(
                    "Insufficient data for technical analysis (minimum 20 periods required). Available: {} periods", 
                    data.len()
                );
                info!("Insufficient data for {}: {}", symbol, error_msg);
                return Ok(Json(ApiResponse::error(error_msg)));
            }

            // Validate and sanitize input data with comprehensive checks
            let prices: Vec<f64> = data.iter()
                .map(|p| p.close.to_f64().unwrap_or(0.0))
                .filter(|&x| x.is_finite() && x > 0.0 && x < 1e10) // Reasonable price range
                .collect();
            
            let volumes: Vec<f64> = data.iter()
                .map(|p| p.volume as f64)
                .filter(|&x| x.is_finite() && x >= 0.0 && x < 1e15) // Reasonable volume range
                .collect();
                
            let _highs: Vec<f64> = data.iter()
                .map(|p| p.high.to_f64().unwrap_or(0.0))
                .filter(|&x| x.is_finite() && x > 0.0 && x < 1e10)
                .collect();
                
            let _lows: Vec<f64> = data.iter()
                .map(|p| p.low.to_f64().unwrap_or(0.0))
                .filter(|&x| x.is_finite() && x > 0.0 && x < 1e10)
                .collect();
            
            // Final validation after sanitization
            if prices.len() < 20 || prices.iter().all(|&p| p == 0.0) {
                let error_msg = format!(
                    "Insufficient valid price data after sanitization. Symbol: {}, Valid prices: {}", 
                    symbol, prices.len()
                );
                warn!("Technical indicators failed for {}: {}", symbol, error_msg);
                return Ok(Json(ApiResponse::error(error_msg)));
            }

            // Calculate technical indicators with comprehensive error handling
            let calculation_result = std::panic::catch_unwind(|| {
                // Simple Moving Averages with validation
                let sma_5 = calculate_sma_safe(&prices, 5);
                let sma_10 = calculate_sma_safe(&prices, 10);
                let sma_20 = calculate_sma_safe(&prices, 20);
                let sma_50 = calculate_sma_safe(&prices, 50);

                // Exponential Moving Averages with validation
                let ema_12 = calculate_ema_safe(&prices, 12);
                let ema_26 = calculate_ema_safe(&prices, 26);

                // RSI with robust error handling
                let rsi = calculate_rsi_safe(&prices, 14);

                // MACD with validation
                let macd_line = calculate_macd_safe(&ema_12, &ema_26);
                let macd_signal = calculate_ema_safe(&macd_line, 9);
                let macd_histogram: Vec<f64> = macd_line.iter()
                    .zip(macd_signal.iter())
                    .map(|(macd, signal)| macd - signal)
                    .filter(|&x| x.is_finite())
                    .collect();

                // Bollinger Bands with validation
                let (bb_upper, bb_middle, bb_lower) = calculate_bollinger_bands_safe(&prices, 20, 2.0);

                // Volume indicators with validation
                let volume_sma_20 = calculate_sma_safe(&volumes, 20);
                
                // Support and resistance levels (improved calculation)
                let recent_prices = &prices[..std::cmp::min(20, prices.len())];
                let support_level = recent_prices.iter().cloned().fold(f64::INFINITY, f64::min);
                let resistance_level = recent_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                
                // Ensure support and resistance are valid
                let support_level = if support_level.is_finite() { support_level } else { 0.0 };
                let resistance_level = if resistance_level.is_finite() { resistance_level } else { 0.0 };
                
                (sma_5, sma_10, sma_20, sma_50, ema_12, ema_26, rsi, macd_line, macd_signal, macd_histogram, bb_upper, bb_middle, bb_lower, volume_sma_20, support_level, resistance_level)
            });

            let (sma_5, sma_10, sma_20, sma_50, ema_12, ema_26, rsi, macd_line, macd_signal, macd_histogram, bb_upper, bb_middle, bb_lower, volume_sma_20, support_level, resistance_level) = match calculation_result {
                Ok(result) => result,
                Err(_) => {
                    let error_msg = format!("Technical indicators calculation failed for symbol: {}", symbol);
                    error!("Technical indicators calculation panic for {}", symbol);
                    return Ok(Json(ApiResponse::error(error_msg)));
                }
            };

            // Helper function to safely get last value
            let safe_last = |vec: &[f64]| -> f64 {
                vec.last().cloned().unwrap_or(0.0)
            };

            let response = serde_json::json!({
                "symbol": symbol,
                "period": limit,
                "data_points": data.len(),
                "valid_prices": prices.len(),
                "indicators": {
                    "moving_averages": {
                        "sma_5": safe_last(&sma_5),
                        "sma_10": safe_last(&sma_10),
                        "sma_20": safe_last(&sma_20),
                        "sma_50": safe_last(&sma_50),
                        "ema_12": safe_last(&ema_12),
                        "ema_26": safe_last(&ema_26)
                    },
                    "momentum": {
                        "rsi": safe_last(&rsi).clamp(0.0, 100.0),
                        "rsi_signal": get_rsi_signal(safe_last(&rsi))
                    },
                    "macd": {
                        "macd_line": safe_last(&macd_line),
                        "signal_line": safe_last(&macd_signal),
                        "histogram": safe_last(&macd_histogram),
                        "signal": get_macd_signal(safe_last(&macd_line), safe_last(&macd_signal))
                    },
                    "bollinger_bands": {
                        "upper": safe_last(&bb_upper),
                        "middle": safe_last(&bb_middle),
                        "lower": safe_last(&bb_lower),
                        "position": get_bollinger_position_safe(prices.first().cloned().unwrap_or(0.0), &bb_upper, &bb_lower)
                    },
                    "support_resistance": {
                        "support": support_level,
                        "resistance": resistance_level,
                        "current_position": get_price_position_safe(prices.first().cloned().unwrap_or(0.0), support_level, resistance_level)
                    },
                    "volume": {
                        "current": volumes.first().cloned().unwrap_or(0.0),
                        "average_20": safe_last(&volume_sma_20),
                        "volume_ratio": (|| {
                            let current_vol = volumes.first().cloned().unwrap_or(0.0);
                            let avg_vol = safe_last(&volume_sma_20);
                            if avg_vol > 0.0 { current_vol / avg_vol } else { 1.0 }
                        })()
                    }
                },
                "signals": {
                    "overall_trend": determine_overall_trend_safe(&sma_20, &prices),
                    "buy_sell_signals": generate_buy_sell_signals_safe(&data),
                    "strength": calculate_trend_strength_safe(&prices, &sma_20)
                },
                "timestamp": Utc::now()
            });

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get technical indicators for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Compare multiple symbols
pub async fn compare_symbols(
    State(service): State<AppState>,
    Query(params): Query<BulkParams>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let symbols: Vec<&str> = params
        .symbols
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if symbols.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if symbols.len() > 10 {
        let error_msg = format!(
            "Too many symbols for comparison: {}. Maximum allowed: 10",
            symbols.len()
        );
        return Ok(Json(ApiResponse::error(error_msg)));
    }

    let interval = params.interval.unwrap_or_else(|| "1d".to_string());
    let limit = 100; // Fixed limit for comparison

    // Fetch data for all symbols
    let mut comparison_data = serde_json::Map::new();
    let mut correlation_matrix = serde_json::Map::new();
    let mut all_returns: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    for symbol in symbols.iter() {
        match service
            .get_historical_data(symbol, None, None, Some(&interval), Some(limit))
            .await
        {
            Ok(data) => {
                if !data.is_empty() {
                    let prices: Vec<f64> = data.iter().map(|p| p.close.to_f64().unwrap_or(0.0)).collect();
                    let volumes: Vec<i64> = data.iter().map(|p| p.volume).collect();
                    
                    // Calculate returns
                    let returns: Vec<f64> = prices.windows(2)
                        .map(|w| if w[1] != 0.0 { (w[0] - w[1]) / w[1] } else { 0.0 })
                        .collect();
                    
                    all_returns.insert(symbol.to_string(), returns.clone());

                    // Calculate basic metrics
                    let latest_price = prices.first().cloned().unwrap_or(0.0);
                    let oldest_price = prices.last().cloned().unwrap_or(0.0);
                    let price_change = if oldest_price != 0.0 {
                        ((latest_price - oldest_price) / oldest_price) * 100.0
                    } else {
                        0.0
                    };

                    let avg_volume = volumes.iter().sum::<i64>() as f64 / volumes.len() as f64;
                    let volatility = calculate_volatility(&returns);

                    comparison_data.insert(symbol.to_string(), serde_json::json!({
                        "symbol": symbol,
                        "latest_price": latest_price,
                        "price_change_percent": price_change,
                        "volatility": volatility,
                        "avg_volume": avg_volume,
                        "data_points": data.len(),
                        "returns": returns
                    }));
                }
            }
            Err(e) => {
                warn!("Failed to fetch data for symbol {}: {}", symbol, e);
                comparison_data.insert(symbol.to_string(), serde_json::json!({
                    "symbol": symbol,
                    "error": format!("Failed to fetch data: {}", e)
                }));
            }
        }
    }

    // Calculate correlation matrix
    for symbol1 in symbols.iter() {
        let mut correlations = serde_json::Map::new();
        if let Some(returns1) = all_returns.get(*symbol1) {
            for symbol2 in symbols.iter() {
                if let Some(returns2) = all_returns.get(*symbol2) {
                    let correlation = calculate_correlation(returns1, returns2);
                    correlations.insert(symbol2.to_string(), serde_json::json!(correlation));
                }
            }
        }
        correlation_matrix.insert(symbol1.to_string(), serde_json::json!(correlations));
    }

    let response = serde_json::json!({
        "symbols": symbols,
        "comparison": comparison_data,
        "correlation_matrix": correlation_matrix,
        "summary": {
            "total_symbols": symbols.len(),
            "successful_fetches": comparison_data.len(),
            "interval": interval,
            "period": limit
        },
        "timestamp": Utc::now()
    });

    Ok(Json(ApiResponse::success(response)))
}

// Helper functions for technical analysis
#[allow(dead_code)]
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }
    
    let mut sma = Vec::new();
    for i in (period - 1)..prices.len() {
        let start_idx = i.saturating_sub(period.saturating_sub(1));
        let slice = &prices[start_idx..(i + 1)];
        let sum: f64 = slice.iter().filter(|&&x| x.is_finite()).sum();
        let count = slice.iter().filter(|&&x| x.is_finite()).count();
        
        if count > 0 {
            sma.push(sum / count as f64);
        } else {
            sma.push(0.0);
        }
    }
    sma
}

// Safe version of SMA calculation with comprehensive validation
fn calculate_sma_safe(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() || period == 0 || period > prices.len() {
        return vec![];
    }
    
    let mut sma = Vec::new();
    for i in (period - 1)..prices.len() {
        // Saturating arithmetic to completely prevent underflow
        let start_idx = i.saturating_sub(period.saturating_sub(1));
        let end_idx = i + 1;
        
        if start_idx >= prices.len() || end_idx > prices.len() || start_idx >= end_idx {
            continue;
        }
        
        let slice = &prices[start_idx..end_idx];
        let valid_prices: Vec<f64> = slice.iter()
            .filter(|&&x| x.is_finite() && x > 0.0)
            .cloned()
            .collect();
        
        if valid_prices.len() >= (period * 2 / 3) { // At least 2/3 of period must be valid
            let avg = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
            if avg.is_finite() && avg > 0.0 {
                sma.push(avg);
            } else {
                sma.push(0.0);
            }
        } else {
            sma.push(0.0);
        }
    }
    sma
}

#[allow(dead_code)]
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() || period == 0 {
        return vec![];
    }
    
    let mut ema = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);
    
    // Start with first valid price
    let first_price = prices.iter().find(|&&p| p.is_finite()).unwrap_or(&0.0);
    ema.push(*first_price);
    
    for i in 1..prices.len() {
        let current_price = if prices[i].is_finite() { prices[i] } else { ema[i - 1] };
        let new_ema = (current_price * multiplier) + (ema[i - 1] * (1.0 - multiplier));
        
        if new_ema.is_finite() {
            ema.push(new_ema);
        } else {
            ema.push(ema[i - 1]);
        }
    }
    
    ema
}

// Safe version of EMA calculation with comprehensive validation
fn calculate_ema_safe(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() || period == 0 {
        return vec![];
    }
    
    // Validate input data
    let valid_prices: Vec<f64> = prices.iter()
        .filter(|&&x| x.is_finite() && x > 0.0)
        .cloned()
        .collect();
    
    if valid_prices.is_empty() {
        return vec![];
    }
    
    let multiplier = 2.0 / (period as f64 + 1.0);
    if !multiplier.is_finite() || multiplier <= 0.0 || multiplier >= 1.0 {
        return vec![];
    }
    
    let mut ema = Vec::new();
    ema.push(valid_prices[0]);
    
    for i in 1..valid_prices.len() {
        let current_price = valid_prices[i];
        let new_ema = (current_price * multiplier) + (ema[i - 1] * (1.0 - multiplier));
        
        if new_ema.is_finite() && new_ema > 0.0 {
            ema.push(new_ema);
        } else {
            ema.push(ema[i - 1]); // Use previous value if calculation fails
        }
    }
    
    ema
}

#[allow(dead_code)]
fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() <= period || period == 0 {
        return vec![];
    }

    let mut rsi = Vec::new();
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calculate price changes
    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change.is_finite() {
            gains.push(if change > 0.0 { change } else { 0.0 });
            losses.push(if change < 0.0 { -change } else { 0.0 });
        } else {
            gains.push(0.0);
            losses.push(0.0);
        }
    }

    if gains.len() < period {
        return vec![];
    }

    // Calculate initial averages
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    // Calculate first RSI with safe division
    let rs = if avg_loss > 0.0 { avg_gain / avg_loss } else if avg_gain > 0.0 { 100.0 } else { 0.0 };
    let rsi_value = if rs.is_finite() { 100.0 - (100.0 / (1.0 + rs)) } else { 50.0 };
    rsi.push(rsi_value.clamp(0.0, 100.0));

    // Calculate subsequent RSI values
    for i in period..gains.len() {
        avg_gain = ((avg_gain * (period as f64 - 1.0)) + gains[i]) / period as f64;
        avg_loss = ((avg_loss * (period as f64 - 1.0)) + losses[i]) / period as f64;
        
        let rs = if avg_loss > 0.0 { avg_gain / avg_loss } else if avg_gain > 0.0 { 100.0 } else { 0.0 };
        let rsi_value = if rs.is_finite() { 100.0 - (100.0 / (1.0 + rs)) } else { 50.0 };
        rsi.push(rsi_value.clamp(0.0, 100.0));
    }

    rsi
}

// Safe version of RSI calculation with robust error handling
fn calculate_rsi_safe(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() <= period || period == 0 || period > 100 {
        return vec![];
    }

    // Validate and sanitize input data
    let valid_prices: Vec<f64> = prices.iter()
        .filter(|&&x| x.is_finite() && x > 0.0)
        .cloned()
        .collect();

    if valid_prices.len() <= period {
        return vec![];
    }

    let mut rsi = Vec::new();
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calculate price changes with validation
    for i in 1..valid_prices.len() {
        let change = valid_prices[i] - valid_prices[i - 1];
        if change.is_finite() {
            gains.push(if change > 0.0 { change } else { 0.0 });
            losses.push(if change < 0.0 { -change } else { 0.0 });
        } else {
            gains.push(0.0);
            losses.push(0.0);
        }
    }

    if gains.len() < period {
        return vec![];
    }

    // Calculate initial averages with validation
    let initial_gain_sum: f64 = gains[..period].iter().sum();
    let initial_loss_sum: f64 = losses[..period].iter().sum();
    
    if !initial_gain_sum.is_finite() || !initial_loss_sum.is_finite() {
        return vec![];
    }

    let mut avg_gain = initial_gain_sum / period as f64;
    let mut avg_loss = initial_loss_sum / period as f64;

    // Calculate first RSI with comprehensive safety checks
    let first_rsi = calculate_rsi_value_safe(avg_gain, avg_loss);
    rsi.push(first_rsi);

    // Calculate subsequent RSI values with validation
    for i in period..gains.len() {
        if !gains[i].is_finite() || !losses[i].is_finite() {
            continue;
        }

        let new_avg_gain = ((avg_gain * (period as f64 - 1.0)) + gains[i]) / period as f64;
        let new_avg_loss = ((avg_loss * (period as f64 - 1.0)) + losses[i]) / period as f64;
        
        if new_avg_gain.is_finite() && new_avg_loss.is_finite() && new_avg_gain >= 0.0 && new_avg_loss >= 0.0 {
            avg_gain = new_avg_gain;
            avg_loss = new_avg_loss;
            
            let rsi_value = calculate_rsi_value_safe(avg_gain, avg_loss);
            rsi.push(rsi_value);
        } else {
            // Use previous RSI if calculation fails
            rsi.push(*rsi.last().unwrap_or(&50.0));
        }
    }

    rsi
}

// Helper function for safe RSI value calculation
fn calculate_rsi_value_safe(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss > 0.0 {
        let rs = avg_gain / avg_loss;
        if rs.is_finite() && rs >= 0.0 {
            let rsi = 100.0 - (100.0 / (1.0 + rs));
            if rsi.is_finite() {
                return rsi.clamp(0.0, 100.0);
            }
        }
    } else if avg_gain > 0.0 {
        return 100.0; // Pure gains, maximum RSI
    }
    
    50.0 // Default neutral RSI
}

#[allow(dead_code)]
fn calculate_macd(ema_fast: &[f64], ema_slow: &[f64]) -> Vec<f64> {
    let min_len = std::cmp::min(ema_fast.len(), ema_slow.len());
    ema_fast[..min_len].iter()
        .zip(ema_slow[..min_len].iter())
        .map(|(fast, slow)| fast - slow)
        .collect()
}

// Safe version of MACD calculation
fn calculate_macd_safe(ema_fast: &[f64], ema_slow: &[f64]) -> Vec<f64> {
    if ema_fast.is_empty() || ema_slow.is_empty() {
        return vec![];
    }

    let min_len = std::cmp::min(ema_fast.len(), ema_slow.len());
    let mut macd = Vec::new();

    for i in 0..min_len {
        let fast = ema_fast[i];
        let slow = ema_slow[i];
        
        if fast.is_finite() && slow.is_finite() {
            let macd_value = fast - slow;
            if macd_value.is_finite() {
                macd.push(macd_value);
            } else {
                macd.push(0.0);
            }
        } else {
            macd.push(0.0);
        }
    }

    macd
}

#[allow(dead_code)]
fn calculate_bollinger_bands(prices: &[f64], period: usize, std_dev: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    if period == 0 || prices.len() < period {
        return (vec![], vec![], vec![]);
    }
    
    let sma = calculate_sma(prices, period);
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    
    for (i, &middle) in sma.iter().enumerate() {
        let start_idx = i + period - 1;
        let end_idx = start_idx + 1;
        
        if end_idx <= prices.len() && start_idx >= period - 1 {
            let slice_start = start_idx.saturating_sub(period.saturating_sub(1));
            let slice = &prices[slice_start..end_idx];
            
            if slice.len() == period {
                let variance = slice.iter()
                    .map(|&x| {
                        let diff = x - middle;
                        if diff.is_finite() { diff.powi(2) } else { 0.0 }
                    })
                    .sum::<f64>() / period as f64;
                
                let std = if variance >= 0.0 { variance.sqrt() } else { 0.0 };
                
                if std.is_finite() {
                    upper.push(middle + (std_dev * std));
                    lower.push(middle - (std_dev * std));
                } else {
                    upper.push(middle);
                    lower.push(middle);
                }
            }
        }
    }
    
    (upper, sma, lower)
}

// Safe version of Bollinger Bands calculation
fn calculate_bollinger_bands_safe(prices: &[f64], period: usize, std_dev: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    if period == 0 || prices.len() < period || !std_dev.is_finite() || std_dev <= 0.0 {
        return (vec![], vec![], vec![]);
    }
    
    let sma = calculate_sma_safe(prices, period);
    if sma.is_empty() {
        return (vec![], vec![], vec![]);
    }
    
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    
    for (i, &middle) in sma.iter().enumerate() {
        let start_idx = i + period - 1;
        let end_idx = start_idx + 1;
        
        if end_idx <= prices.len() && start_idx < prices.len() {
            // Saturating arithmetic to prevent underflow
            let slice_start = start_idx.saturating_sub(period.saturating_sub(1));
            let slice_end = std::cmp::min(end_idx, prices.len());
            
            if slice_start >= prices.len() || slice_end > prices.len() || slice_start >= slice_end {
                continue;
            }
            
            let slice = &prices[slice_start..slice_end];
            
            if slice.len() >= period * 2 / 3 { // Allow some tolerance for missing data
                let valid_slice: Vec<f64> = slice.iter()
                    .filter(|&&x| x.is_finite() && x > 0.0)
                    .cloned()
                    .collect();
                
                if valid_slice.len() >= period / 2 && middle.is_finite() && middle > 0.0 {
                    let variance = valid_slice.iter()
                        .map(|&x| (x - middle).powi(2))
                        .sum::<f64>() / valid_slice.len() as f64;
                    
                    if variance.is_finite() && variance >= 0.0 {
                        let std = variance.sqrt();
                        if std.is_finite() && std >= 0.0 {
                            let upper_band = middle + (std_dev * std);
                            let lower_band = middle - (std_dev * std);
                            
                            if upper_band.is_finite() && lower_band.is_finite() && upper_band > lower_band {
                                upper.push(upper_band);
                                lower.push(lower_band);
                            } else {
                                upper.push(middle);
                                lower.push(middle);
                            }
                        } else {
                            upper.push(middle);
                            lower.push(middle);
                        }
                    } else {
                        upper.push(middle);
                        lower.push(middle);
                    }
                } else {
                    upper.push(middle);
                    lower.push(middle);
                }
            }
        }
    }
    
    (upper, sma, lower)
}

fn calculate_volatility(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;
    
    variance.sqrt() * (252.0_f64).sqrt() // Annualized volatility
}

fn calculate_correlation(returns1: &[f64], returns2: &[f64]) -> f64 {
    let min_len = std::cmp::min(returns1.len(), returns2.len());
    if min_len < 2 {
        return 0.0;
    }
    
    let r1 = &returns1[..min_len];
    let r2 = &returns2[..min_len];
    
    let mean1 = r1.iter().sum::<f64>() / min_len as f64;
    let mean2 = r2.iter().sum::<f64>() / min_len as f64;
    
    let numerator: f64 = r1.iter().zip(r2.iter())
        .map(|(&x1, &x2)| (x1 - mean1) * (x2 - mean2))
        .sum();
    
    let sum_sq1: f64 = r1.iter().map(|&x| (x - mean1).powi(2)).sum();
    let sum_sq2: f64 = r2.iter().map(|&x| (x - mean2).powi(2)).sum();
    
    let denominator = (sum_sq1 * sum_sq2).sqrt();
    
    if denominator != 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

// Signal generation functions
fn get_rsi_signal(rsi: f64) -> &'static str {
    if rsi > 70.0 {
        "Overbought"
    } else if rsi < 30.0 {
        "Oversold"
    } else {
        "Neutral"
    }
}

fn get_macd_signal(macd: f64, signal: f64) -> &'static str {
    if macd > signal {
        "Bullish"
    } else if macd < signal {
        "Bearish"
    } else {
        "Neutral"
    }
}

#[allow(dead_code)]
fn get_bollinger_position(price: f64, upper: &[f64], lower: &[f64]) -> &'static str {
    if let (Some(&upper_val), Some(&lower_val)) = (upper.last(), lower.last()) {
        if price > upper_val {
            "Above Upper Band"
        } else if price < lower_val {
            "Below Lower Band"
        } else {
            "Within Bands"
        }
    } else {
        "Unknown"
    }
}

// Safe version of Bollinger position calculation
fn get_bollinger_position_safe(price: f64, upper: &[f64], lower: &[f64]) -> &'static str {
    if !price.is_finite() || price <= 0.0 {
        return "Unknown";
    }
    
    if let (Some(&upper_val), Some(&lower_val)) = (upper.last(), lower.last()) {
        if upper_val.is_finite() && lower_val.is_finite() && upper_val > lower_val {
            if price > upper_val {
                "Above Upper Band"
            } else if price < lower_val {
                "Below Lower Band"
            } else {
                "Within Bands"
            }
        } else {
            "Unknown"
        }
    } else {
        "Unknown"
    }
}

#[allow(dead_code)]
fn get_price_position(price: f64, support: f64, resistance: f64) -> &'static str {
    let range = resistance - support;
    let position = (price - support) / range;
    
    if position > 0.8 {
        "Near Resistance"
    } else if position < 0.2 {
        "Near Support"
    } else {
        "Mid-Range"
    }
}

// Safe version of price position calculation
fn get_price_position_safe(price: f64, support: f64, resistance: f64) -> &'static str {
    if !price.is_finite() || !support.is_finite() || !resistance.is_finite() {
        return "Unknown";
    }
    
    if price <= 0.0 || support <= 0.0 || resistance <= 0.0 || resistance <= support {
        return "Unknown";
    }
    
    let range = resistance - support;
    if range <= 0.0 {
        return "Unknown";
    }
    
    let position = (price - support) / range;
    if !position.is_finite() {
        return "Unknown";
    }
    
    if position > 0.8 {
        "Near Resistance"
    } else if position < 0.2 {
        "Near Support"
    } else {
        "Mid-Range"
    }
}

#[allow(dead_code)]
fn determine_overall_trend(sma: &[f64], prices: &[f64]) -> &'static str {
    if let (Some(&current_sma), Some(&current_price)) = (sma.last(), prices.first()) {
        if current_price > current_sma * 1.02 {
            "Strong Uptrend"
        } else if current_price > current_sma {
            "Uptrend"
        } else if current_price < current_sma * 0.98 {
            "Strong Downtrend"
        } else {
            "Downtrend"
        }
    } else {
        "Unknown"
    }
}

// Safe version of trend determination
fn determine_overall_trend_safe(sma: &[f64], prices: &[f64]) -> &'static str {
    if let (Some(&current_sma), Some(&current_price)) = (sma.last(), prices.first()) {
        if current_sma.is_finite() && current_price.is_finite() && current_sma > 0.0 && current_price > 0.0 {
            if current_price > current_sma * 1.02 {
                "Strong Uptrend"
            } else if current_price > current_sma {
                "Uptrend"
            } else if current_price < current_sma * 0.98 {
                "Strong Downtrend"
            } else {
                "Downtrend"
            }
        } else {
            "Unknown"
        }
    } else {
        "Unknown"
    }
}

#[allow(dead_code)]
fn generate_buy_sell_signals(data: &[crate::models::HistoricalPrice]) -> Vec<serde_json::Value> {
    let mut signals = Vec::new();
    
    if data.len() < 20 {
        return signals;
    }
    
    let prices: Vec<f64> = data.iter().map(|p| p.close.to_f64().unwrap_or(0.0)).collect();
    let sma_short = calculate_sma(&prices, 5);
    let sma_long = calculate_sma(&prices, 20);
    
    // Golden cross and death cross signals
    for i in 1..std::cmp::min(sma_short.len(), sma_long.len()) {
        let short_prev = sma_short[i - 1];
        let short_curr = sma_short[i];
        let long_prev = sma_long[i - 1];
        let long_curr = sma_long[i];
        
        if short_prev <= long_prev && short_curr > long_curr {
            signals.push(serde_json::json!({
                "type": "Golden Cross",
                "signal": "Buy",
                "strength": "Strong",
                "date": data[data.len() - sma_short.len() + i].timestamp
            }));
        } else if short_prev >= long_prev && short_curr < long_curr {
            signals.push(serde_json::json!({
                "type": "Death Cross",
                "signal": "Sell",
                "strength": "Strong",
                "date": data[data.len() - sma_short.len() + i].timestamp
            }));
        }
    }
    
    signals
}

// Safe version of buy/sell signal generation
fn generate_buy_sell_signals_safe(data: &[crate::models::HistoricalPrice]) -> Vec<serde_json::Value> {
    let mut signals = Vec::new();
    
    if data.len() < 20 {
        return signals;
    }
    
    let prices: Vec<f64> = data.iter()
        .map(|p| p.close.to_f64().unwrap_or(0.0))
        .filter(|&x| x.is_finite() && x > 0.0)
        .collect();
    
    if prices.len() < 20 {
        return signals;
    }
    
    let sma_short = calculate_sma_safe(&prices, 5);
    let sma_long = calculate_sma_safe(&prices, 20);
    
    if sma_short.is_empty() || sma_long.is_empty() {
        return signals;
    }
    
    // Golden cross and death cross signals with validation
    let min_len = std::cmp::min(sma_short.len(), sma_long.len());
    for i in 1..min_len {
        let short_prev = sma_short[i - 1];
        let short_curr = sma_short[i];
        let long_prev = sma_long[i - 1];
        let long_curr = sma_long[i];
        
        if short_prev.is_finite() && short_curr.is_finite() && long_prev.is_finite() && long_curr.is_finite() {
            if short_prev <= long_prev && short_curr > long_curr {
                // Safe index calculation to prevent overflow
                let signal_index = data.len().saturating_sub(sma_short.len()).saturating_add(i);
                if signal_index < data.len() {
                    signals.push(serde_json::json!({
                        "type": "Golden Cross",
                        "signal": "Buy",
                        "strength": "Strong",
                        "date": data[signal_index].timestamp
                    }));
                }
            } else if short_prev >= long_prev && short_curr < long_curr {
                // Safe index calculation to prevent overflow
                let signal_index = data.len().saturating_sub(sma_short.len()).saturating_add(i);
                if signal_index < data.len() {
                    signals.push(serde_json::json!({
                        "type": "Death Cross",
                        "signal": "Sell",
                        "strength": "Strong",
                        "date": data[signal_index].timestamp
                    }));
                }
            }
        }
    }
    
    signals
}

#[allow(dead_code)]
fn calculate_trend_strength(prices: &[f64], sma: &[f64]) -> &'static str {
    if let (Some(&current_price), Some(&current_sma)) = (prices.first(), sma.last()) {
        let deviation = (current_price - current_sma).abs() / current_sma;
        
        if deviation > 0.05 {
            "Strong"
        } else if deviation > 0.02 {
            "Moderate"
        } else {
            "Weak"
        }
    } else {
        "Unknown"
    }
}

// Safe version of trend strength calculation
fn calculate_trend_strength_safe(prices: &[f64], sma: &[f64]) -> &'static str {
    if let (Some(&current_price), Some(&current_sma)) = (prices.first(), sma.last()) {
        let deviation = (current_price - current_sma).abs() / current_sma;
        
        if deviation > 0.05 {
            "Strong"
        } else if deviation > 0.02 {
            "Moderate"
        } else {
            "Weak"
        }
    } else {
        "Unknown"
    }
}

// 404 handler
pub async fn handler_404() -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::error(Cow::Borrowed("Endpoint not found"))),
    )
}

// Cache cleanup endpoint (admin only)
pub async fn cleanup_cache(
    State(service): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let client_id = get_client_id();
    
    // Check rate limit
    if let Err(YahooServiceError::RateLimitExceeded) = service.check_api_rate_limit(&client_id) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    service.cleanup_cache();
    
    let response = serde_json::json!({
        "message": "Cache cleanup completed",
        "timestamp": Utc::now()
    });
    
    Ok(Json(ApiResponse::success(response)))
} 
