# 📊 Data Analytics Guide

This guide demonstrates how to use the Mango Data Service API for comprehensive financial data analytics using pandas and other Python libraries.

## Overview

The `data_analytics_example.py` provides a complete framework for financial data analysis, including:

- **Historical Price Analysis**: Fetch and analyze historical stock data
- **Technical Indicators**: Calculate moving averages, RSI, MACD, Bollinger Bands
- **Risk Metrics**: Volatility, Sharpe ratio, VaR, maximum drawdown
- **Portfolio Analysis**: Multi-stock portfolio performance and correlation
- **Sector Analysis**: Group stocks by sector for comparative analysis
- **Bulk Data Processing**: Efficiently process multiple stocks simultaneously

## Quick Start

### Prerequisites

Install the required dependencies:

```bash
pip install -r examples/requirements.txt
```

### Basic Usage

```python
from examples.data_analytics_example import MangoDataAnalytics

# Initialize the analytics client
analytics = MangoDataAnalytics("http://localhost:3000")

# Analyze a single stock
aapl_data = analytics.get_historical_data("AAPL", limit=252)
aapl_with_indicators = analytics.calculate_technical_indicators(aapl_data)
risk_metrics = analytics.calculate_risk_metrics(aapl_with_indicators)

# Analyze a portfolio
portfolio_results = analytics.analyze_portfolio(["AAPL", "MSFT", "GOOGL"])

# Generate a comprehensive report
report = analytics.generate_report(["AAPL", "MSFT", "GOOGL"], "my_report.txt")
```

### Running the Example

```bash
cd examples
python data_analytics_example.py
```

## Core Features

### 1. Historical Data Analysis

The `get_historical_data()` method fetches historical price data and converts it to a pandas DataFrame:

```python
# Fetch 1 year of daily data
data = analytics.get_historical_data("AAPL", interval="1d", limit=252)

# Data includes: open, high, low, close, volume, adj_close
print(data.head())
```

**Available Intervals:**
- `1m`, `5m`, `15m`, `30m`, `1h` (intraday)
- `1d`, `1wk`, `1mo` (daily and longer)
- `3mo`, `6mo`, `1y`, `2y`, `5y`, `10y` (extended periods)

### 2. Technical Indicators

Calculate common technical indicators using `calculate_technical_indicators()`:

```python
data_with_indicators = analytics.calculate_technical_indicators(data)

# Available indicators:
# - Simple Moving Averages (SMA): 5, 10, 20, 50, 200 days
# - Exponential Moving Averages (EMA): 5, 10, 20, 50, 200 days
# - Bollinger Bands (20-day with 2 std dev)
# - RSI (14-day Relative Strength Index)
# - MACD (12-26-9 configuration)
# - Rolling Volatility (20-day)
```

### 3. Return Calculations

The `calculate_returns()` method computes various return metrics:

```python
data_with_returns = analytics.calculate_returns(data)

# Available return metrics:
# - daily_return: Daily percentage change
# - cumulative_return: Cumulative return from start
# - log_return: Logarithmic returns
# - return_5d, return_20d, return_60d: Rolling period returns
```

### 4. Risk Analysis

Comprehensive risk metrics via `calculate_risk_metrics()`:

```python
risk_metrics = analytics.calculate_risk_metrics(data_with_returns)

# Risk metrics include:
# - Annualized return and volatility
# - Sharpe ratio
# - Maximum drawdown
# - Value at Risk (95% and 99%)
# - Skewness and kurtosis
# - Positive/negative day ratios
# - Beta (if benchmark provided)
```

### 5. Portfolio Analysis

Analyze multiple stocks as a portfolio:

```python
symbols = ["AAPL", "MSFT", "GOOGL", "TSLA"]
weights = [0.3, 0.3, 0.2, 0.2]  # Optional, defaults to equal weight

portfolio_results = analytics.analyze_portfolio(symbols, weights)

# Results include:
# - Portfolio-level risk metrics
# - Correlation matrix between stocks
# - Individual stock metrics
# - Portfolio weights
# - Combined returns data
```

### 6. Bulk Data Processing

Efficiently fetch data for multiple symbols:

```python
symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"]
bulk_data = analytics.get_bulk_historical_data(symbols, limit=100)

# Returns dictionary: {symbol: DataFrame}
for symbol, df in bulk_data.items():
    print(f"{symbol}: {len(df)} records")
```

### 7. Sector Analysis

Group stocks by sector for comparative analysis:

```python
symbols = ["AAPL", "MSFT", "GOOGL", "JPM", "BAC", "XOM", "CVX"]
sector_analysis = analytics.sector_analysis(symbols)

# Groups stocks by sector and calculates:
# - Sector-level portfolio metrics
# - Average correlation within sector
# - Number of stocks per sector
```

## Advanced Usage Examples

### Custom Portfolio Optimization

```python
import numpy as np
from scipy.optimize import minimize

def optimize_portfolio(analytics, symbols, target_return=0.10):
    """
    Simple portfolio optimization example.
    """
    # Get historical data
    portfolio_data = analytics.get_bulk_historical_data(symbols, limit=252)
    
    # Calculate returns matrix
    returns_data = {}
    for symbol in symbols:
        if symbol in portfolio_data:
            df_with_returns = analytics.calculate_returns(portfolio_data[symbol])
            returns_data[symbol] = df_with_returns['daily_return']
    
    returns_df = pd.DataFrame(returns_data).dropna()
    
    # Calculate expected returns and covariance matrix
    expected_returns = returns_df.mean() * 252
    cov_matrix = returns_df.cov() * 252
    
    # Optimization function (minimize portfolio variance)
    def portfolio_variance(weights):
        return np.dot(weights.T, np.dot(cov_matrix, weights))
    
    # Constraints
    constraints = [
        {'type': 'eq', 'fun': lambda x: np.sum(x) - 1},  # Weights sum to 1
        {'type': 'eq', 'fun': lambda x: np.dot(x, expected_returns) - target_return}  # Target return
    ]
    
    # Bounds (0 to 1 for each weight)
    bounds = tuple((0, 1) for _ in range(len(symbols)))
    
    # Initial guess (equal weights)
    initial_weights = np.array([1/len(symbols)] * len(symbols))
    
    # Optimize
    result = minimize(portfolio_variance, initial_weights, 
                     method='SLSQP', bounds=bounds, constraints=constraints)
    
    return dict(zip(symbols, result.x))

# Example usage
optimal_weights = optimize_portfolio(analytics, ["AAPL", "MSFT", "GOOGL"])
print("Optimal weights:", optimal_weights)
```

### Technical Analysis Signals

```python
def generate_trading_signals(df):
    """
    Generate simple trading signals based on technical indicators.
    """
    signals = pd.DataFrame(index=df.index)
    
    # Moving average crossover
    signals['ma_signal'] = np.where(df['sma_20'] > df['sma_50'], 1, -1)
    
    # RSI overbought/oversold
    signals['rsi_signal'] = np.where(df['rsi'] < 30, 1,  # Oversold - buy
                                   np.where(df['rsi'] > 70, -1, 0))  # Overbought - sell
    
    # Bollinger Bands
    signals['bb_signal'] = np.where(df['close'] < df['bb_lower'], 1,  # Below lower band - buy
                                  np.where(df['close'] > df['bb_upper'], -1, 0))  # Above upper band - sell
    
    # MACD
    signals['macd_signal'] = np.where(df['macd'] > df['macd_signal'], 1, -1)
    
    # Combine signals (simple average)
    signals['combined_signal'] = signals[['ma_signal', 'rsi_signal', 'bb_signal', 'macd_signal']].mean(axis=1)
    
    return signals

# Example usage
aapl_data = analytics.get_historical_data("AAPL", limit=100)
aapl_with_indicators = analytics.calculate_technical_indicators(aapl_data)
signals = generate_trading_signals(aapl_with_indicators)
```

### Backtesting Framework

```python
def simple_backtest(df, signals, initial_capital=10000):
    """
    Simple backtesting framework.
    """
    portfolio = pd.DataFrame(index=df.index)
    portfolio['price'] = df['close']
    portfolio['signal'] = signals['combined_signal']
    
    # Generate positions (1 for long, 0 for cash, -1 for short)
    portfolio['position'] = portfolio['signal'].apply(lambda x: 1 if x > 0.1 else (-1 if x < -0.1 else 0))
    
    # Calculate returns
    portfolio['returns'] = df['close'].pct_change()
    portfolio['strategy_returns'] = portfolio['position'].shift(1) * portfolio['returns']
    
    # Calculate cumulative returns
    portfolio['cumulative_returns'] = (1 + portfolio['returns']).cumprod()
    portfolio['cumulative_strategy_returns'] = (1 + portfolio['strategy_returns']).cumprod()
    
    # Performance metrics
    total_return = portfolio['cumulative_strategy_returns'].iloc[-1] - 1
    benchmark_return = portfolio['cumulative_returns'].iloc[-1] - 1
    
    return {
        'total_return': total_return,
        'benchmark_return': benchmark_return,
        'excess_return': total_return - benchmark_return,
        'portfolio': portfolio
    }

# Example usage
backtest_results = simple_backtest(aapl_with_indicators, signals)
print(f"Strategy Return: {backtest_results['total_return']:.2%}")
print(f"Benchmark Return: {backtest_results['benchmark_return']:.2%}")
```

## Visualization Examples

### Price and Indicators Chart

```python
import matplotlib.pyplot as plt
import seaborn as sns

def plot_stock_analysis(df, symbol):
    """
    Create a comprehensive stock analysis chart.
    """
    fig, axes = plt.subplots(4, 1, figsize=(15, 12))
    
    # Price and moving averages
    axes[0].plot(df.index, df['close'], label='Close Price', linewidth=2)
    axes[0].plot(df.index, df['sma_20'], label='SMA 20', alpha=0.7)
    axes[0].plot(df.index, df['sma_50'], label='SMA 50', alpha=0.7)
    axes[0].fill_between(df.index, df['bb_lower'], df['bb_upper'], alpha=0.2, label='Bollinger Bands')
    axes[0].set_title(f'{symbol} - Price and Moving Averages')
    axes[0].legend()
    axes[0].grid(True, alpha=0.3)
    
    # Volume
    axes[1].bar(df.index, df['volume'], alpha=0.7, color='orange')
    axes[1].set_title('Volume')
    axes[1].grid(True, alpha=0.3)
    
    # RSI
    axes[2].plot(df.index, df['rsi'], color='purple', linewidth=2)
    axes[2].axhline(y=70, color='r', linestyle='--', alpha=0.7, label='Overbought')
    axes[2].axhline(y=30, color='g', linestyle='--', alpha=0.7, label='Oversold')
    axes[2].set_title('RSI (14-day)')
    axes[2].set_ylim(0, 100)
    axes[2].legend()
    axes[2].grid(True, alpha=0.3)
    
    # MACD
    axes[3].plot(df.index, df['macd'], label='MACD', linewidth=2)
    axes[3].plot(df.index, df['macd_signal'], label='Signal', linewidth=2)
    axes[3].bar(df.index, df['macd_histogram'], alpha=0.7, label='Histogram')
    axes[3].set_title('MACD')
    axes[3].legend()
    axes[3].grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.show()

# Example usage
aapl_data = analytics.get_historical_data("AAPL", limit=100)
aapl_with_indicators = analytics.calculate_technical_indicators(aapl_data)
plot_stock_analysis(aapl_with_indicators, "AAPL")
```

### Correlation Heatmap

```python
def plot_correlation_matrix(correlation_matrix, title="Stock Correlation Matrix"):
    """
    Plot correlation matrix as a heatmap.
    """
    plt.figure(figsize=(10, 8))
    sns.heatmap(correlation_matrix, annot=True, cmap='coolwarm', center=0,
                square=True, linewidths=0.5, cbar_kws={"shrink": .8})
    plt.title(title)
    plt.tight_layout()
    plt.show()

# Example usage
portfolio_results = analytics.analyze_portfolio(["AAPL", "MSFT", "GOOGL"])
if 'correlation_matrix' in portfolio_results:
    plot_correlation_matrix(portfolio_results['correlation_matrix'])
```

## Performance Considerations

### Caching Strategy

The analytics class includes basic caching. For production use, consider:

```python
import redis
import pickle

class CachedMangoDataAnalytics(MangoDataAnalytics):
    def __init__(self, base_url="http://localhost:3000", redis_host="localhost"):
        super().__init__(base_url)
        self.redis_client = redis.Redis(host=redis_host, decode_responses=False)
        self.cache_ttl = 3600  # 1 hour
    
    def get_historical_data(self, symbol, interval="1d", limit=252):
        cache_key = f"historical:{symbol}:{interval}:{limit}"
        
        # Try to get from cache
        cached_data = self.redis_client.get(cache_key)
        if cached_data:
            return pickle.loads(cached_data)
        
        # Fetch from API
        data = super().get_historical_data(symbol, interval, limit)
        
        # Cache the result
        if data is not None:
            self.redis_client.setex(cache_key, self.cache_ttl, pickle.dumps(data))
        
        return data
```

### Parallel Processing

For large-scale analysis, use parallel processing:

```python
from concurrent.futures import ThreadPoolExecutor, as_completed

def parallel_stock_analysis(analytics, symbols, max_workers=5):
    """
    Analyze multiple stocks in parallel.
    """
    def analyze_single_stock(symbol):
        try:
            data = analytics.get_historical_data(symbol, limit=252)
            if data is not None:
                data_with_indicators = analytics.calculate_technical_indicators(data)
                data_with_returns = analytics.calculate_returns(data_with_indicators)
                risk_metrics = analytics.calculate_risk_metrics(data_with_returns)
                return symbol, risk_metrics
        except Exception as e:
            print(f"Error analyzing {symbol}: {e}")
            return symbol, None
    
    results = {}
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_symbol = {executor.submit(analyze_single_stock, symbol): symbol 
                           for symbol in symbols}
        
        for future in as_completed(future_to_symbol):
            symbol, metrics = future.result()
            if metrics:
                results[symbol] = metrics
    
    return results

# Example usage
symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA", "AMZN", "META"]
parallel_results = parallel_stock_analysis(analytics, symbols)
```

## Error Handling and Best Practices

### Robust Data Fetching

```python
def robust_data_fetch(analytics, symbol, max_retries=3, backoff_factor=2):
    """
    Fetch data with retry logic and exponential backoff.
    """
    for attempt in range(max_retries):
        try:
            data = analytics.get_historical_data(symbol)
            if data is not None and len(data) > 0:
                return data
        except Exception as e:
            if attempt < max_retries - 1:
                wait_time = backoff_factor ** attempt
                print(f"Attempt {attempt + 1} failed for {symbol}, retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                print(f"Failed to fetch data for {symbol} after {max_retries} attempts: {e}")
    
    return None
```

### Data Validation

```python
def validate_data_quality(df, symbol):
    """
    Validate data quality and completeness.
    """
    issues = []
    
    # Check for missing data
    if df.isnull().any().any():
        issues.append("Contains null values")
    
    # Check for negative prices
    price_columns = ['open', 'high', 'low', 'close']
    for col in price_columns:
        if col in df.columns and (df[col] <= 0).any():
            issues.append(f"Contains non-positive {col} prices")
    
    # Check for logical price relationships
    if 'high' in df.columns and 'low' in df.columns:
        if (df['high'] < df['low']).any():
            issues.append("High prices less than low prices")
    
    # Check for extreme price movements (>50% in one day)
    if 'close' in df.columns:
        daily_returns = df['close'].pct_change().abs()
        if (daily_returns > 0.5).any():
            issues.append("Extreme price movements detected")
    
    if issues:
        print(f"⚠️ Data quality issues for {symbol}: {', '.join(issues)}")
    
    return len(issues) == 0
```

## Integration with Other Tools

### Jupyter Notebook Integration

```python
# For Jupyter notebooks, add these magic commands
%matplotlib inline
%load_ext autoreload
%autoreload 2

# Import and setup
import sys
sys.path.append('../')  # If running from notebooks/ subdirectory
from examples.data_analytics_example import MangoDataAnalytics

# Initialize with progress bars
from tqdm.notebook import tqdm
analytics = MangoDataAnalytics()
```

### Export to Excel

```python
def export_analysis_to_excel(analytics, symbols, filename="financial_analysis.xlsx"):
    """
    Export comprehensive analysis to Excel file.
    """
    with pd.ExcelWriter(filename, engine='openpyxl') as writer:
        # Portfolio analysis
        portfolio_results = analytics.analyze_portfolio(symbols)
        
        if portfolio_results:
            # Portfolio metrics
            portfolio_df = pd.DataFrame([portfolio_results['portfolio_metrics']]).T
            portfolio_df.columns = ['Value']
            portfolio_df.to_excel(writer, sheet_name='Portfolio_Metrics')
            
            # Correlation matrix
            if 'correlation_matrix' in portfolio_results:
                portfolio_results['correlation_matrix'].to_excel(writer, sheet_name='Correlations')
            
            # Individual stock metrics
            if 'individual_metrics' in portfolio_results:
                individual_df = pd.DataFrame(portfolio_results['individual_metrics']).T
                individual_df.to_excel(writer, sheet_name='Individual_Stocks')
        
        # Historical data for each stock
        for symbol in symbols:
            data = analytics.get_historical_data(symbol, limit=252)
            if data is not None:
                data_with_indicators = analytics.calculate_technical_indicators(data)
                data_with_returns = analytics.calculate_returns(data_with_indicators)
                data_with_returns.to_excel(writer, sheet_name=f'{symbol}_Data')
    
    print(f"Analysis exported to {filename}")
```

## Troubleshooting

### Common Issues

1. **API Connection Errors**
   ```python
   # Check if the service is running
   response = analytics.make_request("/health")
   if response:
       print("✅ API is accessible")
   else:
       print("❌ Cannot connect to API")
   ```

2. **Missing Data**
   ```python
   # Check data availability
   symbols_to_check = ["AAPL", "INVALID_SYMBOL"]
   for symbol in symbols_to_check:
       response = analytics.make_request(f"/api/symbols/{symbol}/validate")
       if response and response.get("success"):
           print(f"✅ {symbol} is valid")
       else:
           print(f"❌ {symbol} is invalid or unavailable")
   ```

3. **Memory Issues with Large Datasets**
   ```python
   # Process data in chunks
   def process_large_dataset(analytics, symbols, chunk_size=10):
       results = {}
       for i in range(0, len(symbols), chunk_size):
           chunk = symbols[i:i+chunk_size]
           chunk_results = analytics.get_bulk_historical_data(chunk, limit=100)
           results.update(chunk_results)
           time.sleep(1)  # Rate limiting
       return results
   ```

### Performance Optimization

1. **Reduce API Calls**
   - Use bulk endpoints when possible
   - Implement proper caching
   - Batch requests for multiple symbols

2. **Memory Management**
   - Process data in chunks for large datasets
   - Use appropriate data types (float32 vs float64)
   - Clean up intermediate DataFrames

3. **Computation Optimization**
   - Vectorize calculations using pandas/numpy
   - Use numba for performance-critical functions
   - Consider using Dask for very large datasets

## Next Steps

This analytics framework provides a solid foundation for financial data analysis. Consider extending it with:

- **Machine Learning Models**: Integrate scikit-learn for predictive modeling
- **Real-time Analysis**: Add streaming data capabilities
- **Advanced Visualizations**: Create interactive dashboards with Plotly/Dash
- **Risk Management**: Implement sophisticated risk models
- **Backtesting Engine**: Build comprehensive strategy testing framework
- **Portfolio Optimization**: Advanced optimization algorithms
- **Alternative Data**: Integrate news sentiment, social media data
- **Performance Attribution**: Detailed performance analysis tools

For questions or contributions, please refer to the main project documentation. 