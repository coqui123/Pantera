# 📚 API Reference

Complete API documentation for Mango Data Service - a high-performance financial data platform with optional web interface.

## Base Information

- **Base URL**: `http://localhost:3000`
- **Content-Type**: `application/json`
- **Rate Limiting**: IP-based with token bucket algorithm
- **Authentication**: None required (currently)
- **Web Interface**: Available when built with `--features web-ui`

## Response Format

### Success Response
```json
{
  "success": true,
  "data": { ... },
  "message": "Optional success message",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Error Response
```json
{
  "success": false,
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Rate Limit Response
```json
{
  "success": false,
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after": 60,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Web Interface Endpoints

### Overview
When the service is built with the `web-ui` feature (`cargo run --features web-ui`), additional web interface endpoints become available. These provide a professional financial analysis interface with interactive charts, technical indicators, and real-time data visualization.

#### GET /
Redirects to the main dashboard (`/ui`).

**Response**: HTTP 302 redirect to `/ui`

#### GET /ui
Main dashboard interface providing system overview and quick access to financial analysis tools.

**Features:**
- System health monitoring
- Database and cache statistics
- Quick actions for popular stocks
- Navigation to all web features
- Real-time metrics display

**Template**: `dashboard.html`
**Styling**: Tailwind CSS with professional gradient themes

#### GET /ui/search
Symbol search and management interface with real-time search capabilities.

**Query Parameters:**
- `q` (optional): Pre-populate search query

**Features:**
- Real-time symbol search with autocomplete
- Company information display
- Symbol validation tools
- Bulk operations interface
- Search history and suggestions

**Template**: `search.html`

**Example:**
```
GET /ui/search?q=apple
```

#### GET /ui/analytics
Advanced financial analytics suite with professional charting and technical analysis.

**Query Parameters:**
- `symbol` (optional): Pre-load analysis for specific symbol

**Features:**
- Interactive candlestick and line charts
- Technical indicators (RSI, MACD, SMA, EMA, Bollinger Bands)
- Multiple timeframe analysis (5m, 15m, 30m, 1h, 1d, 1wk, 1mo)
- Price analysis and volatility calculations
- Volume analysis and trends
- Risk metrics (Sharpe ratio, VaR, drawdown)
- Export capabilities (CSV, JSON, PDF)

**Template**: `analytics.html`

**Example:**
```
GET /ui/analytics?symbol=AAPL
```

### Web Interface Architecture

**Technology Stack:**
- **Backend**: Rust with Askama templating engine
- **Frontend**: HTML5, CSS3, JavaScript
- **Styling**: Tailwind CSS for responsive design
- **Charts**: Chart.js for interactive visualizations
- **Icons**: FontAwesome for professional appearance

**Performance Optimizations:**
- Server-side template compilation
- Efficient data structures for chart rendering
- Local storage for user preferences
- Debounced search queries
- Optimized static asset delivery

## API Endpoints

### Health & System

#### GET /health
Enhanced health check endpoint with comprehensive system information.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "timestamp": "2024-01-01T00:00:00Z",
    "features": ["rate_limiting", "caching", "cow_optimization"]
  }
}
```

**Features Information:**
- `rate_limiting`: Token bucket algorithm active for API requests
- `caching`: DashMap-based concurrent caching enabled
- `cow_optimization`: Zero-copy string operations for memory efficiency

#### GET /api/stats
Database and cache statistics with system performance metrics.

**Response:**
```json
{
  "success": true,
  "data": {
    "database": {
      "symbols_count": 500,
      "historical_records_count": 125000,
      "realtime_quotes_count": 500,
      "company_profiles_count": 400,
      "symbols": 500,
      "historical_prices": 125000,
      "realtime_quotes": 500,
      "company_profiles": 400,
      "timestamp": "2024-01-01T00:00:00Z"
    },
    "cache": {
      "historical_cache_size": 150,
      "quote_cache_size": 75,
      "profile_cache_size": 50
    },
    "rate_limits": {
      "api_requests_per_minute": 100,
      "yahoo_api_requests_per_minute": 30
    }
  }
}
```

### Symbol Management

#### GET /api/symbols/search
Search for symbols by name or ticker with enhanced fuzzy matching.

**Parameters:**
- `q` (required): Search query
- `limit` (optional): Max results (default: 10, max: 50)

**Example:**
```bash
GET /api/symbols/search?q=apple&limit=5
```

**Response:**
```json
{
  "success": true,
  "data": {
    "symbols": [
      {
        "symbol": "AAPL",
        "company_name": "Apple Inc.",
        "exchange": "NASDAQ",
        "currency": "USD",
        "market_cap": 3000000000000,
        "sector": "Technology",
        "industry": "Consumer Electronics"
      }
    ],
    "total_found": 1,
    "query": "apple",
    "limit": 5,
    "search_time_ms": 12
  }
}
```

**Caching:**
- TTL: 1 hour
- Cache key includes query and limit
- Supports fuzzy matching and company name search

#### GET /api/symbols/{symbol}/validate
Validate if a symbol exists and is tradeable with comprehensive metadata.

**Parameters:**
- `symbol` (path): Stock symbol (e.g., AAPL)

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "valid": true,
    "company_name": "Apple Inc.",
    "exchange": "NASDAQ",
    "currency": "USD",
    "tradeable": true,
    "market_state": "OPEN",
    "last_updated": "2024-01-01T12:00:00Z",
    "data_availability": {
      "historical": true,
      "real_time": true,
      "profile": true
    }
  }
}
```

**Caching:**
- TTL: 24 hours
- Reduces validation overhead for frequently queried symbols

### Historical Data

#### GET /api/symbols/{symbol}/historical
Get historical price data with advanced filtering and optimization.

**Parameters:**
- `symbol` (path): Stock symbol
- `interval` (optional): Time interval (default: 1d)
  - Valid: `1m`, `5m`, `15m`, `30m`, `1h`, `1d`, `1wk`, `1mo`, `3mo`, `6mo`, `1y`, `2y`, `5y`, `10y`, `ytd`, `max`
- `limit` (optional): Number of records (default: 100, max: 1000)
- `start_date` (optional): Start date (ISO 8601)
- `end_date` (optional): End date (ISO 8601)
- `force_refresh` (optional): Bypass cache (default: false)

**Example:**
```bash
GET /api/symbols/AAPL/historical?interval=1d&limit=10
```

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "interval": "1d",
    "data": [
      {
        "timestamp": "2024-01-01T00:00:00Z",
        "open": 150.00,
        "high": 155.00,
        "low": 149.00,
        "close": 154.00,
        "volume": 50000000,
        "adj_close": 154.00
      }
    ],
    "total_records": 10,
    "cached": true,
    "cache_timestamp": "2024-01-01T12:00:00Z",
    "data_quality": {
      "completeness": 1.0,
      "gaps": 0,
      "adjusted_splits": true
    }
  }
}
```

**Smart Caching:**
- Intraday data: 5 minutes TTL
- Daily+ data: 1 hour TTL
- Market hours aware refresh
- Automatic cache warming for popular symbols

#### POST /api/symbols/{symbol}/fetch
Force fetch fresh data from Yahoo Finance with rate limiting protection.

**Parameters:**
- `symbol` (path): Stock symbol
- `interval` (optional): Time interval (default: 1d)

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "interval": "1d",
    "records_fetched": 250,
    "fetch_timestamp": "2024-01-01T12:00:00Z",
    "rate_limited": false,
    "cache_updated": true,
    "data_freshness": "real_time"
  }
}
```

#### GET /api/bulk/historical
Fetch historical data for multiple symbols with concurrent processing.

**Parameters:**
- `symbols` (required): Comma-separated symbols (max 20)
- `interval` (optional): Time interval (default: 1d)
- `limit` (optional): Records per symbol (default: 100)
- `max_concurrent` (optional): Concurrent requests (default: 5, max: 10)

**Example:**
```bash
GET /api/bulk/historical?symbols=AAPL,MSFT,GOOGL&interval=1d&limit=10
```

**Response:**
```json
{
  "success": true,
  "data": {
    "results": {
      "AAPL": {
        "success": true,
        "data": [...],
        "records": 10
      },
      "MSFT": {
        "success": true,
        "data": [...],
        "records": 10
      },
      "GOOGL": {
        "success": false,
        "error": "Rate limited",
        "retry_after": 30
      }
    },
    "summary": {
      "total_symbols": 3,
      "successful": 2,
      "failed": 1,
      "total_records": 20,
      "processing_time_ms": 450
    }
  }
}
```

**Concurrency Control:**
- Semaphore-based limiting to prevent API overload
- Intelligent retry with exponential backoff
- Per-symbol error handling

### Real-time Data

#### GET /api/symbols/{symbol}/quote
Get latest quote with market state awareness and comprehensive pricing data.

**Parameters:**
- `symbol` (path): Stock symbol

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "price": 154.00,
    "change": 2.50,
    "change_percent": 1.65,
    "volume": 45000000,
    "market_cap": 2500000000000,
    "pe_ratio": 25.5,
    "day_high": 155.00,
    "day_low": 151.00,
    "fifty_two_week_high": 180.00,
    "fifty_two_week_low": 120.00,
    "timestamp": "2024-01-01T16:00:00Z",
    "market_state": "CLOSED",
    "pre_market": {
      "price": 153.80,
      "change": -0.20,
      "volume": 1200000
    },
    "after_hours": {
      "price": 154.50,
      "change": 0.50,
      "volume": 800000
    },
    "cached": true,
    "cache_ttl": 300
  }
}
```

**Market-Aware Caching:**
- Market hours: 5 minutes TTL
- After hours: 15 minutes TTL
- Pre-market: 10 minutes TTL
- Weekends: 1 hour TTL

#### GET /api/symbols/{symbol}/profile
Get comprehensive company profile information with enhanced metadata.

**Parameters:**
- `symbol` (path): Stock symbol

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "company_name": "Apple Inc.",
    "sector": "Technology",
    "industry": "Consumer Electronics",
    "description": "Apple Inc. designs, manufactures, and markets smartphones...",
    "website": "https://www.apple.com",
    "employees": 164000,
    "headquarters": "Cupertino, CA",
    "founded": "1976-04-01",
    "market_cap": 2500000000000,
    "enterprise_value": 2600000000000,
    "financial_metrics": {
      "revenue_ttm": 365000000000,
      "gross_margin": 0.42,
      "operating_margin": 0.30,
      "profit_margin": 0.25
    },
    "cached": true,
    "cache_timestamp": "2024-01-01T12:00:00Z"
  }
}
```

### Advanced Analytics

#### GET /api/symbols/{symbol}/comprehensive
Comprehensive data combining multiple sources with enhanced analytics.

**Parameters:**
- `symbol` (path): Stock symbol
- `include_analysis` (optional): Include technical analysis (default: true)

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "data_sources": ["quote", "profile", "historical", "technical"],
    "latest_quote": {
      "price": 154.00,
      "change": 2.50,
      "volume": 45000000,
      "market_state": "CLOSED"
    },
    "company_profile": {
      "company_name": "Apple Inc.",
      "sector": "Technology",
      "market_cap": 2500000000000
    },
    "analysis": {
      "price_change_5d_percent": -2.5,
      "avg_volume_5d": 50000000,
      "volatility_30d": 0.25,
      "trend": "bullish"
    },
    "technical_indicators": {
      "rsi_14": 65.5,
      "sma_20": 152.3,
      "ema_20": 153.1,
      "macd": {
        "macd": 1.2,
        "signal": 0.8,
        "histogram": 0.4
      }
    },
    "ohlc_data": {
      "open": 152.00,
      "high": 155.00,
      "low": 151.00,
      "close": 154.00
    },
    "performance_metrics": {
      "api_calls": 4,
      "cache_hits": 3,
      "processing_time_ms": 89
    }
  }
}
```

#### GET /api/symbols/{symbol}/extended
Extended multi-interval analysis with comprehensive range statistics.

**Parameters:**
- `symbol` (path): Stock symbol
- `intervals` (optional): Comma-separated intervals (default: "1d,1wk")

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "data_sources": ["1d", "1wk", "1mo"],
    "range_analysis": {
      "period": "1 month",
      "price_stats": {
        "min": 145.00,
        "max": 165.00,
        "avg": 155.00,
        "volatility": 0.12,
        "skewness": 0.15,
        "kurtosis": -0.8
      },
      "volume_stats": {
        "avg_volume": 48000000,
        "max_volume": 85000000,
        "min_volume": 25000000,
        "volume_trend": "increasing"
      },
      "trend_analysis": {
        "direction": "upward",
        "strength": 0.75,
        "support_levels": [150.0, 145.0],
        "resistance_levels": [160.0, 165.0]
      }
    },
    "intervals": {
      "1d": {
        "records": 30,
        "latest_price": 154.00,
        "period_return": 0.05
      },
      "1wk": {
        "records": 4,
        "latest_price": 154.00,
        "period_return": 0.02
      }
    }
  }
}
```

#### GET /api/symbols/{symbol}/analysis
Statistical price analysis with enhanced risk metrics.

**Parameters:**
- `symbol` (path): Stock symbol
- `limit` (optional): Days to analyze (default: 30, max: 365)
- `days` (optional): Alias for limit
- `include_risk` (optional): Include risk metrics (default: true)

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "period_days": 30,
    "min_price": 145.00,
    "max_price": 165.00,
    "avg_price": 155.00,
    "volatility": 0.25,
    "price_change_percent": 5.5,
    "analysis": {
      "price_stats": {
        "min": 145.00,
        "max": 165.00,
        "mean": 155.00,
        "median": 154.50,
        "std_dev": 5.2,
        "variance": 27.04
      },
      "volume_stats": {
        "avg_volume": 48000000,
        "max_volume": 85000000,
        "min_volume": 25000000,
        "volume_volatility": 0.15
      },
      "returns": {
        "daily_returns": [-0.01, 0.02, -0.005],
        "cumulative_return": 0.055,
        "annualized_return": 0.22,
        "sharpe_ratio": 1.2,
        "sortino_ratio": 1.5
      },
      "risk_metrics": {
        "value_at_risk_5": -0.025,
        "expected_shortfall": -0.035,
        "maximum_drawdown": -0.08,
        "beta": 1.15,
        "correlation_sp500": 0.78
      }
    },
    "calculated_at": "2024-01-01T12:00:00Z"
  }
}
```

### Technical Indicators

#### GET /api/symbols/{symbol}/indicators
Calculate and retrieve technical indicators for enhanced analysis.

**Parameters:**
- `symbol` (path): Stock symbol
- `indicators` (optional): Comma-separated list (default: "rsi,macd,sma,ema")
- `period` (optional): Analysis period in days (default: 30)

**Available Indicators:**
- `rsi`: Relative Strength Index
- `macd`: Moving Average Convergence Divergence
- `sma`: Simple Moving Average
- `ema`: Exponential Moving Average
- `bb`: Bollinger Bands
- `stoch`: Stochastic Oscillator

**Response:**
```json
{
  "success": true,
  "data": {
    "symbol": "AAPL",
    "period": 30,
    "indicators": {
      "rsi": {
        "current": 65.5,
        "signal": "neutral",
        "overbought_threshold": 70,
        "oversold_threshold": 30
      },
      "macd": {
        "macd": 1.2,
        "signal": 0.8,
        "histogram": 0.4,
        "trend": "bullish"
      },
      "moving_averages": {
        "sma_20": 152.3,
        "sma_50": 148.7,
        "ema_20": 153.1,
        "ema_50": 149.2
      },
      "bollinger_bands": {
        "upper": 160.0,
        "middle": 155.0,
        "lower": 150.0,
        "width": 10.0,
        "position": 0.4
      }
    },
    "interpretation": {
      "overall_signal": "bullish",
      "confidence": 0.75,
      "recommendations": [
        "Price above SMA indicating uptrend",
        "RSI in neutral zone",
        "MACD shows bullish momentum"
      ]
    }
  }
}
```

### Comparison and Portfolio

#### GET /api/compare
Compare multiple symbols with correlation and performance analysis.

**Parameters:**
- `symbols` (required): Comma-separated symbols (max 10)
- `period` (optional): Comparison period in days (default: 30)
- `benchmark` (optional): Benchmark symbol (default: SPY)

**Example:**
```bash
GET /api/compare?symbols=AAPL,MSFT,GOOGL&period=90&benchmark=SPY
```

**Response:**
```json
{
  "success": true,
  "data": {
    "symbols": ["AAPL", "MSFT", "GOOGL"],
    "benchmark": "SPY",
    "period": 90,
    "comparison": {
      "AAPL": {
        "return": 0.12,
        "volatility": 0.25,
        "sharpe_ratio": 1.2,
        "beta": 1.15,
        "correlation_benchmark": 0.78
      },
      "MSFT": {
        "return": 0.08,
        "volatility": 0.22,
        "sharpe_ratio": 0.9,
        "beta": 1.05,
        "correlation_benchmark": 0.82
      },
      "GOOGL": {
        "return": 0.15,
        "volatility": 0.30,
        "sharpe_ratio": 1.1,
        "beta": 1.25,
        "correlation_benchmark": 0.75
      }
    },
    "correlation_matrix": {
      "AAPL_MSFT": 0.65,
      "AAPL_GOOGL": 0.58,
      "MSFT_GOOGL": 0.72
    },
    "ranking": {
      "by_return": ["GOOGL", "AAPL", "MSFT"],
      "by_sharpe": ["AAPL", "GOOGL", "MSFT"],
      "by_risk_adjusted": ["AAPL", "MSFT", "GOOGL"]
    }
  }
}
```

### Admin Endpoints

#### POST /api/admin/cache/cleanup
Manually trigger comprehensive cache cleanup with detailed reporting.

**Response:**
```json
{
  "success": true,
  "data": {
    "cleaned_entries": 150,
    "remaining_entries": 2350,
    "memory_freed_mb": 12.5,
    "cleanup_duration_ms": 45,
    "cache_types_cleaned": {
      "quotes": 45,
      "historical": 80,
      "profiles": 15,
      "search": 10
    },
    "next_scheduled_cleanup": "2024-01-01T13:00:00Z"
  }
}
```

## Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `SYMBOL_NOT_FOUND` | Symbol does not exist | 404 |
| `INVALID_INTERVAL` | Invalid time interval specified | 400 |
| `INVALID_LIMIT` | Limit parameter out of range | 400 |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 |
| `YAHOO_API_ERROR` | Error from Yahoo Finance API | 502 |
| `DATABASE_ERROR` | Database operation failed | 500 |
| `CACHE_ERROR` | Cache operation failed | 500 |
| `VALIDATION_ERROR` | Input validation failed | 400 |
| `INTERNAL_ERROR` | Internal server error | 500 |
| `FEATURE_DISABLED` | Requested feature not enabled | 404 |
| `TEMPLATE_ERROR` | Template rendering failed (web-ui) | 500 |

## Rate Limiting

### API Rate Limits
- **General API**: 100 requests/minute per IP
- **Burst**: 10 requests
- **Yahoo API**: 30 requests/minute
- **Burst**: 5 requests
- **Web Interface**: Same limits apply to web-initiated requests

### Headers
Rate limit information is included in response headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
X-RateLimit-Type: api
```

### Rate Limit Response
When rate limited, the service returns:
```json
{
  "success": false,
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after": 60,
  "limit_type": "api",
  "current_usage": 101,
  "limit": 100,
  "reset_time": "2024-01-01T12:01:00Z",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Caching

### Cache TTL by Endpoint
- **Quotes**: 5 minutes (market hours), 1 hour (after hours)
- **Historical Data**: 1 hour (daily+), 5 minutes (intraday)
- **Company Profiles**: 24 hours
- **Search Results**: 1 hour
- **Symbol Validation**: 24 hours
- **Technical Indicators**: 15 minutes
- **Comparison Data**: 30 minutes

### Cache Headers
Cache information in responses:
```json
{
  "cached": true,
  "cache_timestamp": "2024-01-01T12:00:00Z",
  "cache_ttl": 3600,
  "cache_hit": true,
  "cache_key": "quote:AAPL"
}
```

### Cache Control
Force refresh cache with parameters:
- `force_refresh=true`: Bypass cache and fetch fresh data
- `cache_only=true`: Return cached data only (fail if not cached)

## Examples

### Python Client
```python
import requests
import json

base_url = "http://localhost:3000"

class MangoDataClient:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
        self.session = requests.Session()
    
    def get_comprehensive_data(self, symbol):
        """Get comprehensive data for a symbol"""
        response = self.session.get(f"{self.base_url}/api/symbols/{symbol}/comprehensive")
        return response.json()
    
    def get_technical_indicators(self, symbol, indicators="rsi,macd,sma"):
        """Get technical indicators"""
        response = self.session.get(
            f"{self.base_url}/api/symbols/{symbol}/indicators",
            params={"indicators": indicators}
        )
        return response.json()
    
    def compare_symbols(self, symbols, period=30):
        """Compare multiple symbols"""
        response = self.session.get(
            f"{self.base_url}/api/compare",
            params={"symbols": ",".join(symbols), "period": period}
        )
        return response.json()

# Usage example
client = MangoDataClient()

# Get comprehensive data
data = client.get_comprehensive_data("AAPL")
if data["success"]:
    price = data["data"]["latest_quote"]["price"]
    rsi = data["data"]["technical_indicators"]["rsi_14"]
    print(f"AAPL: ${price}, RSI: {rsi}")

# Compare stocks
comparison = client.compare_symbols(["AAPL", "MSFT", "GOOGL"])
if comparison["success"]:
    for symbol, metrics in comparison["data"]["comparison"].items():
        print(f"{symbol}: Return: {metrics['return']:.2%}, Sharpe: {metrics['sharpe_ratio']:.2f}")
```

### cURL Examples
```bash
# Health check with web interface info
curl http://localhost:3000/health

# Search symbols
curl "http://localhost:3000/api/symbols/search?q=apple&limit=5"

# Get comprehensive data
curl "http://localhost:3000/api/symbols/AAPL/comprehensive"

# Get technical indicators
curl "http://localhost:3000/api/symbols/AAPL/indicators?indicators=rsi,macd,bb"

# Compare symbols
curl "http://localhost:3000/api/compare?symbols=AAPL,MSFT,GOOGL&period=90"

# Bulk historical data
curl "http://localhost:3000/api/bulk/historical?symbols=AAPL,MSFT&interval=1d&limit=30"

# Access web interface (if enabled)
curl http://localhost:3000/ui
```

### JavaScript/Node.js
```javascript
const axios = require('axios');

class MangoDataAPI {
    constructor(baseURL = 'http://localhost:3000') {
        this.client = axios.create({
            baseURL,
            timeout: 10000,
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        // Add response interceptor for error handling
        this.client.interceptors.response.use(
            response => response,
            error => {
                if (error.response?.status === 429) {
                    console.log('Rate limited. Retry after:', error.response.headers['retry-after']);
                }
                return Promise.reject(error);
            }
        );
    }

    async getStockData(symbol) {
        try {
            const response = await this.client.get(`/api/symbols/${symbol}/comprehensive`);
            return response.data;
        } catch (error) {
            console.error('Error fetching stock data:', error.response?.data || error.message);
            throw error;
        }
    }

    async getTechnicalAnalysis(symbol, indicators = 'rsi,macd,sma,ema') {
        try {
            const response = await this.client.get(`/api/symbols/${symbol}/indicators`, {
                params: { indicators }
            });
            return response.data;
        } catch (error) {
            console.error('Error fetching technical analysis:', error.response?.data || error.message);
            throw error;
        }
    }

    async compareStocks(symbols, period = 30) {
        try {
            const response = await this.client.get('/api/compare', {
                params: {
                    symbols: symbols.join(','),
                    period
                }
            });
            return response.data;
        } catch (error) {
            console.error('Error comparing stocks:', error.response?.data || error.message);
            throw error;
        }
    }
}

// Usage example
async function analyzePortfolio() {
    const api = new MangoDataAPI();
    
    try {
        // Analyze individual stock
        const appleData = await api.getStockData('AAPL');
        console.log(`AAPL: $${appleData.data.latest_quote.price}`);
        
        // Get technical indicators
        const technical = await api.getTechnicalAnalysis('AAPL');
        console.log(`RSI: ${technical.data.indicators.rsi.current}`);
        
        // Compare portfolio
        const comparison = await api.compareStocks(['AAPL', 'MSFT', 'GOOGL'], 90);
        console.log('Portfolio Comparison:', comparison.data.ranking);
        
    } catch (error) {
        console.error('Analysis failed:', error.message);
    }
}

analyzePortfolio();
```

### Web Interface Integration
```html
<!DOCTYPE html>
<html>
<head>
    <title>Custom Financial Dashboard</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.9.1/chart.min.js"></script>
</head>
<body>
    <div id="chart-container">
        <canvas id="priceChart"></canvas>
    </div>
    
    <script>
        // Integrate with Mango Data Service API
        async function createPriceChart(symbol) {
            try {
                // Fetch data from API
                const response = await fetch(`/api/symbols/${symbol}/historical?interval=1d&limit=30`);
                const data = await response.json();
                
                if (!data.success) {
                    throw new Error(data.error);
                }
                
                // Prepare chart data
                const chartData = {
                    labels: data.data.data.map(d => new Date(d.timestamp).toLocaleDateString()),
                    datasets: [{
                        label: `${symbol} Price`,
                        data: data.data.data.map(d => d.close),
                        borderColor: 'rgb(75, 192, 192)',
                        backgroundColor: 'rgba(75, 192, 192, 0.2)',
                        tension: 0.1
                    }]
                };
                
                // Create chart
                const ctx = document.getElementById('priceChart').getContext('2d');
                new Chart(ctx, {
                    type: 'line',
                    data: chartData,
                    options: {
                        responsive: true,
                        plugins: {
                            title: {
                                display: true,
                                text: `${symbol} Price Chart`
                            }
                        },
                        scales: {
                            y: {
                                beginAtZero: false
                            }
                        }
                    }
                });
                
            } catch (error) {
                console.error('Failed to create chart:', error);
            }
        }
        
        // Load chart for Apple stock
        createPriceChart('AAPL');
    </script>
</body>
</html>
```

## Best Practices

### API Usage
1. **Respect Rate Limits**: Monitor rate limit headers and implement backoff strategies
2. **Use Bulk Endpoints**: Prefer bulk operations for multiple symbols
3. **Leverage Caching**: Use cached data when real-time accuracy isn't critical
4. **Handle Errors Gracefully**: Implement comprehensive error handling
5. **Optimize Queries**: Use appropriate limits and intervals for your use case

### Web Interface Integration
1. **Progressive Enhancement**: Build functionality that works with and without JavaScript
2. **Responsive Design**: Ensure compatibility across devices
3. **Performance**: Use efficient data visualization techniques
4. **Accessibility**: Follow web accessibility guidelines
5. **Error Handling**: Provide user-friendly error messages

### Security Considerations
1. **Input Validation**: Always validate symbol names and parameters
2. **Rate Limiting**: Implement client-side rate limiting
3. **HTTPS**: Use HTTPS in production environments
4. **CORS**: Configure CORS appropriately for your domain
5. **Monitoring**: Monitor for unusual API usage patterns

---

This API provides a comprehensive financial data platform with both programmatic access and web interface capabilities. For additional support, examples, and community discussions, visit our GitHub repository. 