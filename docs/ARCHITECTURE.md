# 🏗️ Architecture Documentation

Comprehensive technical architecture documentation for Mango Data Service - a high-performance financial data platform with optional professional web interface.

## System Overview

Mango Data Service is a high-performance financial data API built with Rust, designed for scalability, performance, and reliability. The system implements advanced optimizations including zero-copy operations, concurrent caching, intelligent rate limiting, and an optional professional-grade web interface for financial analysis.

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Browser   │    │   API Clients   │    │  Python/Node.js │
│  (Web Interface)│    │                 │    │     Scripts     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │    Mango Data Service    │
                    │                          │
                    │  ┌─────────┐ ┌─────────┐ │
                    │  │   API   │ │ Web UI  │ │ (Optional)
                    │  │ Routes  │ │ Routes  │ │
                    │  └─────────┘ └─────────┘ │
                    │                          │
                    │ ┌──────────────────────┐ │
                    │ │   Askama Templates   │ │ (web-ui feature)
                    │ └──────────────────────┘ │
                    └─────────────┬─────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
┌─────────▼───────┐    ┌─────────▼───────┐    ┌─────────▼───────┐
│   Rate Limiter  │    │  Memory Cache   │    │    Database     │
│  (Token Bucket) │    │   (DashMap)     │    │ SQLite/Postgres │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │   Yahoo Finance API      │
                    │    (External Service)    │
                    └─────────────────────────────┘
```

## Core Components

### 1. Web Server Layer (`main.rs`)

**Technology**: Axum web framework with Tokio async runtime

**Responsibilities**:
- HTTP server setup and configuration
- Route registration (API and web interface)
- Background task management
- Graceful shutdown handling
- CORS configuration
- Feature flag integration

**Key Features**:
```rust
// Server setup with conditional web UI routes
let app = Router::new()
    .route("/health", get(health_check))
    .route("/api/symbols/:symbol/comprehensive", get(get_comprehensive_quote))
    .layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .layer(GovernorLayer {
                config: Box::leak(governor_conf),
            })
    );

// Conditional web UI routes
#[cfg(feature = "web-ui")]
let app = app
    .route("/", get(web_ui::dashboard))
    .route("/ui", get(web_ui::dashboard))
    .route("/ui/search", get(web_ui::search))
    .route("/ui/analytics", get(web_ui::analytics));
```

**Background Tasks**:
- Cache cleanup (every hour)
- Health monitoring
- Metrics collection

### 2. Request Handlers (`handlers.rs`)

**Technology**: Axum handlers with rate limiting middleware

**Responsibilities**:
- HTTP request processing for API endpoints
- Input validation and sanitization
- Rate limiting enforcement
- Response formatting
- Error handling

**Rate Limiting Implementation**:
```rust
// Per-client IP rate limiting
async fn check_rate_limit(client_ip: &str) -> Result<(), AppError> {
    let mut limiter = API_RATE_LIMITER.lock().await;
    if !limiter.check_key(&client_ip).is_ok() {
        return Err(AppError::RateLimitExceeded);
    }
    Ok(())
}
```

**Key Optimizations**:
- Cow-based string handling for zero-copy operations
- Concurrent request processing
- Efficient error propagation

### 3. Web Interface Layer (`web_ui.rs`) - Optional Feature

**Technology**: Askama templating with feature-gated compilation

**Responsibilities**:
- Web interface request handling
- Template data preparation and rendering
- Frontend-backend data integration
- Progressive enhancement for API functionality

**Template Handler Pattern**:
```rust
#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: Cow<'static, str>,
    pub symbols: Vec<Symbol>,
    pub metrics: SystemMetrics,
}

#[cfg(feature = "web-ui")]
pub async fn dashboard() -> impl IntoResponse {
    let symbols = get_popular_symbols().await;
    let metrics = get_system_metrics().await;
    
    DashboardTemplate {
        title: "Financial Dashboard".into(),
        symbols,
        metrics,
    }
}

// Fallback for disabled feature
#[cfg(not(feature = "web-ui"))]
pub async fn dashboard() -> Result<&'static str, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}
```

**Key Features**:
- Server-side rendering with compile-time template checking
- Efficient data binding with Cow optimizations
- Feature-gated compilation for optional deployment
- Progressive enhancement of API functionality

### 4. Template System (`templates/`) - Web-UI Feature

**Technology**: Askama templating engine with Tailwind CSS and Chart.js

**Template Hierarchy**:
```
templates/
├── base.html              # Common layout, navigation, CSS/JS includes
├── dashboard.html         # System overview and quick actions
├── search.html           # Symbol search and management
└── analytics.html        # Advanced charting and technical analysis
```

**Template Architecture**:
```html
<!-- base.html - Foundation template -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mango Data Service{% endblock %}</title>
    
    <!-- Tailwind CSS for responsive design -->
    <link href="https://cdnjs.cloudflare.com/ajax/libs/tailwindcss/2.2.19/tailwind.min.css" rel="stylesheet">
    
    <!-- FontAwesome for icons -->
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css" rel="stylesheet">
    
    <!-- Chart.js for financial charts -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.9.1/chart.min.js"></script>
</head>
<body class="bg-gray-50 min-h-screen">
    {% include "navigation.html" %}
    
    <main class="container mx-auto px-4 py-8">
        {% block content %}{% endblock %}
    </main>
    
    {% block scripts %}{% endblock %}
</body>
</html>
```

**Template Inheritance and Composition**:
- **Base Template**: Provides common layout, styling, and JavaScript libraries
- **Content Templates**: Extend base template with specific functionality
- **Component Templates**: Reusable UI components (navigation, forms, charts)
- **Data Binding**: Efficient data rendering with automatic escaping

### 5. Data Models (`models.rs`)

**Technology**: Serde with Cow optimizations

**Responsibilities**:
- Data structure definitions
- Serialization/deserialization for both API and templates
- Type safety and validation
- Memory-efficient string handling

**Cow Implementation for Web Interface**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<Cow<'static, str>>,
    pub timestamp: DateTime<Utc>,
}

// Template-optimized data structures
#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub symbols: Cow<'_, [Symbol]>,      // Can be borrowed from cache
    pub metrics: SystemMetrics,          // Owned when computed
    pub status: Cow<'static, str>,       // Static strings when possible
}

// Builder pattern for efficient construction
pub struct HistoricalPriceBuilder {
    symbol: Option<Cow<'static, str>>,
    timestamp: Option<DateTime<Utc>>,
    // ... other fields
}
```

**Benefits**:
- 50-80% reduction in memory allocations
- Zero-copy string operations where possible
- Type-safe data handling for both API and web interface
- Consistent data structures across API and web rendering

### 6. Database Layer (`database.rs`)

**Technology**: SQLx with connection pooling

**Responsibilities**:
- Database connection management
- Query execution and optimization
- Transaction handling
- Data persistence

**Web Interface Optimizations**:
```rust
// Optimized queries for web interface
impl Database {
    // Dashboard popular symbols
    pub async fn get_popular_symbols(&self, limit: usize) -> Result<Vec<Symbol>, Error> {
        sqlx::query_as!(
            Symbol,
            "SELECT symbol, company_name, market_cap, sector 
             FROM symbols 
             ORDER BY market_cap DESC 
             LIMIT ?",
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
    }

    // Search with ranking for web interface
    pub async fn search_symbols_web(&self, query: &str, limit: usize) -> Result<Vec<Symbol>, Error> {
        sqlx::query_as!(
            Symbol,
            "SELECT * FROM symbols 
             WHERE symbol LIKE ? OR company_name LIKE ?
             ORDER BY 
                CASE WHEN symbol = ? THEN 1 
                     WHEN symbol LIKE ? THEN 2 
                     ELSE 3 END,
                market_cap DESC
             LIMIT ?",
            format!("%{}%", query),
            format!("%{}%", query),
            query.to_uppercase(),
            format!("{}%", query.to_uppercase()),
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

### 7. Yahoo Finance Service (`yahoo_service.rs`)

**Technology**: HTTP client with concurrent caching

**Responsibilities**:
- External API communication
- Multi-layer caching strategy
- Rate limiting for external calls
- Data transformation and validation

**Caching Architecture for Web Interface**:
```rust
// Multi-layer caching with different TTLs for web interface
static QUOTE_CACHE: Lazy<DashMap<String, CachedData<Quote>>> = 
    Lazy::new(DashMap::new);

static CHART_DATA_CACHE: Lazy<DashMap<String, CachedData<Vec<PricePoint>>>> = 
    Lazy::new(DashMap::new);

// Market-hours aware caching for web interface
pub fn get_cache_ttl(data_type: DataType, market_hours: bool) -> Duration {
    match (data_type, market_hours) {
        (DataType::Quote, true) => Duration::from_secs(300),      // 5 min during market
        (DataType::Quote, false) => Duration::from_secs(3600),   // 1 hour after market
        (DataType::ChartData, _) => Duration::from_secs(900),    // 15 min for charts
        (DataType::Historical, _) => Duration::from_secs(3600),  // 1 hour
        (DataType::Profile, _) => Duration::from_secs(86400),    // 24 hours
    }
}
```

## Web Interface Architecture

### Frontend Technology Stack

#### Core Technologies
- **Templating**: Askama (compile-time Rust templates)
- **Styling**: Tailwind CSS (utility-first responsive design)
- **JavaScript**: Vanilla JS with Chart.js for visualizations
- **Icons**: FontAwesome for professional appearance
- **Charts**: Chart.js for interactive financial charts

#### Responsive Design System
```css
/* Mobile-first responsive breakpoints */
.container {
  @apply w-full px-4;
}

@screen sm {
  .container { @apply max-w-screen-sm; }
}

@screen md {
  .container { @apply max-w-screen-md; }
}

@screen lg {
  .container { @apply max-w-screen-lg; }
}

@screen xl {
  .container { @apply max-w-screen-xl; }
}
```

### Template Engine Integration

#### Compile-time Template Processing
```rust
// Template compilation happens at build time
#[derive(Template)]
#[template(path = "analytics.html")]
pub struct AnalyticsTemplate {
    pub symbol: Option<String>,
    pub chart_data: Option<Vec<PricePoint>>,
    pub indicators: TechnicalIndicators,
}

// Template rendering with error handling
impl AnalyticsTemplate {
    pub async fn render_with_data(symbol: &str) -> Result<String, TemplateError> {
        let chart_data = fetch_chart_data(symbol).await?;
        let indicators = calculate_indicators(&chart_data).await?;
        
        let template = AnalyticsTemplate {
            symbol: Some(symbol.to_string()),
            chart_data: Some(chart_data),
            indicators,
        };
        
        template.render().map_err(TemplateError::from)
    }
}
```

#### Template Data Flow
```
API Data → Rust Structures → Template Context → HTML Rendering → Client Browser
    ↓              ↓               ↓              ↓              ↓
Validation    Cow Usage      Data Binding    Server-side    Client Display
             Zero-copy      Type Safety      Rendering      Interactivity
```

### Frontend Performance Architecture

#### Chart Rendering Optimization
```javascript
// Efficient chart creation with data management
class FinancialChartManager {
    constructor(containerId, maxDataPoints = 1000) {
        this.container = document.getElementById(containerId);
        this.maxDataPoints = maxDataPoints;
        this.chart = null;
    }
    
    createCandlestickChart(data) {
        // Optimize data for performance
        const optimizedData = this.optimizeData(data);
        
        const config = {
            type: 'candlestick',
            data: {
                datasets: [{
                    label: 'Price',
                    data: optimizedData,
                    pointRadius: 0, // Disable points for performance
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: {
                    duration: 0 // Disable animations for large datasets
                },
                scales: {
                    x: { 
                        type: 'time',
                        ticks: { maxTicksLimit: 20 }
                    },
                    y: { beginAtZero: false }
                },
                plugins: {
                    legend: { display: true },
                    tooltip: { 
                        mode: 'index',
                        intersect: false
                    }
                }
            }
        };
        
        this.chart = new Chart(this.container, config);
    }
    
    optimizeData(data) {
        // Limit data points for performance
        if (data.length > this.maxDataPoints) {
            const step = Math.ceil(data.length / this.maxDataPoints);
            return data.filter((_, index) => index % step === 0);
        }
        return data;
    }
    
    updateChart(newData) {
        if (this.chart) {
            this.chart.data.datasets[0].data = this.optimizeData(newData);
            this.chart.update('none'); // Skip animation
        }
    }
}
```

#### API Integration Layer
```javascript
// Centralized API communication with error handling
class MangoDataAPI {
    constructor(baseURL = '') {
        this.baseURL = baseURL;
        this.cache = new Map();
    }
    
    async fetchWithCache(endpoint, cacheTTL = 300000) {
        const cacheKey = endpoint;
        const cached = this.cache.get(cacheKey);
        
        if (cached && Date.now() - cached.timestamp < cacheTTL) {
            return cached.data;
        }
        
        try {
            const response = await fetch(`${this.baseURL}${endpoint}`);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            
            const data = await response.json();
            this.cache.set(cacheKey, {
                data,
                timestamp: Date.now()
            });
            
            return data;
        } catch (error) {
            console.error('API request failed:', error);
            this.handleApiError(error);
            throw error;
        }
    }
    
    async getSymbolData(symbol) {
        return this.fetchWithCache(`/api/symbols/${symbol}/comprehensive`);
    }
    
    async searchSymbols(query) {
        return this.fetchWithCache(`/api/symbols/search?q=${encodeURIComponent(query)}`, 60000);
    }
    
    handleApiError(error) {
        const errorDiv = document.getElementById('error-container');
        if (errorDiv) {
            errorDiv.innerHTML = `
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                    <strong>Error:</strong> ${error.message}
                </div>
            `;
        }
    }
}
```

## Performance Optimizations

### Web Interface Specific Optimizations

#### Server-Side Rendering Performance
```rust
// Template rendering with caching
static TEMPLATE_CACHE: Lazy<DashMap<String, (String, Instant)>> = 
    Lazy::new(DashMap::new);

async fn render_cached_template<T: Template>(
    template: T, 
    cache_key: &str,
    cache_ttl: Duration
) -> Result<String, Error> {
    // Check cache in production
    if !cfg!(debug_assertions) {
        if let Some((cached_html, timestamp)) = TEMPLATE_CACHE.get(cache_key) {
            if timestamp.elapsed() < cache_ttl {
                return Ok(cached_html.clone());
            }
        }
    }
    
    // Render template
    let rendered = template.render()?;
    
    // Cache result in production
    if !cfg!(debug_assertions) {
        TEMPLATE_CACHE.insert(
            cache_key.to_string(), 
            (rendered.clone(), Instant::now())
        );
    }
    
    Ok(rendered)
}
```

#### Memory Efficiency for Templates
```rust
// Optimized data structures for template rendering
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    // Use Cow for efficient string handling
    pub title: Cow<'static, str>,
    pub status_message: Cow<'static, str>,
    
    // Borrow data when possible
    pub symbols: Cow<'_, [Symbol]>,
    pub recent_data: Cow<'_, [PricePoint]>,
    
    // Own complex computed data
    pub metrics: SystemMetrics,
    pub charts_config: ChartConfiguration,
}

impl DashboardTemplate {
    pub fn new(symbols: &[Symbol], metrics: SystemMetrics) -> Self {
        Self {
            title: "Financial Dashboard".into(),
            status_message: "System Operational".into(),
            symbols: Cow::Borrowed(symbols),
            recent_data: Cow::Borrowed(&[]), // Will be populated from cache
            metrics,
            charts_config: ChartConfiguration::default(),
        }
    }
}
```

### Frontend Performance Patterns

#### Efficient DOM Manipulation
```javascript
// Batch DOM updates for better performance
class UIUpdater {
    constructor() {
        this.updateQueue = [];
        this.updateScheduled = false;
    }
    
    scheduleUpdate(updateFn) {
        this.updateQueue.push(updateFn);
        if (!this.updateScheduled) {
            this.updateScheduled = true;
            requestAnimationFrame(() => {
                this.flushUpdates();
            });
        }
    }
    
    flushUpdates() {
        const fragment = document.createDocumentFragment();
        
        this.updateQueue.forEach(updateFn => {
            updateFn(fragment);
        });
        
        document.body.appendChild(fragment);
        this.updateQueue = [];
        this.updateScheduled = false;
    }
    
    updateSymbolList(symbols) {
        this.scheduleUpdate((fragment) => {
            const container = document.getElementById('symbol-list');
            container.innerHTML = '';
            
            symbols.forEach(symbol => {
                const element = this.createSymbolElement(symbol);
                container.appendChild(element);
            });
        });
    }
}
```

#### Debounced Search Implementation
```javascript
// Efficient search with debouncing and caching
class SearchManager {
    constructor(apiClient, debounceMs = 300) {
        this.api = apiClient;
        this.debounceMs = debounceMs;
        this.searchTimeout = null;
        this.lastQuery = '';
        this.resultsCache = new Map();
    }
    
    search(query) {
        clearTimeout(this.searchTimeout);
        
        if (query === this.lastQuery) return;
        
        // Check cache first
        if (this.resultsCache.has(query)) {
            this.displayResults(this.resultsCache.get(query));
            return;
        }
        
        this.searchTimeout = setTimeout(async () => {
            try {
                this.showLoading();
                const results = await this.api.searchSymbols(query);
                
                this.resultsCache.set(query, results);
                this.displayResults(results);
                this.lastQuery = query;
            } catch (error) {
                this.showError(error.message);
            } finally {
                this.hideLoading();
            }
        }, this.debounceMs);
    }
    
    displayResults(results) {
        const container = document.getElementById('search-results');
        container.innerHTML = results.data.symbols.map(symbol => `
            <div class="symbol-result" data-symbol="${symbol.symbol}">
                <span class="symbol">${symbol.symbol}</span>
                <span class="name">${symbol.company_name}</span>
            </div>
        `).join('');
    }
}
```

## Caching Strategy

### Multi-Layer Caching Architecture

```
┌─────────────────────────────────────────────────────┐
│                Browser Cache                        │
│  ├─ Static Assets (CSS/JS)                         │
│  ├─ Template HTML (with Cache-Control headers)     │
│  └─ API Response Cache (JavaScript localStorage)   │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│               Server Cache                          │
│  ├─ Template Rendering Cache (DashMap)             │
│  ├─ API Response Cache (DashMap)                   │
│  └─ Database Query Cache (SQLx + DashMap)          │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│              Database Cache                         │
│  ├─ Query Result Cache (PostgreSQL/SQLite)         │
│  └─ Connection Pool Cache                          │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│            External API Cache                       │
│  └─ Yahoo Finance API Response Cache               │
└─────────────────────────────────────────────────────┘
```

### Cache TTL Strategy for Web Interface

```rust
pub enum CacheType {
    TemplateHtml,
    ApiResponse,
    ChartData,
    SearchResults,
    StaticData,
}

pub fn get_cache_ttl(cache_type: CacheType, market_state: MarketState) -> Duration {
    match (cache_type, market_state) {
        // Template caching
        (CacheType::TemplateHtml, _) => Duration::from_secs(300),
        
        // API response caching (varies by market state)
        (CacheType::ApiResponse, MarketState::Open) => Duration::from_secs(300),
        (CacheType::ApiResponse, MarketState::Closed) => Duration::from_secs(3600),
        
        // Chart data (optimized for web interface)
        (CacheType::ChartData, MarketState::Open) => Duration::from_secs(300),
        (CacheType::ChartData, MarketState::Closed) => Duration::from_secs(1800),
        
        // Search results
        (CacheType::SearchResults, _) => Duration::from_secs(3600),
        
        // Static company data
        (CacheType::StaticData, _) => Duration::from_secs(86400),
    }
}
```

## Data Flow Architecture

### Request Processing Pipeline

#### API Request Flow
```
1. Client Request (API)
   ↓
2. Rate Limiter Check
   ↓
3. Input Validation
   ↓
4. Memory Cache Check
   ↓ (Cache Miss)
5. Database Cache Check
   ↓ (Cache Miss)
6. Yahoo Finance API Call
   ↓
7. Data Processing & Validation
   ↓
8. Cache Storage (Memory + Database)
   ↓
9. JSON Response Formatting
   ↓
10. Client Response
```

#### Web Interface Request Flow
```
1. Browser Request (Web UI)
   ↓
2. Route Matching (/ui/*)
   ↓
3. Feature Flag Check (web-ui enabled?)
   ↓
4. Template Handler Execution
   ↓
5. Data Fetching (via internal API calls)
   ↓
6. Template Data Preparation
   ↓
7. Template Rendering (Askama)
   ↓
8. HTML Response Generation
   ↓
9. Browser Rendering
   ↓
10. JavaScript Initialization
    ↓
11. Additional API Calls (for dynamic content)
```

### Feature Flag Architecture

#### Conditional Compilation Strategy
```rust
// Cargo.toml features configuration
[features]
default = []
web-ui = ["askama", "askama_axum"]

// Conditional module loading
#[cfg(feature = "web-ui")]
pub mod web_ui {
    use askama::Template;
    use axum::response::IntoResponse;
    
    // Web interface implementation
    pub async fn dashboard() -> impl IntoResponse {
        DashboardTemplate::new().render()
    }
}

#[cfg(not(feature = "web-ui"))]
pub mod web_ui {
    use axum::http::StatusCode;
    
    // Fallback implementations
    pub async fn dashboard() -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
```

#### Build-time Optimization
```rust
// Route registration with feature gates
let mut app = Router::new()
    // Core API routes (always available)
    .route("/health", get(health_check))
    .route("/api/symbols/search", get(search_symbols));

// Conditional web interface routes
#[cfg(feature = "web-ui")]
{
    app = app
        .route("/", get(web_ui::dashboard))
        .route("/ui", get(web_ui::dashboard))
        .route("/ui/search", get(web_ui::search))
        .route("/ui/analytics", get(web_ui::analytics));
}

app
```

## Database Schema

### Core Tables (Enhanced for Web Interface)

```sql
-- Symbols table with optimized indexing for web interface
CREATE TABLE symbols (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol VARCHAR(10) NOT NULL UNIQUE,
    company_name TEXT NOT NULL,
    exchange VARCHAR(10),
    currency VARCHAR(3),
    sector TEXT,
    industry TEXT,
    market_cap BIGINT,
    website TEXT,
    description TEXT,
    logo_url TEXT,               -- For web interface display
    is_popular BOOLEAN DEFAULT FALSE,  -- For dashboard quick access
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Historical prices with composite index
CREATE TABLE historical_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol VARCHAR(10) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    interval_type VARCHAR(5) NOT NULL,  -- 1m, 5m, 1h, 1d, etc.
    open DECIMAL(10,4),
    high DECIMAL(10,4),
    low DECIMAL(10,4),
    close DECIMAL(10,4),
    volume BIGINT,
    adj_close DECIMAL(10,4),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (symbol) REFERENCES symbols(symbol),
    UNIQUE(symbol, timestamp, interval_type)
);

-- User preferences for web interface (if user system added)
CREATE TABLE user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id VARCHAR(50) NOT NULL,  -- Could be session ID
    favorite_symbols TEXT,         -- JSON array of symbols
    chart_preferences TEXT,        -- JSON object of chart settings
    theme VARCHAR(20) DEFAULT 'light',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Web interface specific optimizations
CREATE INDEX idx_symbols_popular ON symbols(is_popular, market_cap DESC) WHERE is_popular = TRUE;
CREATE INDEX idx_symbols_search ON symbols(company_name, symbol);
CREATE INDEX idx_historical_web ON historical_prices(symbol, interval_type, timestamp DESC);
CREATE INDEX idx_historical_chart ON historical_prices(symbol, timestamp DESC) WHERE interval_type IN ('1d', '1h');
```

### Query Optimization for Web Interface

```rust
// Optimized queries for web interface performance
impl Database {
    // Dashboard quick symbols with optimized query
    pub async fn get_dashboard_symbols(&self) -> Result<Vec<Symbol>, Error> {
        sqlx::query_as!(
            Symbol,
            r#"
            SELECT symbol, company_name, market_cap, sector, logo_url
            FROM symbols 
            WHERE is_popular = TRUE 
            ORDER BY market_cap DESC 
            LIMIT 12
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    // Chart data with interval-specific optimization
    pub async fn get_chart_data(
        &self, 
        symbol: &str, 
        interval: &str, 
        limit: usize
    ) -> Result<Vec<PricePoint>, Error> {
        sqlx::query_as!(
            PricePoint,
            r#"
            SELECT timestamp, open, high, low, close, volume
            FROM historical_prices 
            WHERE symbol = ? AND interval_type = ?
            ORDER BY timestamp DESC 
            LIMIT ?
            "#,
            symbol,
            interval,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
    }

    // Search with ranking and fuzzy matching
    pub async fn search_symbols_ranked(
        &self, 
        query: &str, 
        limit: usize
    ) -> Result<Vec<Symbol>, Error> {
        sqlx::query_as!(
            Symbol,
            r#"
            SELECT *, 
                   CASE 
                       WHEN symbol = ? THEN 1
                       WHEN symbol LIKE ? THEN 2
                       WHEN company_name LIKE ? THEN 3
                       ELSE 4
                   END as rank_order
            FROM symbols 
            WHERE symbol LIKE ? OR company_name LIKE ?
            ORDER BY rank_order, market_cap DESC
            LIMIT ?
            "#,
            query.to_uppercase(),
            format!("{}%", query.to_uppercase()),
            format!("%{}%", query),
            format!("%{}%", query.to_uppercase()),
            format!("%{}%", query),
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

## Error Handling Strategy

### Enhanced Error Handling for Web Interface

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // API errors
    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },
    
    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },
    
    #[error("Yahoo Finance API error: {message}")]
    YahooApiError { message: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    // Web interface specific errors
    #[error("Template rendering failed: {template}")]
    TemplateError { template: String },
    
    #[error("Feature not available: {feature}")]
    FeatureDisabled { feature: String },
    
    #[error("Invalid web request: {message}")]
    WebRequestError { message: String },
}

// Error response handling for web interface
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            // For web interface requests, return HTML error pages
            AppError::TemplateError { template } => {
                let error_html = format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Template Error</title></head>
                    <body>
                        <h1>Template Rendering Error</h1>
                        <p>Failed to render template: {}</p>
                        <a href="/ui">Return to Dashboard</a>
                    </body>
                    </html>
                    "#,
                    template
                );
                
                (StatusCode::INTERNAL_SERVER_ERROR, Html(error_html)).into_response()
            }
            
            AppError::FeatureDisabled { feature } => {
                let error_html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Feature Not Available</title></head>
                    <body>
                        <h1>Feature Not Available</h1>
                        <p>The web interface is not enabled in this build.</p>
                        <p>Please use the API endpoints instead.</p>
                    </body>
                    </html>
                "#;
                
                (StatusCode::NOT_FOUND, Html(error_html)).into_response()
            }
            
            // For API requests, return JSON
            _ => {
                let (status, error_message, error_code) = match self {
                    AppError::SymbolNotFound { .. } => (
                        StatusCode::NOT_FOUND,
                        self.to_string(),
                        "SYMBOL_NOT_FOUND"
                    ),
                    AppError::RateLimitExceeded { .. } => (
                        StatusCode::TOO_MANY_REQUESTS,
                        self.to_string(),
                        "RATE_LIMIT_EXCEEDED"
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                        "INTERNAL_ERROR"
                    ),
                };
                
                let body = json!({
                    "success": false,
                    "error": error_message,
                    "code": error_code,
                    "timestamp": Utc::now()
                });
                
                (status, Json(body)).into_response()
            }
        }
    }
}
```

## Security Considerations

### Web Interface Security

#### Input Validation and Sanitization
```rust
// Template data sanitization
use askama::filters;

#[derive(Template)]
#[template(path = "analytics.html")]
pub struct AnalyticsTemplate {
    pub symbol: String,  // Will be automatically escaped in templates
    pub user_query: String,
}

// Custom filters for additional security
mod template_filters {
    pub fn sanitize_symbol(s: &str) -> askama::Result<String> {
        let sanitized = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '.')
            .take(10)  // Limit length
            .collect();
        Ok(sanitized)
    }
    
    pub fn sanitize_query(s: &str) -> askama::Result<String> {
        let sanitized = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
            .take(100)  // Limit length
            .collect();
        Ok(sanitized)
    }
}
```

#### Content Security Policy
```rust
// Add CSP headers for web interface
async fn add_security_headers<B>(request: Request<B>, next: Next<B>) -> Response {
    let mut response = next.run(request).await;
    
    // Only add CSP for HTML responses (web interface)
    if let Some(content_type) = response.headers().get("content-type") {
        if content_type.to_str().unwrap_or("").contains("text/html") {
            response.headers_mut().insert(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; font-src 'self' https://cdnjs.cloudflare.com".parse().unwrap()
            );
        }
    }
    
    response
}
```

#### Rate Limiting for Web Interface
```rust
// Separate rate limits for web interface
pub struct RateLimitConfig {
    pub api_requests_per_minute: u32,
    pub web_requests_per_minute: u32,
    pub search_requests_per_minute: u32,
}

async fn web_rate_limiter(
    client_ip: String,
    request: Request<Body>,
    next: Next<Body>
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    let limit = if path.starts_with("/ui") {
        &WEB_RATE_LIMITER
    } else if path.contains("/search") {
        &SEARCH_RATE_LIMITER
    } else {
        &API_RATE_LIMITER
    };
    
    if !limit.check_key(&client_ip).is_ok() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}
```

## Monitoring and Observability

### Enhanced Metrics for Web Interface

```rust
// Comprehensive metrics collection
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub api_metrics: ApiMetrics,
    pub web_metrics: WebMetrics,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebMetrics {
    pub page_views_last_hour: u64,
    pub unique_visitors: u64,
    pub most_viewed_symbols: Vec<String>,
    pub avg_page_load_time_ms: f64,
    pub template_render_time_ms: f64,
    pub chart_generations: u64,
    pub search_queries: u64,
    pub error_rate: f64,
}

// Metrics collection for web interface
impl MetricsCollector {
    pub async fn record_page_view(&self, page: &str, user_id: Option<&str>) {
        let timestamp = Utc::now();
        
        // Record in metrics database or memory
        self.page_views.entry(page.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        
        // Track unique visitors (simplified)
        if let Some(user) = user_id {
            self.unique_visitors.insert(user.to_string());
        }
    }
    
    pub async fn record_template_render(&self, template: &str, duration: Duration) {
        self.template_render_times
            .entry(template.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub async fn record_chart_generation(&self, symbol: &str, chart_type: &str) {
        self.chart_generations += 1;
        self.popular_charts
            .entry(format!("{}:{}", symbol, chart_type))
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}
```

### Health Check Enhancement

```rust
// Enhanced health check with web interface status
async fn enhanced_health_check() -> impl IntoResponse {
    let health_status = HealthStatus {
        status: "healthy".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime: get_uptime(),
        features: FeatureFlags {
            cow_optimizations: true,
            rate_limiting: true,
            concurrent_caching: true,
            background_cleanup: true,
            // Feature flags are compile-time only, no runtime web_interface status
        },
        // Note: No runtime web interface status tracking implemented
            enabled: cfg!(feature = "web-ui"),
            template_engine: if cfg!(feature = "web-ui") { 
                Some("askama".to_string()) 
            } else { 
                None 
            },
            active_sessions: get_active_sessions().await,
            templates_compiled: get_template_count(),
            static_assets_served: get_static_asset_count().await,
        },
        database: check_database_health().await,
        performance: PerformanceMetrics {
            avg_response_time_ms: get_avg_response_time().await,
            requests_per_second: get_requests_per_second().await,
            cache_hit_rate: get_cache_hit_rate().await,
        },
    };
    
    Json(ApiResponse::success(health_status))
}
```

## Deployment Architecture

### Container Strategy

#### Multi-stage Docker Build
```dockerfile
# Build stage
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY templates/ templates/

# Build with web-ui feature
RUN cargo build --release --features web-ui

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mango-data-service /usr/local/bin/
COPY --from=builder /app/templates /app/templates

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

CMD ["mango-data-service"]
```

#### Production Docker Compose
```yaml
version: '3.8'

services:
  mango-data-service:
    build:
      context: .
      args:
        - FEATURES=web-ui
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://mango:${DB_PASSWORD}@postgres:5432/mango_finance
      - RUST_LOG=info
      - WEB_UI_THEME=professional
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    networks:
      - mango-network

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=mango_finance
      - POSTGRES_USER=mango
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - mango-network

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - mango-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - mango-data-service
    networks:
      - mango-network

volumes:
  postgres_data:
  redis_data:

networks:
  mango-network:
    driver: bridge
```

### Load Balancing and Scaling

#### Nginx Configuration for Web Interface
```nginx
upstream mango_backend {
    server mango-data-service-1:3000;
    server mango-data-service-2:3000;
    server mango-data-service-3:3000;
}

server {
    listen 80;
    server_name your-domain.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";
    
    # API endpoints
    location /api/ {
        proxy_pass http://mango_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # API-specific caching
        add_header Cache-Control "no-cache, must-revalidate";
    }
    
    # Web interface
    location /ui/ {
        proxy_pass http://mango_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Web interface caching
        add_header Cache-Control "public, max-age=300";
    }
    
    # Health check
    location /health {
        proxy_pass http://mango_backend;
        access_log off;
    }
    
    # Root redirects to web interface
    location = / {
        return 301 /ui;
    }
    
    # Static assets (if served separately)
    location /static/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

## Future Enhancements

### Planned Web Interface Features

1. **Real-time Data Streaming**
   ```rust
   // WebSocket integration for real-time updates
   async fn websocket_handler(
       ws: WebSocketUpgrade,
       user_agent: Option<TypedHeader<headers::UserAgent>>,
   ) -> impl IntoResponse {
       ws.on_upgrade(handle_socket)
   }
   
   async fn handle_socket(socket: WebSocket) {
       // Stream real-time market data to web interface
   }
   ```

2. **User Authentication and Personalization**
   ```rust
   // User session management for web interface
   #[derive(Template)]
   #[template(path = "personalized_dashboard.html")]
   pub struct PersonalizedDashboard {
       pub user: User,
       pub watchlist: Vec<Symbol>,
       pub preferences: UserPreferences,
   }
   ```

3. **Advanced Analytics Dashboard**
   - Portfolio tracking and analysis
   - Risk assessment tools
   - Comparative analysis features
   - Custom indicator creation

4. **Mobile Progressive Web App (PWA)**
   - Service worker for offline functionality
   - Native app-like experience
   - Push notifications for price alerts

### Performance Targets

- **Web Interface Response Time**: < 200ms for template rendering
- **Chart Loading Time**: < 500ms for complex financial charts
- **Search Response Time**: < 100ms for symbol search
- **Concurrent Users**: Support 1000+ simultaneous web users
- **Memory Efficiency**: < 50MB additional memory usage for web interface

---

This comprehensive architecture provides a robust foundation for both high-performance API access and an intuitive web interface, with room for future growth and optimization. The modular design ensures that the web interface enhances rather than compromises the core API performance. 