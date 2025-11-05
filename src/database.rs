use crate::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

pub type DbPool = Pool<Sqlite>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        // Handle SQLite-specific setup
        let processed_url = if database_url.starts_with("sqlite:") {
            // Extract the file path from the URL
            let file_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);

            // If it's not an in-memory database, ensure the directory exists
            if file_path != ":memory:" && !file_path.is_empty() {
                if let Some(parent) = std::path::Path::new(file_path).parent() {
                    info!("Creating directory: {:?}", parent);
                    std::fs::create_dir_all(parent)?;
                    info!("Directory created successfully");
                }

                // Check if we can create the file
                info!("Attempting to create database file: {}", file_path);
                if let Err(e) = std::fs::File::create(file_path) {
                    info!("Failed to create database file: {}", e);
                } else {
                    info!("Database file created successfully");
                }
            }

            database_url.to_string()
        } else {
            database_url.to_string()
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect(&processed_url)
            .await?;

        let db = Database { pool };
        db.run_migrations().await?;
        db.create_indexes().await?;

        Ok(db)
    }

    #[allow(dead_code)]
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create symbols table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS symbols (
                id TEXT PRIMARY KEY,
                symbol TEXT UNIQUE NOT NULL,
                name TEXT,
                exchange TEXT,
                sector TEXT,
                industry TEXT,
                market_cap TEXT, -- Decimal stored as TEXT
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create historical_prices table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS historical_prices (
                id TEXT PRIMARY KEY,
                symbol_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                open TEXT NOT NULL, -- Decimal stored as TEXT
                high TEXT NOT NULL,
                low TEXT NOT NULL,
                close TEXT NOT NULL,
                adjusted_close TEXT,
                volume INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (symbol_id) REFERENCES symbols (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create realtime_quotes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS realtime_quotes (
                id TEXT PRIMARY KEY,
                symbol_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                price TEXT NOT NULL, -- Decimal stored as TEXT
                change TEXT,
                change_percent TEXT,
                volume INTEGER,
                market_time TEXT NOT NULL,
                trading_session TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (symbol_id) REFERENCES symbols (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create company_profiles table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS company_profiles (
                id TEXT PRIMARY KEY,
                symbol_id TEXT NOT NULL,
                symbol TEXT UNIQUE NOT NULL,
                company_name TEXT,
                description TEXT,
                sector TEXT,
                industry TEXT,
                employees INTEGER,
                website TEXT,
                address TEXT,
                city TEXT,
                state TEXT,
                country TEXT,
                zip_code TEXT,
                phone TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (symbol_id) REFERENCES symbols (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        info!("Creating database indexes...");

        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_symbols_symbol ON symbols (symbol)",
            "CREATE INDEX IF NOT EXISTS idx_historical_prices_symbol ON historical_prices (symbol)",
            "CREATE INDEX IF NOT EXISTS idx_historical_prices_timestamp ON historical_prices (timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_historical_prices_symbol_timestamp ON historical_prices (symbol, timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_realtime_quotes_symbol ON realtime_quotes (symbol)",
            "CREATE INDEX IF NOT EXISTS idx_realtime_quotes_market_time ON realtime_quotes (market_time)",
            "CREATE INDEX IF NOT EXISTS idx_company_profiles_symbol ON company_profiles (symbol)",
        ];

        for index in indexes {
            sqlx::query(index).execute(&self.pool).await?;
        }

        info!("Database indexes created successfully");
        Ok(())
    }

    // Symbol operations
    pub async fn upsert_symbol(&self, symbol: &str, name: Option<&str>) -> Result<Uuid> {
        let symbol_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO symbols (id, symbol, name, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(symbol) DO UPDATE SET
                name = COALESCE(?3, name),
                updated_at = ?5
            "#,
        )
        .bind(symbol_id.to_string())
        .bind(symbol)
        .bind(name)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Get the actual symbol_id (might be existing one)
        let existing_id: String = sqlx::query_scalar("SELECT id FROM symbols WHERE symbol = ?1")
            .bind(symbol)
            .fetch_one(&self.pool)
            .await?;

        Ok(Uuid::from_str(&existing_id)?)
    }

    pub async fn get_symbol_id(&self, symbol: &str) -> Result<Option<Uuid>> {
        let result: Option<String> = sqlx::query_scalar("SELECT id FROM symbols WHERE symbol = ?1")
            .bind(symbol)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(id_str) => Ok(Some(Uuid::from_str(&id_str)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_symbols(&self) -> Result<Vec<Symbol>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String)>(
            "SELECT id, symbol, name, exchange, sector, industry, market_cap, created_at, updated_at FROM symbols ORDER BY symbol"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut symbols = Vec::new();
        for row in rows {
            symbols.push(Symbol {
                id: Uuid::from_str(&row.0)?,
                symbol: row.1,
                name: row.2,
                exchange: row.3,
                sector: row.4,
                industry: row.5,
                market_cap: row.6.as_ref().and_then(|s| Decimal::from_str(s).ok()),
                created_at: DateTime::parse_from_rfc3339(&row.7)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.8)?.with_timezone(&Utc),
            });
        }

        Ok(symbols)
    }

    pub async fn search_symbols(&self, query: &str, limit: i32) -> Result<Vec<Symbol>> {
        let search_pattern = format!("%{}%", query.to_uppercase());
        let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String)>(
            "SELECT id, symbol, name, exchange, sector, industry, market_cap, created_at, updated_at 
             FROM symbols 
             WHERE UPPER(symbol) LIKE ?1 OR UPPER(COALESCE(name, '')) LIKE ?1 
             ORDER BY symbol 
             LIMIT ?2"
        )
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut symbols = Vec::new();
        for row in rows {
            symbols.push(Symbol {
                id: Uuid::from_str(&row.0)?,
                symbol: row.1,
                name: row.2,
                exchange: row.3,
                sector: row.4,
                industry: row.5,
                market_cap: row.6.as_ref().and_then(|s| Decimal::from_str(s).ok()),
                created_at: DateTime::parse_from_rfc3339(&row.7)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.8)?.with_timezone(&Utc),
            });
        }

        Ok(symbols)
    }

    // Historical price operations
    pub async fn insert_historical_prices(&self, prices: &[HistoricalPrice]) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;

        for price in prices {
            let result = sqlx::query(
                r#"
                INSERT OR IGNORE INTO historical_prices 
                (id, symbol_id, symbol, timestamp, open, high, low, close, adjusted_close, volume, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
            )
            .bind(price.id.to_string())
            .bind(price.symbol_id.to_string())
            .bind(&price.symbol)
            .bind(price.timestamp.to_rfc3339())
            .bind(price.open.to_string())
            .bind(price.high.to_string())
            .bind(price.low.to_string())
            .bind(price.close.to_string())
            .bind(price.adjusted_close.as_ref().map(|d| d.to_string()))
            .bind(price.volume)
            .bind(price.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() > 0 {
                inserted += 1;
            }
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn get_historical_prices(
        &self,
        symbol: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<i32>,
    ) -> Result<Vec<HistoricalPrice>> {
        let mut query = String::from(
            "SELECT id, symbol_id, symbol, timestamp, open, high, low, close, adjusted_close, volume, created_at 
             FROM historical_prices WHERE symbol = ?1"
        );

        let mut bind_count = 1;
        if start_date.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND timestamp >= ?{bind_count}"));
        }
        if end_date.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND timestamp <= ?{bind_count}"));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(_limit) = limit {
            bind_count += 1;
            query.push_str(&format!(" LIMIT ?{bind_count}"));
        }

        let mut sqlx_query = sqlx::query(&query).bind(symbol);

        if let Some(start) = start_date {
            sqlx_query = sqlx_query.bind(start.to_rfc3339());
        }
        if let Some(end) = end_date {
            sqlx_query = sqlx_query.bind(end.to_rfc3339());
        }
        if let Some(limit) = limit {
            sqlx_query = sqlx_query.bind(limit);
        }

        let rows = sqlx_query.fetch_all(&self.pool).await?;

        let mut prices = Vec::new();
        for row in rows {
            prices.push(HistoricalPrice {
                id: Uuid::from_str(&row.get::<String, _>(0))?,
                symbol_id: Uuid::from_str(&row.get::<String, _>(1))?,
                symbol: row.get(2),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>(3))?
                    .with_timezone(&Utc),
                open: Decimal::from_str(&row.get::<String, _>(4))?,
                high: Decimal::from_str(&row.get::<String, _>(5))?,
                low: Decimal::from_str(&row.get::<String, _>(6))?,
                close: Decimal::from_str(&row.get::<String, _>(7))?,
                adjusted_close: row
                    .get::<Option<String>, _>(8)
                    .as_ref()
                    .and_then(|s| Decimal::from_str(s).ok()),
                volume: row.get(9),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>(10))?
                    .with_timezone(&Utc),
            });
        }

        Ok(prices)
    }

    // Real-time quote operations
    pub async fn insert_realtime_quote(&self, quote: &RealTimeQuote) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO realtime_quotes 
            (id, symbol_id, symbol, price, change, change_percent, volume, market_time, trading_session, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(quote.id.to_string())
        .bind(quote.symbol_id.to_string())
        .bind(&quote.symbol)
        .bind(quote.price.to_string())
        .bind(quote.change.as_ref().map(|d| d.to_string()))
        .bind(quote.change_percent.as_ref().map(|d| d.to_string()))
        .bind(quote.volume)
        .bind(quote.market_time.to_rfc3339())
        .bind(&quote.trading_session)
        .bind(quote.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_quote(&self, symbol: &str) -> Result<Option<RealTimeQuote>> {
        let row = sqlx::query(
            "SELECT id, symbol_id, symbol, price, change, change_percent, volume, market_time, trading_session, created_at 
             FROM realtime_quotes 
             WHERE symbol = ?1 
             ORDER BY market_time DESC 
             LIMIT 1"
        )
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(RealTimeQuote {
                id: Uuid::from_str(&row.get::<String, _>(0))?,
                symbol_id: Uuid::from_str(&row.get::<String, _>(1))?,
                symbol: row.get(2),
                price: Decimal::from_str(&row.get::<String, _>(3))?,
                change: row
                    .get::<Option<String>, _>(4)
                    .as_ref()
                    .and_then(|s| Decimal::from_str(s).ok()),
                change_percent: row
                    .get::<Option<String>, _>(5)
                    .as_ref()
                    .and_then(|s| Decimal::from_str(s).ok()),
                volume: row.get(6),
                market_time: DateTime::parse_from_rfc3339(&row.get::<String, _>(7))?
                    .with_timezone(&Utc),
                trading_session: row.get(8),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>(9))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    // Company profile operations
    pub async fn upsert_company_profile(&self, profile: &CompanyProfile) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO company_profiles 
            (id, symbol_id, symbol, company_name, description, sector, industry, employees, 
             website, address, city, state, country, zip_code, phone, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(symbol) DO UPDATE SET
                company_name = COALESCE(?4, company_name),
                description = COALESCE(?5, description),
                sector = COALESCE(?6, sector),
                industry = COALESCE(?7, industry),
                employees = COALESCE(?8, employees),
                website = COALESCE(?9, website),
                address = COALESCE(?10, address),
                city = COALESCE(?11, city),
                state = COALESCE(?12, state),
                country = COALESCE(?13, country),
                zip_code = COALESCE(?14, zip_code),
                phone = COALESCE(?15, phone),
                updated_at = ?17
            "#,
        )
        .bind(profile.id.to_string())
        .bind(profile.symbol_id.to_string())
        .bind(&profile.symbol)
        .bind(&profile.company_name)
        .bind(&profile.description)
        .bind(&profile.sector)
        .bind(&profile.industry)
        .bind(profile.employees)
        .bind(&profile.website)
        .bind(&profile.address)
        .bind(&profile.city)
        .bind(&profile.state)
        .bind(&profile.country)
        .bind(&profile.zip_code)
        .bind(&profile.phone)
        .bind(profile.created_at.to_rfc3339())
        .bind(profile.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_company_profile(&self, symbol: &str) -> Result<Option<CompanyProfile>> {
        let row = sqlx::query(
            "SELECT id, symbol_id, symbol, company_name, description, sector, industry, employees, 
             website, address, city, state, country, zip_code, phone, created_at, updated_at
             FROM company_profiles 
             WHERE symbol = ?1",
        )
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(CompanyProfile {
                id: Uuid::from_str(&row.get::<String, _>(0))?,
                symbol_id: Uuid::from_str(&row.get::<String, _>(1))?,
                symbol: row.get(2),
                company_name: row.get(3),
                description: row.get(4),
                sector: row.get(5),
                industry: row.get(6),
                employees: row.get(7),
                website: row.get(8),
                address: row.get(9),
                city: row.get(10),
                state: row.get(11),
                country: row.get(12),
                zip_code: row.get(13),
                phone: row.get(14),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>(15))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>(16))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    // Analytics and utility functions
    pub async fn get_database_stats(&self) -> Result<serde_json::Value> {
        let symbols_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM symbols")
            .fetch_one(&self.pool)
            .await?;

        let historical_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM historical_prices")
            .fetch_one(&self.pool)
            .await?;

        let quotes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM realtime_quotes")
            .fetch_one(&self.pool)
            .await?;

        let profiles_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM company_profiles")
            .fetch_one(&self.pool)
            .await?;

        Ok(serde_json::json!({
            "symbols_count": symbols_count,
            "historical_records_count": historical_count,
            "realtime_quotes_count": quotes_count,
            "company_profiles_count": profiles_count,
            "symbols": symbols_count,
            "historical_prices": historical_count,
            "realtime_quotes": quotes_count,
            "company_profiles": profiles_count,
            "timestamp": Utc::now()
        }))
    }
}
