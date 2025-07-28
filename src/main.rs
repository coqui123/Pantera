use anyhow::Result;
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod handlers;
mod models;
mod yahoo_service;
mod web_ui;

use database::Database;
use handlers::{
    health_check, get_symbols, search_symbols, validate_symbol,
    get_historical_data, fetch_historical_data, bulk_fetch_historical,
    get_real_time_quote, get_company_profile, get_symbol_overview,
    get_price_analysis, get_database_stats, get_comprehensive_quote,
    get_extended_quote_data, handler_404, cleanup_cache,
    get_technical_indicators, compare_symbols,
};
use yahoo_service::YahooFinanceService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mango_data_service=info,tower_http=info,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Mango Data Service with optimizations");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database URL - use SQLite by default with proper path
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            let current_dir = std::env::current_dir().unwrap_or_default();
            let db_path = current_dir.join("data").join("data.db");
            format!("sqlite:{}", db_path.display())
        });
    
    info!("Connecting to database: {}", database_url);

    // Initialize database
    let db = Database::new(&database_url).await?;
    info!("✅ Database initialized successfully");

    // Create Yahoo Finance service with optimizations
    let yahoo_service = Arc::new(YahooFinanceService::new(Arc::new(db))?);
    info!("✅ Yahoo Finance service initialized with rate limiting and caching");

    // Start background cache cleanup task
    let cleanup_service = yahoo_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            cleanup_service.cleanup_cache();
            info!("🧹 Cache cleanup completed");
        }
    });

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // Build the application with optimized routes
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Symbol management
        .route("/api/symbols", get(get_symbols))
        .route("/api/symbols/search", get(search_symbols))
        .route("/api/symbols/:symbol/validate", get(validate_symbol))
        
        // Historical data
        .route("/api/symbols/:symbol/historical", get(get_historical_data))
        .route("/api/symbols/:symbol/fetch", post(fetch_historical_data))
        .route("/api/bulk/historical", get(bulk_fetch_historical))
        
        // Real-time quotes
        .route("/api/symbols/:symbol/quote", get(get_real_time_quote))
        
        // Company profiles
        .route("/api/symbols/:symbol/profile", get(get_company_profile))
        
        // Comprehensive data
        .route("/api/symbols/:symbol/overview", get(get_symbol_overview))
        .route("/api/symbols/:symbol/analysis", get(get_price_analysis))
        .route("/api/symbols/:symbol/comprehensive", get(get_comprehensive_quote))
        .route("/api/symbols/:symbol/extended", get(get_extended_quote_data))
        .route("/api/symbols/:symbol/indicators", get(get_technical_indicators))
        
        // Comparison and advanced analytics
        .route("/api/compare", get(compare_symbols))
        
        // Statistics and monitoring
        .route("/api/stats", get(get_database_stats))
        
        // Admin endpoints
        .route("/api/admin/cache/cleanup", post(cleanup_cache));
        
    // Add web UI routes if feature is enabled
    #[cfg(feature = "web-ui")]
    let app = app
        .route("/ui", get(web_ui::dashboard))
        .route("/ui/search", get(web_ui::search))
        .route("/ui/analytics", get(web_ui::analytics))
        .route("/", get(web_ui::dashboard)); // Root redirects to dashboard
        
    // Add basic API info route when web-ui is disabled
    #[cfg(not(feature = "web-ui"))]
    let app = app
        .route("/", get(|| async { "Mango Data Service API - Use /health for status or /api/* for endpoints" }));
        
    let app = app
        // Fallback for 404
        .fallback(handler_404)
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        )
        
        // Add shared state
        .with_state(yahoo_service);

    // Start the server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{port}");
    info!("🌐 Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("🚀 Mango Data Service is running on http://{}", addr);
    
    // Print available endpoints with optimization info
    print_api_info();

    axum::serve(listener, app).await?;

    Ok(())
}

fn print_api_info() {
    info!("🔗 Available API Endpoints (with optimizations):");
    info!("  ⚡ Features: Rate Limiting, Cow Optimization, Concurrent Caching");
    info!("");
    
    #[cfg(feature = "web-ui")]
    {
        info!("  Web Interface:");
        info!("    GET  /                              - Dashboard (default)");
        info!("    GET  /ui                            - Interactive Dashboard");
        info!("    GET  /ui/search                     - Symbol Search Interface");
        info!("    GET  /ui/analytics                  - Financial Analytics Interface");
        info!("");
    }
    
    info!("  Health Check:");
    info!("    GET  /health");
    info!("");
    info!("  Symbol Management:");
    info!("    GET  /api/symbols                    - List all symbols");
    info!("    GET  /api/symbols/search?q=QUERY    - Search symbols (optimized)");
    info!("    GET  /api/symbols/{{symbol}}/validate  - Validate symbol (cached)");
    info!("");
    info!("  Historical Data:");
    info!("    GET  /api/symbols/{{symbol}}/historical?interval=1d&limit=100&force_refresh=false");
    info!("    POST /api/symbols/{{symbol}}/fetch?interval=1d");
    info!("    GET  /api/bulk/historical?symbols=AAPL,MSFT&interval=1d&max_concurrent=5");
    info!("");
    info!("  Real-time Data:");
    info!("    GET  /api/symbols/{{symbol}}/quote     - Latest quote (cached)");
    info!("");
    info!("  Company Information:");
    info!("    GET  /api/symbols/{{symbol}}/profile   - Company profile (cached)");
    info!("");
    info!("  Analytics:");
    info!("    GET  /api/symbols/{{symbol}}/overview  - Comprehensive overview");
    info!("    GET  /api/symbols/{{symbol}}/analysis?limit=30 - Price analysis (optimized)");
    info!("    GET  /api/symbols/{{symbol}}/comprehensive - Comprehensive quote");
    info!("    GET  /api/symbols/{{symbol}}/extended - Extended quote data");
    info!("    GET  /api/symbols/{{symbol}}/indicators - Technical indicators");
    info!("");
    info!("  Comparison:");
    info!("    GET  /api/compare?symbol1=AAPL&symbol2=MSFT - Compare two symbols");
    info!("");
    info!("  System:");
    info!("    GET  /api/stats                      - Database & cache statistics");
    info!("    POST /api/admin/cache/cleanup        - Manual cache cleanup");
    info!("");
    info!("  🛡️  Rate Limits:");
    info!("    - API: 100 requests/minute (burst: 10)");
    info!("    - Yahoo API: 30 requests/minute (burst: 5)");
    info!("");
    
    #[cfg(feature = "web-ui")]
    {
        info!("💡 Web Interface usage:");
        info!("  Open http://localhost:3000 in your browser for interactive demo");
    }
    
    info!("💡 API usage examples:");
    info!("  curl http://localhost:3000/api/symbols/AAPL/historical?interval=1d&limit=10");
    info!("  curl http://localhost:3000/api/symbols/AAPL/overview");
    info!("  curl -X POST http://localhost:3000/api/symbols/AAPL/fetch?interval=1d");
    info!("  curl http://localhost:3000/api/symbols/search?q=apple&limit=5");
    info!("  curl http://localhost:3000/api/compare?symbol1=AAPL&symbol2=MSFT");
    info!("");
    info!("🔧 Optimizations Active:");
    info!("  - Cow (Clone on Write) for zero-copy string operations");
    info!("  - DashMap for concurrent caching");
    info!("  - Governor for rate limiting");
    info!("  - Automatic cache cleanup every hour");
    info!("  - Optimized bulk operations with semaphore control");
} 
