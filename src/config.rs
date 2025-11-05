use anyhow::Result;
use std::time::Duration;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub rate_limiting: RateLimitConfig,
    pub cache: CacheConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub api_requests_per_minute: u32,
    pub api_burst: u32,
    pub yahoo_api_requests_per_minute: u32,
    pub yahoo_api_burst: u32,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl_quotes: Duration,
    pub ttl_historical: Duration,
    pub ttl_profiles: Duration,
    pub cleanup_interval: Duration,
    pub max_size_historical: usize,
    pub max_size_quotes: usize,
    pub max_size_profiles: usize,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_all_origins: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database = DatabaseConfig {
            url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                let current_dir = std::env::current_dir().unwrap_or_default();
                let db_path = current_dir.join("data").join("data.db");
                format!("sqlite:{}", db_path.display())
            }),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
        };

        let server = ServerConfig {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
        };

        let rate_limiting = RateLimitConfig {
            api_requests_per_minute: std::env::var("API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            api_burst: std::env::var("API_RATE_LIMIT_BURST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            yahoo_api_requests_per_minute: std::env::var("YAHOO_API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120), // Increased from 30 to 120 requests/min (2 per second)
            yahoo_api_burst: std::env::var("YAHOO_API_RATE_LIMIT_BURST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        };

        let cache = CacheConfig {
            ttl_quotes: Duration::from_secs(
                std::env::var("CACHE_TTL_QUOTES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            ),
            ttl_historical: Duration::from_secs(
                std::env::var("CACHE_TTL_HISTORICAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3600),
            ),
            ttl_profiles: Duration::from_secs(
                std::env::var("CACHE_TTL_PROFILES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(86400),
            ),
            cleanup_interval: Duration::from_secs(
                std::env::var("CACHE_CLEANUP_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3600),
            ),
            max_size_historical: std::env::var("CACHE_MAX_SIZE_HISTORICAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            max_size_quotes: std::env::var("CACHE_MAX_SIZE_QUOTES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500),
            max_size_profiles: std::env::var("CACHE_MAX_SIZE_PROFILES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200),
        };

        let cors_origins = std::env::var("CORS_ALLOWED_ORIGINS").ok();
        let cors = if let Some(origins_str) = cors_origins {
            if origins_str.trim() == "*" {
                CorsConfig {
                    allowed_origins: vec![],
                    allow_all_origins: true,
                }
            } else {
                CorsConfig {
                    allowed_origins: origins_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    allow_all_origins: false,
                }
            }
        } else {
            // Default: allow all in development, but log warning
            CorsConfig {
                allowed_origins: vec![],
                allow_all_origins: true,
            }
        };

        Ok(Config {
            database,
            server,
            rate_limiting,
            cache,
            cors,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.rate_limiting.api_requests_per_minute == 0 {
            anyhow::bail!("API_RATE_LIMIT_PER_MINUTE must be greater than 0");
        }
        if self.rate_limiting.yahoo_api_requests_per_minute == 0 {
            anyhow::bail!("YAHOO_API_RATE_LIMIT_PER_MINUTE must be greater than 0");
        }
        if self.database.max_connections == 0 {
            anyhow::bail!("DATABASE_MAX_CONNECTIONS must be greater than 0");
        }
        Ok(())
    }
}

// Constants for validation
pub const MAX_SYMBOL_LENGTH: usize = 20;
pub const MAX_SEARCH_QUERY_LENGTH: usize = 100;
pub const MAX_BULK_SYMBOLS: usize = 20;
pub const MAX_COMPARE_SYMBOLS: usize = 10;
pub const DEFAULT_HISTORICAL_LIMIT: i32 = 100;
pub const MAX_HISTORICAL_LIMIT: i32 = 1000;
pub const MIN_TECHNICAL_INDICATOR_PERIODS: usize = 20;

