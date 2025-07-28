use crate::database::Database;
use crate::models::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Mutex;

use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use yahoo_finance_api::YahooConnector;

#[derive(Debug, Clone)]
pub struct CachedData<T> {
    pub data: T,
    pub timestamp: Instant,
    pub ttl: Duration,
}

impl<T> CachedData<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

#[derive(Debug, thiserror::Error)]
pub enum YahooServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub struct YahooFinanceService {
    pub db: Arc<Database>,
    provider: YahooConnector,
    // Concurrent cache using DashMap for better performance
    historical_cache: DashMap<String, CachedData<Vec<HistoricalPrice>>>,
    quote_cache: DashMap<String, CachedData<RealTimeQuote>>,
    profile_cache: DashMap<String, CachedData<Option<CompanyProfile>>>,
    // Simple rate limiting using timestamps
    api_rate_limits: Mutex<HashMap<String, Vec<Instant>>>,
    yahoo_api_calls: Mutex<Vec<Instant>>,
    // Configuration
    config: RateLimitConfig,
}

impl YahooFinanceService {
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let provider = YahooConnector::new()?;
        let config = RateLimitConfig::default();

        Ok(Self {
            db,
            provider,
            historical_cache: DashMap::new(),
            quote_cache: DashMap::new(),
            profile_cache: DashMap::new(),
            api_rate_limits: Mutex::new(HashMap::new()),
            yahoo_api_calls: Mutex::new(Vec::new()),
            config,
        })
    }

    fn get_cache_ttl(&self, interval: &str) -> Duration {
        match interval {
            "1m" | "2m" | "5m" => Duration::from_secs(60), // 1 minute for intraday
            "15m" | "30m" | "90m" => Duration::from_secs(300), // 5 minutes
            "1h" => Duration::from_secs(1800),             // 30 minutes
            "1d" => Duration::from_secs(3600),             // 1 hour for daily
            _ => Duration::from_secs(3600),                // Default 1 hour
        }
    }

    // Check API rate limit
    pub fn check_api_rate_limit(&self, client_id: &str) -> Result<(), YahooServiceError> {
        let now = Instant::now();
        let window = Duration::from_secs(60); // 1 minute window

        let mut limits = self.api_rate_limits.lock().unwrap();
        let client_calls = limits.entry(client_id.to_string()).or_default();

        // Remove old calls outside the window
        client_calls.retain(|&call_time| now.duration_since(call_time) < window);

        if client_calls.len() >= self.config.requests_per_minute as usize {
            warn!("API rate limit exceeded for client: {}", client_id);
            return Err(YahooServiceError::RateLimitExceeded);
        }

        client_calls.push(now);
        Ok(())
    }

    // Check Yahoo API rate limit
    fn check_yahoo_api_rate_limit(&self) -> Result<(), YahooServiceError> {
        let now = Instant::now();
        let window = Duration::from_secs(60); // 1 minute window

        let mut calls = self.yahoo_api_calls.lock().unwrap();

        // Remove old calls outside the window
        calls.retain(|&call_time| now.duration_since(call_time) < window);

        if calls.len() >= self.config.yahoo_api_requests_per_minute as usize {
            warn!("Yahoo API rate limit exceeded");
            return Err(YahooServiceError::RateLimitExceeded);
        }

        calls.push(now);
        Ok(())
    }

    /// Fetch and store historical data for a symbol with optimized caching
    pub async fn fetch_historical_data(
        &self,
        symbol: &str,
        interval: &str,
        force_refresh: bool,
    ) -> Result<Vec<HistoricalPrice>> {
        let _symbol_cow = Cow::Borrowed(symbol);
        let cache_key = format!("{symbol}:{interval}");

        // Check cache first (unless force refresh)
        if !force_refresh {
            if let Some(cached) = self.historical_cache.get(&cache_key) {
                if !cached.is_expired() {
                    debug!("Using cached historical data for {}", symbol);
                    return Ok(cached.data.clone());
                }
            }
        }

        info!(
            "Fetching historical data for {} with interval {}",
            symbol, interval
        );

        // Check Yahoo API rate limit
        self.check_yahoo_api_rate_limit()?;

        // Ensure symbol exists in database
        let symbol_id = self.db.upsert_symbol(symbol, None).await?;

        // Check if we already have recent data (unless force refresh)
        if !force_refresh {
            let existing_data = self
                .db
                .get_historical_prices(symbol, None, None, Some(1))
                .await?;

            if !existing_data.is_empty() {
                let latest_time = existing_data[0].timestamp;
                let now = Utc::now();
                let hours_diff = (now - latest_time).num_hours();

                // If data is less than threshold, return cached
                let refresh_threshold = match interval {
                    "1m" | "2m" | "5m" | "15m" | "30m" | "60m" | "1h" => 1,
                    _ => 24,
                };

                if hours_diff < refresh_threshold {
                    info!(
                        "Using database cached data for {} (last updated {} hours ago)",
                        symbol, hours_diff
                    );
                    let data = self
                        .db
                        .get_historical_prices(symbol, None, None, None)
                        .await?;

                    // Update memory cache
                    let ttl = self.get_cache_ttl(interval);
                    self.historical_cache
                        .insert(cache_key, CachedData::new(data.clone(), ttl));

                    return Ok(data);
                }
            }
        }

        // Fetch from Yahoo Finance API
        let response = self
            .provider
            .get_quote_range(symbol, interval, "1y")
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to fetch data from Yahoo Finance for {}: {}",
                    symbol,
                    e
                )
            })?;

        let quotes = response
            .quotes()
            .map_err(|e| anyhow!("Failed to parse quotes for {}: {}", symbol, e))?;

        // Convert Yahoo data to our format using optimized builder
        let historical_prices: Vec<HistoricalPrice> = quotes
            .iter()
            .map(|quote| HistoricalPrice::from_yahoo_quote(quote, symbol, symbol_id))
            .collect();

        // Store in database
        let inserted = self.db.insert_historical_prices(&historical_prices).await?;
        info!(
            "Inserted {} new historical price records for {}",
            inserted, symbol
        );

        // Update cache
        let ttl = self.get_cache_ttl(interval);
        self.historical_cache
            .insert(cache_key, CachedData::new(historical_prices.clone(), ttl));

        Ok(historical_prices)
    }

    /// Fetch and store company profile with optimized caching
    pub async fn fetch_company_profile(
        &self,
        symbol: &str,
        force_refresh: bool,
    ) -> Result<Option<CompanyProfile>> {
        let cache_key = symbol.to_string();

        // Check cache first
        if !force_refresh {
            if let Some(cached) = self.profile_cache.get(&cache_key) {
                if !cached.is_expired() {
                    debug!("Using cached profile for {}", symbol);
                    return Ok(cached.data.clone());
                }
            }
        }

        info!("Fetching company profile for {}", symbol);

        // Check if we already have profile data (unless force refresh)
        if !force_refresh {
            if let Some(existing_profile) = self.db.get_company_profile(symbol).await? {
                let hours_diff = (Utc::now() - existing_profile.updated_at).num_hours();
                if hours_diff < 24 {
                    info!(
                        "Using database cached profile for {} (last updated {} hours ago)",
                        symbol, hours_diff
                    );

                    // Update memory cache
                    let ttl = Duration::from_secs(24 * 3600); // 24 hours for profiles
                    self.profile_cache.insert(
                        cache_key,
                        CachedData::new(Some(existing_profile.clone()), ttl),
                    );

                    return Ok(Some(existing_profile));
                }
            }
        }

        // Check Yahoo API rate limit
        self.check_yahoo_api_rate_limit()?;

        // Ensure symbol exists in database
        let symbol_id = self.db.upsert_symbol(symbol, None).await?;

        // Try to search for the symbol to get basic info
        let search_result = self.provider.search_ticker(symbol).await;

        let company_profile = match search_result {
            Ok(search_response) => {
                if let Some(quote_summary) = search_response.quotes.first() {
                    let profile = CompanyProfile {
                        id: Uuid::new_v4(),
                        symbol_id,
                        symbol: symbol.to_string(),
                        company_name: Some(quote_summary.long_name.clone()),
                        description: None, // Not available in search API
                        sector: None,      // Not available in search API
                        industry: None,    // Not available in search API
                        employees: None,   // Not available in search API
                        website: None,     // Not available in search API
                        address: None,
                        city: None,
                        state: None,
                        country: None,
                        zip_code: None,
                        phone: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    // Store in database
                    self.db.upsert_company_profile(&profile).await?;
                    info!("Updated company profile for {}", symbol);

                    // Update cache
                    let ttl = Duration::from_secs(24 * 3600); // 24 hours
                    self.profile_cache
                        .insert(cache_key, CachedData::new(Some(profile.clone()), ttl));

                    Some(profile)
                } else {
                    warn!("No company information found for {}", symbol);

                    // Cache the None result to avoid repeated API calls
                    let ttl = Duration::from_secs(3600); // 1 hour for failed lookups
                    self.profile_cache
                        .insert(cache_key, CachedData::new(None, ttl));

                    None
                }
            }
            Err(e) => {
                warn!("Failed to search for company info for {}: {}", symbol, e);

                // Cache the None result
                let ttl = Duration::from_secs(3600);
                self.profile_cache
                    .insert(cache_key, CachedData::new(None, ttl));

                None
            }
        };

        Ok(company_profile)
    }

    /// Get historical data with smart caching and Cow optimization
    pub async fn get_historical_data(
        &self,
        symbol: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        interval: Option<&str>,
        limit: Option<i32>,
    ) -> Result<Vec<HistoricalPrice>> {
        let interval = interval.unwrap_or("1d");
        let cache_key = format!(
            "{}:{}:{}:{:?}:{:?}",
            symbol,
            interval,
            limit.unwrap_or(-1),
            start_date.map(|d| d.timestamp()),
            end_date.map(|d| d.timestamp())
        );

        // Check memory cache first
        if let Some(cached) = self.historical_cache.get(&cache_key) {
            if !cached.is_expired() {
                debug!("Using memory cached historical data for {}", symbol);
                return Ok(cached.data.clone());
            }
        }

        // First try to get from database
        let mut db_data = self
            .db
            .get_historical_prices(symbol, start_date, end_date, limit)
            .await?;

        // If we have no data or data is stale, fetch from Yahoo
        let should_fetch = if db_data.is_empty() {
            true
        } else {
            let latest_time = db_data[0].timestamp;
            let hours_diff = (Utc::now() - latest_time).num_hours();
            hours_diff > 1 // Refresh if data is more than 1 hour old
        };

        if should_fetch {
            if let Ok(fresh_data) = self.fetch_historical_data(symbol, interval, false).await {
                db_data = fresh_data;
            }
        }

        // Update memory cache
        let ttl = self.get_cache_ttl(interval);
        self.historical_cache
            .insert(cache_key, CachedData::new(db_data.clone(), ttl));

        Ok(db_data)
    }

    /// Get latest quote with caching
    pub async fn get_latest_quote(&self, symbol: &str) -> Result<Option<RealTimeQuote>> {
        let cache_key = symbol.to_string();

        // Check cache first
        if let Some(cached) = self.quote_cache.get(&cache_key) {
            if !cached.is_expired() {
                debug!("Using cached quote for {}", symbol);
                return Ok(Some(cached.data.clone()));
            }
        }

        // Try to get from database first
        if let Some(quote) = self.db.get_latest_quote(symbol).await? {
            let minutes_diff = (Utc::now() - quote.created_at).num_minutes();
            if minutes_diff < 5 {
                // Use database data if less than 5 minutes old
                let ttl = Duration::from_secs(300); // 5 minutes
                self.quote_cache
                    .insert(cache_key, CachedData::new(quote.clone(), ttl));
                return Ok(Some(quote));
            }
        }

        // Check Yahoo API rate limit
        self.check_yahoo_api_rate_limit()?;

        // Fetch fresh data from Yahoo Finance
        match self.provider.get_latest_quotes(symbol, "1d").await {
            Ok(response) => {
                if let Ok(quote_data) = response.last_quote() {
                    let symbol_id = self.db.upsert_symbol(symbol, None).await?;
                    let quote = RealTimeQuote::from_latest_quote_cow(
                        Cow::Borrowed(symbol),
                        symbol_id,
                        &quote_data,
                    );

                    // Store in database
                    if let Err(e) = self.db.insert_realtime_quote(&quote).await {
                        warn!("Failed to store real-time quote for {}: {}", symbol, e);
                    }

                    // Update cache
                    let ttl = Duration::from_secs(300); // 5 minutes
                    self.quote_cache
                        .insert(cache_key, CachedData::new(quote.clone(), ttl));

                    Ok(Some(quote))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to fetch latest quote for {}: {}", symbol, e);
                Ok(None)
            }
        }
    }

    /// Bulk fetch historical data with rate limiting
    pub async fn bulk_fetch_historical(
        &self,
        symbols: Vec<&str>,
        interval: &str,
        _max_concurrent: usize,
    ) -> Result<Vec<(String, Result<Vec<HistoricalPrice>>)>> {
        let mut results = Vec::new();

        for symbol in symbols {
            let result = self.fetch_historical_data(symbol, interval, false).await;
            results.push((symbol.to_string(), result));
        }

        Ok(results)
    }

    /// Get symbol overview with optimized data fetching
    pub async fn get_symbol_overview(&self, symbol: &str) -> Result<SymbolOverview> {
        // Fetch data concurrently
        let (latest_quote, historical_data, profile) = tokio::try_join!(
            self.get_latest_quote(symbol),
            self.get_historical_data(symbol, None, None, Some("1d"), Some(30)),
            self.fetch_company_profile(symbol, false)
        )?;

        // Calculate analytics
        let (avg_volume_30d, price_change_30d, price_change_30d_percent) =
            if historical_data.len() >= 2 {
                let avg_volume = historical_data.iter().map(|p| p.volume).sum::<i64>()
                    / historical_data.len() as i64;

                let latest_price = historical_data[0].close;
                let price_30d_ago = historical_data.last().unwrap().close;
                let price_change = latest_price - price_30d_ago;
                let price_change_percent = if price_30d_ago != Decimal::ZERO {
                    (price_change / price_30d_ago) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                (
                    Some(avg_volume),
                    Some(price_change),
                    Some(price_change_percent),
                )
            } else {
                (None, None, None)
            };

        Ok(SymbolOverview {
            symbol: symbol.to_string(),
            latest_quote,
            historical_data,
            profile,
            avg_volume_30d,
            price_change_30d,
            price_change_30d_percent,
        })
    }

    /// Validate symbol exists
    pub async fn validate_symbol(&self, symbol: &str) -> Result<bool> {
        // Check cache first
        if self.profile_cache.contains_key(symbol) || self.quote_cache.contains_key(symbol) {
            return Ok(true);
        }

        // Check database
        if self.db.get_symbol_id(symbol).await?.is_some() {
            return Ok(true);
        }

        // Check Yahoo API rate limit
        self.check_yahoo_api_rate_limit()?;

        // Try Yahoo Finance API
        match self.provider.search_ticker(symbol).await {
            Ok(response) => Ok(!response.quotes.is_empty()),
            Err(_) => Ok(false),
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let stats = self.db.get_database_stats().await?;
        Ok(serde_json::json!({
            "database": stats,
            "cache": {
                "historical_cache_size": self.historical_cache.len(),
                "quote_cache_size": self.quote_cache.len(),
                "profile_cache_size": self.profile_cache.len(),
            },
            "rate_limits": {
                "api_requests_per_minute": self.config.requests_per_minute,
                "yahoo_api_requests_per_minute": self.config.yahoo_api_requests_per_minute,
            }
        }))
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&self) {
        self.historical_cache
            .retain(|_, cached| !cached.is_expired());
        self.quote_cache.retain(|_, cached| !cached.is_expired());
        self.profile_cache.retain(|_, cached| !cached.is_expired());
    }

    // Additional optimized methods...
    pub async fn get_comprehensive_quote(&self, symbol: &str) -> Result<serde_json::Value> {
        let overview = self.get_symbol_overview(symbol).await?;

        // Create comprehensive quote with OHLC data from latest historical record
        let latest_historical = overview.historical_data.first();
        let latest_quote_with_ohlc = if let Some(hist) = latest_historical {
            serde_json::json!({
                "symbol": symbol,
                "timestamp": hist.timestamp,
                "open": hist.open,
                "high": hist.high,
                "low": hist.low,
                "close": hist.close,
                "volume": hist.volume,
                "price": hist.close,
                "market_time": hist.timestamp,
                "trading_session": "regular"
            })
        } else if let Some(ref quote) = overview.latest_quote {
            serde_json::json!({
                "symbol": symbol,
                "timestamp": quote.market_time,
                "price": quote.price,
                "volume": quote.volume,
                "market_time": quote.market_time,
                "trading_session": quote.trading_session
            })
        } else {
            serde_json::json!({
                "symbol": symbol,
                "timestamp": Utc::now(),
                "price": 0,
                "volume": 0
            })
        };

        let comprehensive = serde_json::json!({
            "symbol": overview.symbol,
            "latest_quote": latest_quote_with_ohlc,
            "profile": overview.profile,
            "data_sources": ["yahoo_finance", "database_cache"],
            "metadata": {
                "data_sources": ["yahoo_finance", "database_cache"],
                "last_updated": Utc::now(),
                "cache_status": "active"
            },
            "analysis": {
                "price_change_5d_percent": overview.price_change_30d_percent.unwrap_or_default(),
                "avg_volume_5d": overview.avg_volume_30d.unwrap_or_default(),
                "trend": if overview.price_change_30d.unwrap_or_default() > Decimal::ZERO { "bullish" } else { "bearish" },
                "volatility": "calculated",
                "volume_trend": "normal"
            },
            "analytics": {
                "avg_volume_30d": overview.avg_volume_30d,
                "price_change_30d": overview.price_change_30d,
                "price_change_30d_percent": overview.price_change_30d_percent,
                "historical_data_points": overview.historical_data.len(),
                "latest_price": overview.historical_data.first().map(|p| p.close),
                "oldest_price": overview.historical_data.last().map(|p| p.close),
            }
        });

        Ok(comprehensive)
    }

    pub async fn get_extended_quote_data(&self, symbol: &str) -> Result<serde_json::Value> {
        // Get data for multiple intervals
        let daily_data = self
            .get_historical_data(symbol, None, None, Some("1d"), Some(30))
            .await?;
        let weekly_data = self
            .get_historical_data(symbol, None, None, Some("1wk"), Some(10))
            .await?;

        // Calculate price statistics
        let all_prices: Vec<_> = daily_data.iter().map(|p| p.close).collect();
        let min_price = all_prices.iter().min().cloned().unwrap_or_default();
        let max_price = all_prices.iter().max().cloned().unwrap_or_default();
        let avg_price = if !all_prices.is_empty() {
            all_prices.iter().sum::<Decimal>() / Decimal::from(all_prices.len())
        } else {
            Decimal::ZERO
        };

        let range_percent = if min_price > Decimal::ZERO {
            ((max_price - min_price) / min_price * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let extended = serde_json::json!({
            "symbol": symbol,
            "data_sources": ["yahoo_finance", "database_cache"],
            "quotes_1d": daily_data.len(),
            "quotes_1wk": weekly_data.len(),
            "range_analysis": {
                "price_stats": {
                    "min": min_price.to_f64().unwrap_or(0.0),
                    "max": max_price.to_f64().unwrap_or(0.0),
                    "avg": avg_price.to_f64().unwrap_or(0.0),
                    "range_percent": range_percent
                }
            },
            "intervals": {
                "1d": {
                    "data_points": daily_data.len(),
                    "latest_price": daily_data.first().map(|p| p.close),
                    "price_range": {
                        "high": daily_data.iter().map(|p| p.high).max(),
                        "low": daily_data.iter().map(|p| p.low).min(),
                    }
                },
                "1wk": {
                    "data_points": weekly_data.len(),
                    "latest_price": weekly_data.first().map(|p| p.close),
                    "price_range": {
                        "high": weekly_data.iter().map(|p| p.high).max(),
                        "low": weekly_data.iter().map(|p| p.low).min(),
                    }
                }
            }
        });

        Ok(extended)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOverview {
    pub symbol: String,
    pub latest_quote: Option<RealTimeQuote>,
    pub historical_data: Vec<HistoricalPrice>,
    pub profile: Option<CompanyProfile>,
    pub avg_volume_30d: Option<i64>,
    pub price_change_30d: Option<Decimal>,
    pub price_change_30d_percent: Option<Decimal>,
}
