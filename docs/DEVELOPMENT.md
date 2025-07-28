# 🛠️ Development Guide

Complete development guide for Mango Data Service contributors and maintainers, including comprehensive web interface development.

## Table of Contents

- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Web Interface Development](#web-interface-development)
- [Development Workflow](#development-workflow)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Strategy](#testing-strategy)
- [Performance Optimization Guidelines](#performance-optimization-guidelines)
- [Debugging and Troubleshooting](#debugging-and-troubleshooting)
- [Contributing Guidelines](#contributing-guidelines)

## Development Environment Setup

### Prerequisites

1. **Rust Toolchain**
   ```bash
   # Install Rust via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Verify installation
   rustc --version  # Should be 1.70+
   cargo --version
   ```

2. **Development Tools**
   ```bash
   # Essential development tools
   cargo install cargo-watch      # Auto-reload during development
   cargo install cargo-audit      # Security vulnerability scanning
   cargo install cargo-outdated   # Check for outdated dependencies
   cargo install cargo-tree       # Dependency tree visualization
   cargo install cargo-expand     # Macro expansion for debugging
   cargo install cargo-flamegraph # Performance profiling
   ```

3. **Database Tools** (Optional)
   ```bash
   # SQLite CLI for database inspection
   sudo apt-get install sqlite3  # Ubuntu/Debian
   brew install sqlite3          # macOS
   
   # PostgreSQL client (for production setup)
   sudo apt-get install postgresql-client
   ```

4. **Frontend Development Tools** (for web-ui feature)
   ```bash
   # Node.js for frontend tooling (optional)
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   
   # Prettier for template formatting (optional)
   npm install -g prettier
   
   # Live server for static asset testing (optional)
   npm install -g live-server
   ```

5. **IDE Setup**
   
   **VS Code Extensions:**
   - rust-analyzer (essential)
   - CodeLLDB (for debugging)
   - Better TOML
   - Error Lens
   - HTML CSS Support
   - Tailwind CSS IntelliSense (for web-ui)
   - Auto Rename Tag
   
   **Settings (`.vscode/settings.json`):**
   ```json
   {
     "rust-analyzer.cargo.features": ["web-ui"],
     "rust-analyzer.checkOnSave.command": "clippy",
     "rust-analyzer.inlayHints.enable": true,
     "files.watcherExclude": {
       "**/target/**": true
     },
     "html.format.indentInnerHtml": true,
     "css.validate": true,
     "tailwindCSS.includeLanguages": {
       "html": "html"
     },
     "emmet.includeLanguages": {
       "html": "html"
     }
   }
   ```

### Environment Configuration

1. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd mango-data-service
   
   # Copy environment template
   cp .env.example .env
   
   # Edit environment variables
   nano .env
   ```

2. **Environment Variables**
   ```env
   # Development configuration
   DATABASE_URL=sqlite:data/dev.db
   RUST_LOG=mango_data_service=debug,tower_http=debug
   PORT=3000
   HOST=127.0.0.1
   
   # Rate limiting (relaxed for development)
   API_RATE_LIMIT_PER_MINUTE=1000
   YAHOO_API_RATE_LIMIT_PER_MINUTE=100
   
   # Cache settings
   CACHE_TTL_QUOTES=60           # 1 minute for faster testing
   CACHE_TTL_HISTORICAL=300      # 5 minutes
   CACHE_TTL_PROFILES=3600       # 1 hour
   
   # Web Interface settings (when web-ui enabled)
   WEB_UI_THEME=development      # Development theme
   WEB_UI_DEBUG_MODE=true        # Enable debug features
   WEB_UI_TEMPLATE_CACHE=false   # Disable template caching for development
   ```

3. **Database Setup**
   ```bash
   # Create data directory
   mkdir -p data
   
   # Run database migrations (if any)
   cargo run --bin migrate
   
   # Or let the application create the database automatically
   cargo run --features web-ui
   ```

4. **Feature Flag Development Setup**
   ```bash
   # Test API only build
   cargo check
   
   # Test with web-ui feature
   cargo check --features web-ui
   
   # Verify feature compilation
   cargo build --features web-ui
   ```

## Project Structure

```
mango-data-service/
├── src/                          # Source code
│   ├── main.rs                   # Application entry point
│   ├── handlers.rs               # HTTP request handlers
│   ├── models.rs                 # Data structures and types
│   ├── database.rs               # Database operations
│   ├── yahoo_service.rs          # Yahoo Finance API client
│   └── web_ui.rs                # Web interface handlers (feature-gated)
├── templates/                    # Askama templates (web-ui feature)
│   ├── base.html                 # Base template with common layout
│   ├── dashboard.html            # Main dashboard
│   ├── search.html               # Symbol search interface
│   └── analytics.html            # Financial analytics suite
├── tests/                        # Integration tests
│   ├── integration_tests.rs      # API integration tests
│   ├── web_ui_tests.rs          # Web interface tests
│   └── common/                   # Test utilities
├── examples/                     # Example scripts and tests
│   ├── comprehensive_tests.py    # Python test suite
│   ├── web_ui_examples/         # Web interface examples
│   └── test_kweb_fxi.py         # Specific ticker tests
├── docs/                         # Documentation
│   ├── API_REFERENCE.md          # API documentation
│   ├── ARCHITECTURE.md           # System architecture
│   └── DEVELOPMENT.md            # This file
├── data/                         # Database files (gitignored)
├── target/                       # Rust build artifacts (gitignored)
├── Cargo.toml                    # Project dependencies
├── Cargo.lock                    # Dependency lock file
├── .env.example                  # Environment template
├── .gitignore                    # Git ignore rules
└── README.md                     # Project overview
```

### Key Files Explained

#### `src/main.rs`
- Application entry point
- Server configuration and startup
- Route registration (including conditional web UI routes)
- Background task spawning
- Graceful shutdown handling
- Feature flag integration

#### `src/handlers.rs`
- HTTP request handlers for all API endpoints
- Rate limiting implementation
- Input validation and sanitization
- Response formatting
- Error handling and conversion

#### `src/models.rs`
- Data structures with Cow optimizations
- Serialization/deserialization logic
- Builder patterns for complex types
- Type safety and validation

#### `src/database.rs`
- Database connection management
- CRUD operations
- Query optimization
- Connection pooling
- Migration support

#### `src/yahoo_service.rs`
- Yahoo Finance API client
- Caching layer implementation
- Rate limiting for external calls
- Data transformation and validation

#### `src/web_ui.rs` (Web-UI Feature)
- Web interface request handlers
- Template data preparation
- Feature-gated implementations
- Responsive error handling for disabled features

#### `templates/` Directory (Web-UI Feature)
- **`base.html`**: Common layout, navigation, styling, and JavaScript includes
- **`dashboard.html`**: System overview, quick actions, and metrics display
- **`search.html`**: Symbol search interface with real-time capabilities
- **`analytics.html`**: Advanced charting and technical analysis interface

## Web Interface Development

### Template System (Askama)

The web interface uses the Askama templating engine for server-side rendering with compile-time template checking.

#### Template Structure
```rust
// Template definition in src/web_ui.rs
#[cfg(feature = "web-ui")]
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: Cow<'static, str>,
    pub symbols: Vec<Symbol>,
    pub metrics: SystemMetrics,
}

// Handler implementation
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
```

#### Template Development Best Practices

1. **Use Cow for String Data**
   ```rust
   pub struct TemplateData {
       pub title: Cow<'static, str>,        // Efficient for static strings
       pub dynamic_content: Cow<'_, str>,   // Borrowed when possible
   }
   ```

2. **Template Inheritance**
   ```html
   <!-- base.html -->
   <!DOCTYPE html>
   <html>
   <head>
       <title>{% block title %}Default Title{% endblock %}</title>
   </head>
   <body>
       {% block content %}{% endblock %}
   </body>
   </html>
   
   <!-- dashboard.html -->
   {% extends "base.html" %}
   
   {% block title %}Dashboard - Mango Data Service{% endblock %}
   
   {% block content %}
   <div class="dashboard">
       <h1>{{ title }}</h1>
       <!-- Dashboard content -->
   </div>
   {% endblock %}
   ```

3. **Conditional Rendering**
   ```html
   {% if cfg!(feature = "web-ui") %}
       <div class="web-features">
           <!-- Web interface specific content -->
       </div>
   {% else %}
       <div class="api-only">
           <p>Web interface not available. API access only.</p>
       </div>
   {% endif %}
   ```

4. **Loop Handling**
   ```html
   <div class="symbols-grid">
   {% for symbol in symbols %}
       <div class="symbol-card">
           <h3>{{ symbol.symbol }}</h3>
           <p>{{ symbol.company_name }}</p>
           <span class="price">${{ symbol.price }}</span>
       </div>
   {% endfor %}
   </div>
   ```

### Frontend Development

#### Technology Stack
- **Styling**: Tailwind CSS for responsive design
- **JavaScript**: Vanilla JS with Chart.js for visualizations
- **Icons**: FontAwesome for professional appearance
- **Charts**: Chart.js for interactive financial charts

#### CSS/Styling Guidelines

1. **Use Tailwind CSS Classes**
   ```html
   <div class="bg-white shadow-lg rounded-lg p-6 hover:shadow-xl transition-shadow">
       <h2 class="text-2xl font-bold text-gray-800 mb-4">Stock Analysis</h2>
       <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
           <!-- Content -->
       </div>
   </div>
   ```

2. **Responsive Design**
   ```html
   <!-- Mobile-first responsive classes -->
   <div class="w-full sm:w-1/2 lg:w-1/3 xl:w-1/4">
       <div class="p-4 sm:p-6 lg:p-8">
           <!-- Responsive padding -->
       </div>
   </div>
   ```

3. **Custom CSS for Specific Features**
   ```html
   <style>
   .gradient-bg { 
       background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
   }
   .card-hover { 
       transition: all 0.3s ease; 
   }
   .card-hover:hover { 
       transform: translateY(-5px); 
       box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1); 
   }
   </style>
   ```

#### JavaScript Development

1. **Chart.js Integration**
   ```javascript
   // Create candlestick chart for financial data
   function createCandlestickChart(data, containerId) {
       const ctx = document.getElementById(containerId).getContext('2d');
       
       const chartData = {
           labels: data.map(d => new Date(d.timestamp).toLocaleDateString()),
           datasets: [{
               label: 'Price',
               data: data.map(d => ({
                   x: d.timestamp,
                   o: d.open,
                   h: d.high,
                   l: d.low,
                   c: d.close
               })),
               borderColor: 'rgb(75, 192, 192)',
               backgroundColor: 'rgba(75, 192, 192, 0.2)',
           }]
       };
       
       new Chart(ctx, {
           type: 'candlestick',
           data: chartData,
           options: {
               responsive: true,
               plugins: {
                   legend: { display: true },
                   tooltip: { mode: 'index' }
               },
               scales: {
                   x: { type: 'time' },
                   y: { beginAtZero: false }
               }
           }
       });
   }
   ```

2. **API Integration**
   ```javascript
   // Fetch data from Mango Data Service API
   async function fetchSymbolData(symbol) {
       try {
           showLoading();
           const response = await fetch(`/api/symbols/${symbol}/comprehensive`);
           const data = await response.json();
           
           if (!data.success) {
               throw new Error(data.error);
           }
           
           updateUI(data.data);
       } catch (error) {
           showError(error.message);
       } finally {
           hideLoading();
       }
   }
   
   // Real-time search functionality
   function setupSymbolSearch() {
       const searchInput = document.getElementById('symbol-search');
       const resultsContainer = document.getElementById('search-results');
       
       let searchTimeout;
       searchInput.addEventListener('input', (e) => {
           clearTimeout(searchTimeout);
           searchTimeout = setTimeout(() => {
               performSearch(e.target.value);
           }, 300); // Debounce search
       });
   }
   ```

3. **Error Handling and User Feedback**
   ```javascript
   function showError(message) {
       const errorDiv = document.createElement('div');
       errorDiv.className = 'bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded';
       errorDiv.textContent = message;
       document.getElementById('error-container').appendChild(errorDiv);
       
       // Auto-remove after 5 seconds
       setTimeout(() => errorDiv.remove(), 5000);
   }
   
   function showLoading() {
       document.getElementById('loading-spinner').classList.remove('hidden');
   }
   
   function hideLoading() {
       document.getElementById('loading-spinner').classList.add('hidden');
   }
   ```

### Feature Flag Implementation

#### Conditional Compilation
```rust
// Feature-gated module
#[cfg(feature = "web-ui")]
pub mod web_ui {
    use askama::Template;
    use axum::response::IntoResponse;
    
    #[derive(Template)]
    #[template(path = "dashboard.html")]
    pub struct DashboardTemplate {
        // Template data
    }
    
    pub async fn dashboard() -> impl IntoResponse {
        DashboardTemplate {
            // Data initialization
        }
    }
}

// Fallback implementations
#[cfg(not(feature = "web-ui"))]
pub mod web_ui {
    use axum::http::StatusCode;
    
    pub async fn dashboard() -> Result<&'static str, StatusCode> {
        Err(StatusCode::NOT_FOUND)
    }
}
```

#### Route Registration
```rust
// In main.rs
let app = Router::new()
    // API routes (always available)
    .route("/api/symbols/search", get(search_symbols))
    .route("/health", get(health_check));

// Conditional web UI routes
#[cfg(feature = "web-ui")]
let app = app
    .route("/", get(web_ui::dashboard))
    .route("/ui", get(web_ui::dashboard))
    .route("/ui/search", get(web_ui::search))
    .route("/ui/analytics", get(web_ui::analytics));
```

#### Development vs Production Templates
```rust
// Template development helper
#[cfg(debug_assertions)]
fn get_template_cache_setting() -> bool {
    std::env::var("WEB_UI_TEMPLATE_CACHE")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false)
}

#[cfg(not(debug_assertions))]
fn get_template_cache_setting() -> bool {
    true // Always cache in production
}
```

## Development Workflow

### Daily Development

1. **Start Development Server**
   ```bash
   # API only development
   cargo watch -x run
   
   # Web interface development
   cargo watch -x "run --features web-ui"
   
   # Watch templates and auto-restart
   cargo watch -w src -w templates -x "run --features web-ui"
   
   # With specific logging
   RUST_LOG=debug cargo watch -x "run --features web-ui"
   ```

2. **Template Development Workflow**
   ```bash
   # Terminal 1: Run server with template watching
   cargo watch -w templates -x "run --features web-ui"
   
   # Terminal 2: Format templates (optional)
   find templates -name "*.html" | xargs prettier --write
   
   # Terminal 3: Live reload browser (if using separate frontend tools)
   live-server --port=3001 --proxy=http://localhost:3000
   ```

3. **Run Tests**
   ```bash
   # Unit tests
   cargo test
   
   # Test with web-ui feature
   cargo test --features web-ui
   
   # Integration tests
   cargo test --test integration_tests
   
   # Web interface tests
   cargo test --test web_ui_tests --features web-ui
   
   # With output
   cargo test -- --nocapture
   
   # Specific test
   cargo test test_dashboard_template --features web-ui
   ```

4. **Code Quality Checks**
   ```bash
   # Format code
   cargo fmt
   
   # Lint code
   cargo clippy --features web-ui
   
   # Security audit
   cargo audit
   
   # Check for outdated dependencies
   cargo outdated
   
   # Template syntax check (implicit with cargo check)
   cargo check --features web-ui
   ```

### Feature Development Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/new-web-feature
   ```

2. **Implement Feature**
   
   **For API Features:**
   - Add endpoint to `handlers.rs`
   - Update models in `models.rs`
   - Add tests
   
   **For Web Interface Features:**
   - Add template to `templates/`
   - Add handler to `web_ui.rs`
   - Add route in `main.rs` with feature gate
   - Update base template if needed
   - Add JavaScript functionality
   - Add tests for both API and web interface

3. **Test Thoroughly**
   ```bash
   # Test both configurations
   cargo test
   cargo test --features web-ui
   
   # Test template compilation
   cargo check --features web-ui
   
   # Test with Python scripts
   python examples/comprehensive_tests.py
   
   # Manual web interface testing
   curl http://localhost:3000/ui/new-feature
   ```

4. **Code Review Checklist**
   - [ ] Tests added and passing (API and web interface)
   - [ ] Template syntax valid and responsive
   - [ ] Feature flags properly implemented
   - [ ] Documentation updated (including web interface features)
   - [ ] Performance impact considered
   - [ ] Error handling implemented for both API and web
   - [ ] Cross-browser compatibility verified (for web features)
   - [ ] Mobile responsiveness tested

### Release Process

1. **Version Bump**
   ```bash
   # Update version in Cargo.toml
   # Update CHANGELOG.md
   # Update web interface version info
   # Tag release
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Build Release**
   ```bash
   # API only
   cargo build --release
   
   # With web interface
   cargo build --release --features web-ui
   
   # Test release build
   ./target/release/mango-data-service
   ```

3. **Performance Testing**
   ```bash
   # Load testing API
   wrk -t12 -c400 -d30s http://localhost:3000/health
   
   # Test web interface performance
   lighthouse http://localhost:3000/ui --output html
   
   # Memory profiling
   cargo flamegraph --bin mango-data-service --features web-ui
   ```

## Code Style and Standards

### Rust Style Guidelines

1. **Naming Conventions**
   ```rust
   // Use snake_case for functions and variables
   fn get_historical_data() -> Result<Vec<HistoricalPrice>, Error> {}
   let symbol_name = "AAPL";
   
   // Use PascalCase for types
   struct HistoricalPrice {}
   enum ApiError {}
   
   // Use SCREAMING_SNAKE_CASE for constants
   const MAX_SYMBOLS_PER_REQUEST: usize = 20;
   ```

2. **Error Handling**
   ```rust
   // Use Result types for fallible operations
   async fn fetch_data(symbol: &str) -> Result<Data, ApiError> {
       // Prefer ? operator for error propagation
       let response = http_client.get(url).send().await?;
       let data = response.json().await?;
       Ok(data)
   }
   
   // Use custom error types
   #[derive(Debug, thiserror::Error)]
   pub enum ApiError {
       #[error("Symbol not found: {symbol}")]
       SymbolNotFound { symbol: String },
       
       #[error("Template rendering failed: {0}")]
       TemplateError(#[from] askama::Error),
   }
   ```

3. **Memory Optimization**
   ```rust
   // Use Cow for potentially borrowed strings
   pub struct ApiResponse<T> {
       pub data: T,
       pub message: Cow<'static, str>,
   }
   
   // Template data structures with Cow
   #[derive(Template)]
   #[template(path = "dashboard.html")]
   pub struct DashboardTemplate {
       pub title: Cow<'static, str>,
       pub symbols: Vec<Symbol>,
   }
   
   // Use builder patterns for complex construction
   let price = HistoricalPriceBuilder::new()
       .symbol("AAPL")
       .timestamp(Utc::now())
       .close(150.0)
       .build()?;
   ```

4. **Feature Flag Best Practices**
   ```rust
   // Always provide fallback implementations
   #[cfg(feature = "web-ui")]
   pub async fn web_handler() -> impl IntoResponse {
       // Web interface implementation
       DashboardTemplate { /* data */ }
   }
   
   #[cfg(not(feature = "web-ui"))]
   pub async fn web_handler() -> Result<&'static str, StatusCode> {
       Err(StatusCode::NOT_FOUND)
   }
   
   // Use feature gates for dependencies
   #[cfg(feature = "web-ui")]
   use askama::Template;
   ```

### Template Style Guidelines

1. **HTML Structure**
   ```html
   <!-- Use semantic HTML -->
   <main class="container mx-auto px-4">
       <header class="mb-8">
           <h1 class="text-3xl font-bold">{{ title }}</h1>
       </header>
       
       <section class="dashboard-content">
           <!-- Main content -->
       </section>
   </main>
   ```

2. **Tailwind CSS Usage**
   ```html
   <!-- Mobile-first responsive design -->
   <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
       <div class="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
           <!-- Card content -->
       </div>
   </div>
   
   <!-- Consistent spacing and typography -->
   <h2 class="text-2xl font-semibold text-gray-800 mb-4">Section Title</h2>
   <p class="text-gray-600 leading-relaxed">Description text.</p>
   ```

3. **Template Data Binding**
   ```html
   <!-- Safe data rendering -->
   <h1>{{ title|escape }}</h1>
   
   <!-- Conditional rendering -->
   {% if symbols|length > 0 %}
       <div class="symbols-grid">
       {% for symbol in symbols %}
           <div class="symbol-card">
               <h3>{{ symbol.symbol }}</h3>
               <p>{{ symbol.company_name|escape }}</p>
           </div>
       {% endfor %}
       </div>
   {% else %}
       <p class="text-gray-500">No symbols available.</p>
   {% endif %}
   ```

### JavaScript Style Guidelines

1. **Modern JavaScript Practices**
   ```javascript
   // Use async/await for API calls
   async function fetchStockData(symbol) {
       try {
           const response = await fetch(`/api/symbols/${symbol}/comprehensive`);
           if (!response.ok) {
               throw new Error(`HTTP ${response.status}: ${response.statusText}`);
           }
           const data = await response.json();
           return data;
       } catch (error) {
           console.error('Failed to fetch stock data:', error);
           showUserFriendlyError('Failed to load data. Please try again.');
           throw error;
       }
   }
   
   // Use arrow functions for short callbacks
   const symbols = ['AAPL', 'MSFT', 'GOOGL'];
   const promises = symbols.map(symbol => fetchStockData(symbol));
   ```

2. **Error Handling**
   ```javascript
   // Graceful error handling with user feedback
   function handleApiError(error) {
       const errorMessage = error.message || 'An unexpected error occurred';
       
       // Show user-friendly error message
       showNotification(errorMessage, 'error');
       
       // Log detailed error for debugging
       console.error('API Error:', error);
   }
   ```

3. **Performance Considerations**
   ```javascript
   // Debounce search input
   let searchTimeout;
   function debounceSearch(query) {
       clearTimeout(searchTimeout);
       searchTimeout = setTimeout(() => {
           performSearch(query);
       }, 300);
   }
   
   // Efficient DOM manipulation
   function updateSymbolList(symbols) {
       const container = document.getElementById('symbols-container');
       const fragment = document.createDocumentFragment();
       
       symbols.forEach(symbol => {
           const element = createSymbolElement(symbol);
           fragment.appendChild(element);
       });
       
       container.innerHTML = '';
       container.appendChild(fragment);
   }
   ```

### Documentation Standards

1. **Function Documentation**
   ```rust
   /// Renders the dashboard template with system metrics and popular symbols.
   /// 
   /// This endpoint provides an overview of the system including:
   /// - Current system health and performance metrics
   /// - Quick access to popular stock symbols
   /// - Navigation to other web interface features
   /// 
   /// # Returns
   /// 
   /// Returns a rendered HTML page with the dashboard interface, or a 404 error
   /// if the web-ui feature is not enabled.
   /// 
   /// # Examples
   /// 
   /// ```rust
   /// // Access dashboard
   /// GET /ui
   /// ```
   #[cfg(feature = "web-ui")]
   async fn dashboard() -> impl IntoResponse {
       // Implementation
   }
   ```

2. **Template Documentation**
   ```html
   {# 
   Dashboard Template
   
   Displays system overview and quick access to financial analysis tools.
   
   Required data:
   - title: Page title
   - symbols: List of popular symbols for quick access
   - metrics: System performance metrics
   
   Extends: base.html
   #}
   {% extends "base.html" %}
   ```

3. **API Documentation Updates**
   - Update API_REFERENCE.md when adding new endpoints
   - Document web interface features and capabilities
   - Include examples for both API and web interface usage
   - Maintain consistency between API and web interface documentation

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_symbol_validation() {
        assert!(validate_symbol("AAPL").is_ok());
        assert!(validate_symbol("").is_err());
        assert!(validate_symbol("TOOLONGSYMBOL").is_err());
    }
    
    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new();
        cache.insert("key", "value", Duration::from_millis(100)).await;
        
        // Should be present immediately
        assert!(cache.get("key").await.is_some());
        
        // Should expire after TTL
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(cache.get("key").await.is_none());
    }
}
```

### Web Interface Testing

```rust
// tests/web_ui_tests.rs
#[cfg(feature = "web-ui")]
mod web_ui_tests {
    use super::*;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_dashboard_loads() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/ui").await;
        assert_eq!(response.status_code(), 200);
        
        let body = response.text();
        assert!(body.contains("Mango Data Service"));
        assert!(body.contains("Dashboard"));
    }
    
    #[tokio::test]
    async fn test_analytics_with_symbol() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/ui/analytics?symbol=AAPL").await;
        assert_eq!(response.status_code(), 200);
        
        let body = response.text();
        assert!(body.contains("AAPL"));
        assert!(body.contains("Analytics"));
    }
    
    #[tokio::test]
    async fn test_web_ui_disabled_fallback() {
        // Test that endpoints return 404 when web-ui feature is disabled
        // This would be in a separate test configuration
    }
}
```

### Template Testing

```rust
#[cfg(test)]
mod template_tests {
    use super::*;
    use askama::Template;
    
    #[test]
    fn test_dashboard_template_renders() {
        let template = DashboardTemplate {
            title: "Test Dashboard".into(),
            symbols: vec![],
            metrics: SystemMetrics::default(),
        };
        
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Test Dashboard"));
        assert!(rendered.contains("<!DOCTYPE html>"));
    }
    
    #[test]
    fn test_template_with_symbols() {
        let symbols = vec![
            Symbol {
                symbol: "AAPL".into(),
                company_name: "Apple Inc.".into(),
            }
        ];
        
        let template = SearchTemplate { symbols };
        let rendered = template.render().unwrap();
        assert!(rendered.contains("AAPL"));
        assert!(rendered.contains("Apple Inc."));
    }
}
```

### Integration Tests

```rust
// tests/integration_tests.rs
use mango_data_service::*;
use axum_test::TestServer;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), 200);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    
    #[cfg(feature = "web-ui")]
    {
            assert!(body["data"]["features"].as_array().unwrap().contains(&serde_json::Value::String("rate_limiting".to_string())));
    assert!(body["data"]["features"].as_array().unwrap().contains(&serde_json::Value::String("caching".to_string())));
    }
}

#[tokio::test]
async fn test_api_and_web_consistency() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Test that API and web interface return consistent data
    let api_response = server.get("/api/symbols/AAPL/comprehensive").await;
    let api_data: serde_json::Value = api_response.json();
    
    #[cfg(feature = "web-ui")]
    {
        let web_response = server.get("/ui/analytics?symbol=AAPL").await;
        let web_body = web_response.text();
        
        // Verify that web interface includes data from API
        let price = api_data["data"]["latest_quote"]["price"].as_f64().unwrap();
        assert!(web_body.contains(&price.to_string()));
    }
}
```

### Frontend Testing

```javascript
// Example frontend test (if using a testing framework)
describe('Symbol Search', () => {
    test('should search for symbols', async () => {
        // Mock fetch
        global.fetch = jest.fn(() =>
            Promise.resolve({
                ok: true,
                json: () => Promise.resolve({
                    success: true,
                    data: {
                        symbols: [
                            { symbol: 'AAPL', company_name: 'Apple Inc.' }
                        ]
                    }
                })
            })
        );
        
        // Test search functionality
        const result = await searchSymbols('apple');
        expect(result.symbols).toHaveLength(1);
        expect(result.symbols[0].symbol).toBe('AAPL');
    });
});
```

### Performance Tests

```rust
#[tokio::test]
async fn test_concurrent_web_requests() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let start = std::time::Instant::now();
    
    // Spawn 100 concurrent requests to web interface
    let tasks: Vec<_> = (0..100).map(|_| {
        let server = server.clone();
        tokio::spawn(async move {
            server.get("/ui").await
        })
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    let duration = start.elapsed();
    
    // All requests should succeed
    for result in results {
        assert!(result.unwrap().status_code() == 200);
    }
    
    // Should complete within reasonable time
    assert!(duration < std::time::Duration::from_secs(10));
}
```

## Performance Optimization Guidelines

### Web Interface Optimizations

#### Template Performance
```rust
// Use Cow for template data to avoid unnecessary allocations
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: Cow<'static, str>,        // Static strings when possible
    pub symbols: Vec<Symbol>,            // Consider using Cow<'_, [Symbol]> for large lists
    pub cached_data: Cow<'_, str>,       // Borrowed when data comes from cache
}

// Efficient template data preparation
impl DashboardTemplate {
    pub fn new(title: &'static str, symbols: Vec<Symbol>) -> Self {
        Self {
            title: title.into(),
            symbols,
            cached_data: "cached".into(),
        }
    }
}
```

#### Frontend Performance
```javascript
// Efficient chart rendering with data limits
function createChart(data) {
    // Limit data points for performance
    const maxPoints = 1000;
    const chartData = data.length > maxPoints 
        ? data.slice(-maxPoints) 
        : data;
    
    // Use canvas rendering for better performance
    const ctx = canvas.getContext('2d');
    const chart = new Chart(ctx, {
        type: 'line',
        data: {
            datasets: [{
                data: chartData,
                pointRadius: 0, // Disable points for large datasets
                tension: 0.1
            }]
        },
        options: {
            responsive: true,
            animation: {
                duration: 0 // Disable animations for large datasets
            },
            scales: {
                x: { 
                    type: 'time',
                    ticks: { maxTicksLimit: 20 }
                }
            }
        }
    });
}

// Debounce user input
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Use efficient DOM updates
function updateSymbolList(symbols) {
    const container = document.getElementById('symbol-list');
    
    // Use DocumentFragment for efficient DOM manipulation
    const fragment = document.createDocumentFragment();
    
    symbols.forEach(symbol => {
        const element = document.createElement('div');
        element.className = 'symbol-item';
        element.innerHTML = `
            <span class="symbol">${symbol.symbol}</span>
            <span class="name">${symbol.company_name}</span>
        `;
        fragment.appendChild(element);
    });
    
    // Single DOM update
    container.innerHTML = '';
    container.appendChild(fragment);
}
```

#### Caching Strategies
   ```rust
// Template-specific caching
static TEMPLATE_CACHE: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

async fn render_cached_template<T: Template>(template: T, cache_key: &str) -> Result<String, Error> {
    // Check cache first (in production)
    if !cfg!(debug_assertions) {
        if let Some(cached) = TEMPLATE_CACHE.get(cache_key) {
            return Ok(cached.clone());
        }
    }
    
    // Render template
    let rendered = template.render()?;
    
    // Cache result (in production)
    if !cfg!(debug_assertions) {
        TEMPLATE_CACHE.insert(cache_key.to_string(), rendered.clone());
    }
    
    Ok(rendered)
}
```

### Memory Efficiency

#### Cow Usage Patterns
```rust
// Template data with optimal Cow usage
pub struct AnalyticsData {
    pub symbol: Cow<'static, str>,           // Often static
    pub company_name: Cow<'_, str>,          // From database, can be borrowed
    pub price_data: Vec<PricePoint>,         // Owned when necessary
    pub indicators: Cow<'_, [Indicator]>,    // Can be borrowed from cache
}

// Efficient data transformation
impl From<DatabaseSymbol> for AnalyticsData {
    fn from(db_symbol: DatabaseSymbol) -> Self {
        Self {
            symbol: db_symbol.symbol.into(),      // Convert String to Cow
            company_name: db_symbol.name.into(),  // Borrow when possible
            price_data: db_symbol.prices,         // Own the vec
            indicators: Cow::Borrowed(&[]),       // Borrow when available
        }
    }
}
```

### Database Optimization for Web Interface

   ```rust
// Optimized queries for web interface
impl Database {
    // Efficient symbol search for web interface
    pub async fn search_symbols_for_web(&self, query: &str, limit: usize) -> Result<Vec<Symbol>, Error> {
   sqlx::query_as!(
            Symbol,
            r#"
            SELECT symbol, company_name, exchange, currency, market_cap
            FROM symbols 
            WHERE symbol LIKE ? OR company_name LIKE ?
            ORDER BY 
                CASE WHEN symbol = ? THEN 1 ELSE 2 END,
                market_cap DESC
            LIMIT ?
            "#,
            format!("%{}%", query),
            format!("%{}%", query),
            query.to_uppercase(),
            limit as i64
        )
        .fetch_all(&self.pool)
   .await
        .map_err(Into::into)
    }
    
    // Batch load popular symbols for dashboard
    pub async fn get_popular_symbols(&self, limit: usize) -> Result<Vec<Symbol>, Error> {
        sqlx::query_as!(
            Symbol,
            "SELECT * FROM symbols ORDER BY market_cap DESC LIMIT ?",
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
   ```

## Debugging and Troubleshooting

### Logging Configuration

```rust
// Enhanced logging for web interface
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "mango_data_service=debug,tower_http=debug,askama=debug".into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Template rendering logging
#[tracing::instrument(skip(template))]
async fn render_template<T: Template>(template: T, template_name: &str) -> Result<String, Error> {
    tracing::debug!("Rendering template: {}", template_name);
    
    let start = std::time::Instant::now();
    let result = template.render().map_err(|e| {
        tracing::error!("Template rendering failed: {}", e);
        Error::TemplateError(e)
    });
    let duration = start.elapsed();
    
    match &result {
        Ok(_) => tracing::info!(
            "Successfully rendered template '{}' in {:?}",
            template_name,
            duration
        ),
        Err(e) => tracing::error!("Failed to render template '{}': {}", template_name, e),
    }
    
    result
}
```

### Common Issues and Solutions

1. **Template Compilation Issues**
   ```bash
   # Error: Template not found
   error: template not found: 'dashboard.html'
   
   # Solution: Check template path and cargo features
   cargo check --features web-ui
   ls templates/  # Verify file exists
   ```

2. **Feature Flag Issues**
   ```rust
   // Error: web_ui module not found
   // Solution: Ensure proper feature gating
   
   #[cfg(feature = "web-ui")]
   use crate::web_ui;
   
   // Always provide fallback routes
   #[cfg(not(feature = "web-ui"))]
   async fn fallback_handler() -> StatusCode {
       StatusCode::NOT_FOUND
   }
   ```

3. **Static Asset Issues**
   ```html
   <!-- Problem: Assets not loading -->
   <link href="https://cdnjs.cloudflare.com/ajax/libs/tailwindcss/2.2.19/tailwind.min.css" rel="stylesheet">
   
   <!-- Solution: Use CDN or implement static file serving -->
   <!-- For development, CDN is sufficient -->
   ```

4. **JavaScript Errors**
   ```javascript
   // Problem: API calls failing
   // Solution: Add proper error handling
   
   async function safeApiCall(url) {
       try {
           const response = await fetch(url);
           if (!response.ok) {
               throw new Error(`HTTP ${response.status}: ${response.statusText}`);
           }
           return await response.json();
       } catch (error) {
           console.error('API call failed:', error);
           showUserFriendlyError('Failed to load data. Please try again.');
           throw error;
       }
   }
   ```

### Debugging Tools

1. **Template Debugging**
   ```rust
   // Add debug prints to templates (development only)
   #[cfg(debug_assertions)]
   fn debug_template_data<T: std::fmt::Debug>(data: &T, name: &str) {
       tracing::debug!("Template data for {}: {:#?}", name, data);
   }
   
   #[cfg(not(debug_assertions))]
   fn debug_template_data<T>(_data: &T, _name: &str) {
       // No-op in production
   }
   ```

2. **Frontend Debugging**
   ```javascript
   // Development-only debugging helpers
   if (window.location.hostname === 'localhost') {
       window.debugAPI = {
           fetchSymbol: (symbol) => fetch(`/api/symbols/${symbol}/comprehensive`).then(r => r.json()),
           clearCache: () => fetch('/api/admin/cache/cleanup', { method: 'POST' }),
           getStats: () => fetch('/api/stats').then(r => r.json())
       };
       
       console.log('Debug API available as window.debugAPI');
   }
   ```

3. **Performance Profiling**
   ```bash
   # Profile template rendering
   cargo flamegraph --bin mango-data-service --features web-ui
   
   # Memory profiling
   valgrind --tool=massif ./target/release/mango-data-service
   
   # Web interface performance
   lighthouse http://localhost:3000/ui --output html
   ```

## Contributing Guidelines

### Contributing to Web Interface

When contributing to the web interface:

1. **Template Changes**:
   - Follow Askama template syntax
   - Use Tailwind CSS for styling
   - Ensure responsive design (mobile-first)
   - Test on multiple screen sizes
   - Maintain accessibility standards

2. **JavaScript Features**:
   - Use vanilla JavaScript or integrate with Chart.js
   - Ensure compatibility across modern browsers
   - Follow existing code patterns and naming conventions
   - Add comments for complex functionality
   - Include error handling and user feedback

3. **Feature Development**:
   - Always use `#[cfg(feature = "web-ui")]` for web-specific code
   - Provide fallback implementations for disabled features
   - Update both API and web interface documentation
   - Add tests for new web features
   - Consider performance impact on both server and client

4. **Testing Requirements**:
   - Unit tests for Rust code
   - Template compilation tests
   - Integration tests for web routes
   - Manual testing across browsers
   - Responsive design verification

### Development Guidelines

- **Performance First**: Consider memory and CPU impact for both API and web interface
- **Progressive Enhancement**: Web interface should enhance, not replace, API functionality
- **Feature Parity**: Maintain consistency between API and web interface capabilities
- **Error Handling**: Comprehensive error handling for both server and client-side
- **Security**: Validate all inputs, especially in web interface forms
- **Accessibility**: Ensure web interface is accessible to all users
- **Documentation**: Update docs for both API and web interface changes

### Code Review Checklist

- [ ] Tests added and passing (API and web interface)
- [ ] Template syntax valid and compiles successfully
- [ ] Responsive design maintained and tested
- [ ] Feature flags properly implemented
- [ ] Documentation updated (including web interface features)
- [ ] Performance impact considered and optimized
- [ ] Error handling implemented for both success and failure cases
- [ ] Cross-browser compatibility verified
- [ ] Accessibility guidelines followed
- [ ] Mobile responsiveness tested
- [ ] JavaScript functionality working correctly
- [ ] Template data properly escaped to prevent XSS
- [ ] Rate limiting considerations for new endpoints

---

This comprehensive development guide covers all aspects of developing for the Mango Data Service, including the advanced web interface capabilities. Happy coding! 🦀✨ 