use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Symbol {
    pub id: Uuid,
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HistoricalPrice {
    pub id: Uuid,
    pub symbol_id: Uuid,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub adjusted_close: Option<Decimal>,
    pub volume: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RealTimeQuote {
    pub id: Uuid,
    pub symbol_id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub change: Option<Decimal>,
    pub change_percent: Option<Decimal>,
    pub volume: Option<i64>,
    pub market_time: DateTime<Utc>,
    pub trading_session: String, // "regular", "pre", "post"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompanyProfile {
    pub id: Uuid,
    pub symbol_id: Uuid,
    pub symbol: String,
    pub company_name: Option<String>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub employees: Option<i32>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Optimized request structures using Cow for zero-copy when possible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRequest<'a> {
    pub symbol: Cow<'a, str>,
    pub interval: Option<Cow<'a, str>>, // "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// Optimized response structures using Cow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse<'a> {
    pub symbol: Cow<'a, str>,
    pub price: Decimal,
    pub change: Option<Decimal>,
    pub change_percent: Option<Decimal>,
    pub volume: Option<i64>,
    pub market_time: DateTime<Utc>,
    pub trading_session: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalResponse<'a> {
    pub symbol: Cow<'a, str>,
    pub data: Vec<HistoricalPrice>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse<'a> {
    pub symbol: Cow<'a, str>,
    pub profile: Option<CompanyProfile>,
}

// Data transfer objects for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<Cow<'static, str>>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

// Optimized builder pattern for HistoricalPrice to avoid unnecessary allocations
pub struct HistoricalPriceBuilder<'a> {
    symbol: Cow<'a, str>,
    symbol_id: Uuid,
    timestamp: DateTime<Utc>,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    adjusted_close: Option<Decimal>,
    volume: i64,
}

impl<'a> HistoricalPriceBuilder<'a> {
    pub fn new(symbol: impl Into<Cow<'a, str>>, symbol_id: Uuid) -> Self {
        Self {
            symbol: symbol.into(),
            symbol_id,
            timestamp: Utc::now(),
            open: Decimal::ZERO,
            high: Decimal::ZERO,
            low: Decimal::ZERO,
            close: Decimal::ZERO,
            adjusted_close: None,
            volume: 0,
        }
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn prices(mut self, open: Decimal, high: Decimal, low: Decimal, close: Decimal) -> Self {
        self.open = open;
        self.high = high;
        self.low = low;
        self.close = close;
        self
    }

    pub fn adjusted_close(mut self, adjusted_close: Option<Decimal>) -> Self {
        self.adjusted_close = adjusted_close;
        self
    }

    pub fn volume(mut self, volume: i64) -> Self {
        self.volume = volume;
        self
    }

    pub fn build(self) -> HistoricalPrice {
        HistoricalPrice {
            id: Uuid::new_v4(),
            symbol_id: self.symbol_id,
            symbol: self.symbol.into_owned(),
            timestamp: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            adjusted_close: self.adjusted_close,
            volume: self.volume,
            created_at: Utc::now(),
        }
    }
}

// Conversion implementations for Yahoo Finance API types
impl From<&yahoo_finance_api::Quote> for HistoricalPrice {
    fn from(quote: &yahoo_finance_api::Quote) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol_id: Uuid::new_v4(), // This should be looked up from symbols table
            symbol: String::new(),     // This should be set externally
            timestamp: DateTime::from_timestamp(quote.timestamp as i64, 0)
                .unwrap_or_default()
                .with_timezone(&Utc),
            open: Decimal::from_f64_retain(quote.open).unwrap_or_default(),
            high: Decimal::from_f64_retain(quote.high).unwrap_or_default(),
            low: Decimal::from_f64_retain(quote.low).unwrap_or_default(),
            close: Decimal::from_f64_retain(quote.close).unwrap_or_default(),
            adjusted_close: Some(Decimal::from_f64_retain(quote.adjclose).unwrap_or_default()),
            volume: quote.volume as i64,
            created_at: Utc::now(),
        }
    }
}

// Optimized conversion using builder pattern
impl HistoricalPrice {
    pub fn from_yahoo_quote(
        quote: &yahoo_finance_api::Quote,
        symbol: &str,
        symbol_id: Uuid,
    ) -> Self {
        HistoricalPriceBuilder::new(symbol, symbol_id)
            .timestamp(
                DateTime::from_timestamp(quote.timestamp as i64, 0)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
            )
            .prices(
                Decimal::from_f64_retain(quote.open).unwrap_or_default(),
                Decimal::from_f64_retain(quote.high).unwrap_or_default(),
                Decimal::from_f64_retain(quote.low).unwrap_or_default(),
                Decimal::from_f64_retain(quote.close).unwrap_or_default(),
            )
            .adjusted_close(Some(
                Decimal::from_f64_retain(quote.adjclose).unwrap_or_default(),
            ))
            .volume(quote.volume as i64)
            .build()
    }
}

// Helper function to create RealTimeQuote from latest quote data
impl RealTimeQuote {
    pub fn from_latest_quote_cow(
        symbol: Cow<'_, str>,
        symbol_id: Uuid,
        quote: &yahoo_finance_api::Quote,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol_id,
            symbol: symbol.into_owned(),
            price: Decimal::from_f64_retain(quote.close).unwrap_or_default(),
            change: None,
            change_percent: None,
            volume: Some(quote.volume as i64),
            market_time: DateTime::from_timestamp(quote.timestamp as i64, 0)
                .unwrap_or_default()
                .with_timezone(&Utc),
            trading_session: "regular".to_string(),
            created_at: Utc::now(),
        }
    }
}

// Portfolio models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHolding {
    pub id: Uuid,
    pub symbol: String,
    pub symbol_id: Option<Uuid>,
    pub asset_type: String, // "stock", "etf", "crypto"
    pub quantity: Decimal,
    pub purchase_price: Decimal,
    pub current_price: Option<Decimal>,
    pub current_value: Option<Decimal>,
    pub gain_loss: Option<Decimal>,
    pub gain_loss_percent: Option<Decimal>,
    pub last_updated: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHoldingWithQuote {
    pub holding: PortfolioHolding,
    pub quote: Option<RealTimeQuote>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_holdings: usize,
    pub total_cost: Decimal,
    pub total_value: Decimal,
    pub total_gain_loss: Decimal,
    pub total_gain_loss_percent: Decimal,
    pub holdings: Vec<PortfolioHoldingWithQuote>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddHoldingRequest {
    pub symbol: String,
    #[serde(default)]
    pub asset_type: Option<String>, // Optional: "stock", "etf", "crypto" - auto-detected if not provided
    pub quantity: Decimal,
    #[serde(default)]
    pub purchase_price: Option<Decimal>, // Optional: will use current price if not provided
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHoldingRequest {
    pub quantity: Option<Decimal>,
    pub purchase_price: Option<Decimal>,
}

// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub yahoo_api_requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            yahoo_api_requests_per_minute: 30, // Conservative limit for Yahoo Finance API
        }
    }
}
