# 📊 Mango Data Service - Examples

This directory contains example scripts, notebooks, and tests demonstrating how to use the Mango Data Service API and web interface for various financial data analysis tasks.

## Available Examples

### 1. Simple Analytics Demo (`simple_analytics_demo.py`) ⭐ **Start Here**

A concise demonstration of key analytics features with minimal output - perfect for getting started.

**Features:**
- Single stock analysis with key metrics
- Portfolio correlation analysis
- Clean, readable output
- Quick demonstration of capabilities

**Usage:**
```bash
# Install dependencies
pip install -r requirements.txt

# Run the simple demo
python simple_analytics_demo.py
```

### 2. Data Analytics with Pandas (`data_analytics_example.py`)

A comprehensive example showcasing financial data analytics using pandas and the Mango Data Service API.

**Features:**
- Historical price data analysis
- Technical indicators (SMA, EMA, RSI, MACD, Bollinger Bands)
- Risk metrics calculation (Sharpe ratio, VaR, maximum drawdown)
- Portfolio analysis and correlation
- Sector-based analysis
- Bulk data processing
- Comprehensive reporting

**Usage:**
```bash
# Run the comprehensive example
python data_analytics_example.py
```

### 3. Interactive Jupyter Notebook (`data_analytics_notebook.ipynb`)

An interactive notebook version of the data analytics example with visualizations.

**Features:**
- Step-by-step analysis workflow
- Interactive visualizations
- Trading signal generation
- Risk-return analysis
- Correlation heatmaps

**Usage:**
```bash
# Install Jupyter if not already installed
pip install jupyter

# Start Jupyter
jupyter notebook

# Open data_analytics_notebook.ipynb
```

### 4. Comprehensive API Tests (`comprehensive_tests.py`)

Extensive test suite covering all API endpoints, web interface routes, and data validation.

**Features:**
- Health check validation
- Historical data quality testing
- Quote data verification
- Bulk operations testing
- Error handling validation
- Web interface endpoint testing (when enabled)
- Rate limiting validation

**Usage:**
```bash
# Test API endpoints
python comprehensive_tests.py

# Test with web interface enabled
# (requires service running with --features web-ui)
python comprehensive_tests.py --test-web-ui
```

### 5. KWEB FXI Test (`test_kweb_fxi.py`)

Specific test for KWEB and FXI ETF data - demonstrates testing of previously problematic tickers.

**Usage:**
```bash
python test_kweb_fxi.py
```

## Web Interface Examples

### Accessing the Web Interface

When the service is built with the web interface feature, you can access several interactive interfaces:

**1. Dashboard Interface**
```bash
# Start service with web interface
cargo run --features web-ui

# Access dashboard at:
http://localhost:3000/ui
```

**2. Search Interface**
```bash
# Symbol search and management
http://localhost:3000/ui/search
```

**3. Analytics Interface**
```bash
# Advanced financial analytics with charts
http://localhost:3000/ui/analytics

# Pre-load specific symbol
http://localhost:3000/ui/analytics?symbol=AAPL
```

### Web Interface Features

The web interface provides:
- **Interactive Charts**: Candlestick charts with technical indicators
- **Real-time Search**: Symbol search with autocomplete
- **Technical Analysis**: RSI, MACD, SMA, EMA, Bollinger Bands
- **Portfolio Tools**: Comparison and analysis features
- **Export Options**: CSV, JSON data export
- **Responsive Design**: Works on desktop, tablet, and mobile

## Prerequisites

### Required Dependencies

Install the core dependencies:

```bash
pip install -r requirements.txt
```

**Core packages:**
- `pandas` - Data manipulation and analysis
- `numpy` - Numerical computing
- `requests` - HTTP requests for API calls

**Optional packages for enhanced functionality:**
- `matplotlib` - Basic plotting
- `seaborn` - Statistical visualizations
- `plotly` - Interactive visualizations
- `scipy` - Scientific computing
- `scikit-learn` - Machine learning
- `jupyter` - Interactive notebooks

### Service Requirements

Make sure the Mango Data Service is running:

**API Only:**
```bash
cargo run
# Service accessible at: http://localhost:3000
```

**With Web Interface:**
```bash
cargo run --features web-ui
# API accessible at: http://localhost:3000
# Web interface at: http://localhost:3000/ui
```

**Health Check:**
```bash
curl http://localhost:3000/health
```

## Quick Start

### Option 1: API Examples

1. **Start the Mango Data Service** (see main README for instructions)

2. **Install dependencies:**
   ```bash
   cd examples
   pip install -r requirements.txt
   ```

3. **Run the simple demo:**
   ```bash
   python simple_analytics_demo.py
   ```

4. **Or start with the interactive notebook:**
   ```bash
   jupyter notebook data_analytics_notebook.ipynb
   ```

### Option 2: Web Interface

1. **Start the service with web interface:**
   ```bash
   cargo run --features web-ui
   ```

2. **Open web interface:**
   ```bash
   # Dashboard
   open http://localhost:3000/ui
   
   # Analytics (with AAPL pre-loaded)
   open http://localhost:3000/ui/analytics?symbol=AAPL
   ```

3. **Explore the features:**
   - Use the search interface to find symbols
   - View interactive charts and technical indicators
   - Export data for further analysis

## Example Workflows

### Basic Stock Analysis (API)

```python
from data_analytics_example import MangoDataAnalytics

# Initialize client
analytics = MangoDataAnalytics()

# Get historical data
data = analytics.get_historical_data("AAPL", limit=252)

# Calculate technical indicators
data_with_indicators = analytics.calculate_technical_indicators(data)

# Calculate risk metrics
risk_metrics = analytics.calculate_risk_metrics(data_with_indicators)

print(f"Sharpe Ratio: {risk_metrics['sharpe_ratio']:.3f}")
```

### Advanced API Usage

```python
# Get comprehensive data (includes technical analysis)
import requests

response = requests.get("http://localhost:3000/api/symbols/AAPL/comprehensive")
data = response.json()

if data["success"]:
    quote = data["data"]["latest_quote"]
    indicators = data["data"]["technical_indicators"]
    analysis = data["data"]["analysis"]
    
    print(f"Price: ${quote['price']}")
    print(f"RSI: {indicators['rsi_14']}")
    print(f"Trend: {analysis['trend']}")
```

### Portfolio Analysis

```python
# Analyze a portfolio
symbols = ["AAPL", "MSFT", "GOOGL"]
portfolio_results = analytics.analyze_portfolio(symbols)

# Get correlation matrix
correlation_matrix = portfolio_results['correlation_matrix']
print(correlation_matrix)

# Generate comprehensive report
report = analytics.generate_report(symbols, "my_report.txt")
```

### Using New Comparison Endpoint

```python
# Compare multiple symbols
import requests

response = requests.get(
    "http://localhost:3000/api/compare?symbols=AAPL,MSFT,GOOGL&period=90"
)
comparison = response.json()

if comparison["success"]:
    for symbol, metrics in comparison["data"]["comparison"].items():
        print(f"{symbol}: Return: {metrics['return']:.2%}, "
              f"Sharpe: {metrics['sharpe_ratio']:.2f}")
```

### Bulk Data Processing

```python
# Fetch data for multiple symbols efficiently
symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"]

# Using new bulk endpoint
response = requests.get(
    f"http://localhost:3000/api/bulk/historical"
    f"?symbols={','.join(symbols)}&interval=1d&limit=100"
)
bulk_data = response.json()

if bulk_data["success"]:
    for symbol, result in bulk_data["data"]["results"].items():
        if result["success"]:
            print(f"{symbol}: {result['records']} records")
```

## Advanced Usage

### Custom Analysis

You can extend the `MangoDataAnalytics` class for custom analysis:

```python
class CustomAnalytics(MangoDataAnalytics):
    def custom_indicator(self, df):
        # Your custom technical indicator
        return df['close'].rolling(window=10).mean()
    
    def custom_strategy(self, df):
        # Your custom trading strategy
        signals = pd.DataFrame(index=df.index)
        signals['signal'] = np.where(df['rsi'] < 30, 1, 0)
        return signals
```

### Integration with Web Interface

```python
# Fetch data and open in web interface
import webbrowser
import requests

# Get symbol data via API
symbol = "AAPL"
response = requests.get(f"http://localhost:3000/api/symbols/{symbol}/validate")

if response.json().get("success"):
    # Open in web analytics interface
    webbrowser.open(f"http://localhost:3000/ui/analytics?symbol={symbol}")
```

### Integration with Other Libraries

```python
# With scikit-learn for machine learning
from sklearn.ensemble import RandomForestRegressor

# With plotly for interactive charts
import plotly.graph_objects as go

# With statsmodels for statistical analysis
import statsmodels.api as sm
```

## Testing Examples

### API Testing

```bash
# Run comprehensive test suite
python comprehensive_tests.py

# Test specific endpoints
python test_kweb_fxi.py
```

### Web Interface Testing

```python
# Test web interface endpoints
import requests

# Test dashboard loads
response = requests.get("http://localhost:3000/ui")
assert response.status_code == 200

# Test analytics page with symbol
response = requests.get("http://localhost:3000/ui/analytics?symbol=AAPL")
assert response.status_code == 200

# Test search interface
response = requests.get("http://localhost:3000/ui/search")
assert response.status_code == 200
```

## Output Files

The examples generate several output files:

- `analytics_report.txt` - Comprehensive analysis report
- `portfolio_analysis_report.txt` - Portfolio-specific report
- Various CSV files with processed data
- Web interface allows direct export of chart data

## Development and Testing

### Testing Service Configurations

**Test API Only:**
```bash
# Start service
cargo run

# Run API tests
python comprehensive_tests.py
```

**Test with Web Interface:**
```bash
# Start service with web UI
cargo run --features web-ui

# Test both API and web interface
python comprehensive_tests.py --test-web-ui
```

### Performance Testing

```python
# Test bulk operations
import time
import requests

symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"]
start_time = time.time()

response = requests.get(
    f"http://localhost:3000/api/bulk/historical"
    f"?symbols={','.join(symbols)}&interval=1d&limit=100&max_concurrent=5"
)

end_time = time.time()
print(f"Bulk fetch took {end_time - start_time:.2f} seconds")
```

## Troubleshooting

### Common Issues

1. **API Connection Error**
   ```
   ❌ Request failed for /api/symbols/AAPL/historical: Connection refused
   ```
   **Solution:** Make sure the Mango Data Service is running on `http://localhost:3000`

2. **Web Interface Not Available**
   ```
   ❌ 404 Not Found for /ui
   ```
   **Solution:** Start the service with web interface enabled: `cargo run --features web-ui`

3. **Missing Dependencies**
   ```
   ModuleNotFoundError: No module named 'pandas'
   ```
   **Solution:** Install dependencies with `pip install -r requirements.txt`

4. **No Data Available**
   ```
   ❌ No historical data found for SYMBOL
   ```
   **Solution:** Check if the symbol is valid and data is available in the service

5. **Template Not Found (Web Interface)**
   ```
   ❌ Template rendering failed
   ```
   **Solution:** Ensure the service was built with `--features web-ui` and templates are available

### Performance Tips

- Use bulk endpoints for multiple symbols
- Implement caching for repeated requests
- Process large datasets in chunks
- Use appropriate time intervals for your analysis
- Use the web interface for interactive exploration before automating with scripts

### Web Interface Tips

- Use browser developer tools to inspect network requests
- The web interface uses the same API endpoints internally
- Export data from web interface for use in scripts
- Use URL parameters to pre-configure analytics views

## Contributing

To add new examples:

1. Create a new Python file in this directory
2. Follow the existing code style and documentation format
3. Add appropriate error handling and logging
4. Update this README with your example
5. Test thoroughly with different market conditions
6. Test both API-only and web interface scenarios if applicable

### Web Interface Examples

When adding web interface examples:
1. Ensure they work with both enabled and disabled web-ui feature
2. Provide graceful fallbacks when web interface is not available
3. Document any web-specific functionality
4. Include screenshots or descriptions of visual elements

## Documentation

For detailed documentation, see:
- [`../docs/DATA_ANALYTICS_GUIDE.md`](../docs/DATA_ANALYTICS_GUIDE.md) - Comprehensive analytics guide
- [`../docs/API_REFERENCE.md`](../docs/API_REFERENCE.md) - Complete API documentation including web interface endpoints
- [`../docs/DEVELOPMENT.md`](../docs/DEVELOPMENT.md) - Development guide including web interface development
- [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) - System architecture including web interface architecture

## Support

For questions or issues:
1. Check the troubleshooting section above
2. Review the comprehensive documentation
3. Examine the test files for usage patterns
4. Test both API and web interface features
5. Refer to the main project documentation

---

**Built with ❤️ and ⚡ in Rust** | **Professional Financial Analysis Platform** | **Happy Trading! 📊💰** 