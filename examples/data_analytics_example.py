#!/usr/bin/env python3
"""
Mango Data Service - Data Analytics Example with Pandas
=======================================================

This example demonstrates how to use the Mango Data Service API for financial data analytics
using pandas for data manipulation and analysis. It showcases various analytical techniques
including:

- Historical price analysis
- Portfolio performance tracking
- Technical indicators calculation
- Risk analysis and volatility metrics
- Correlation analysis between stocks
- Bulk data processing
- Data visualization preparation

Requirements:
- pandas
- numpy
- requests
- matplotlib (optional, for visualization)
- seaborn (optional, for advanced visualization)

Usage:
    python data_analytics_example.py

Author: Mango Data Service Team
"""

import requests
import pandas as pd
import numpy as np
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple
import warnings
warnings.filterwarnings('ignore')

class MangoDataAnalytics:
    """
    A comprehensive data analytics class for financial data analysis using the Mango Data Service API.
    """
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        """
        Initialize the analytics client.
        
        Args:
            base_url: Base URL of the Mango Data Service API
        """
        self.base_url = base_url
        self.session = requests.Session()
        self.cache = {}
        
    def make_request(self, endpoint: str, params: Optional[Dict] = None) -> Optional[Dict]:
        """
        Make API request with error handling and caching.
        
        Args:
            endpoint: API endpoint
            params: Query parameters
            
        Returns:
            API response data or None if failed
        """
        try:
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url, params=params, timeout=30)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            print(f"❌ Request failed for {endpoint}: {e}")
            return None
    
    def get_historical_data(self, symbol: str, interval: str = "1d", limit: int = 252) -> Optional[pd.DataFrame]:
        """
        Fetch historical data and convert to pandas DataFrame.
        
        Args:
            symbol: Stock symbol
            interval: Time interval (1d, 1wk, etc.)
            limit: Number of records to fetch
            
        Returns:
            DataFrame with historical price data
        """
        response = self.make_request(f"/api/symbols/{symbol}/historical", {
            "interval": interval,
            "limit": limit
        })
        
        if not response or not response.get("success"):
            print(f"❌ Failed to fetch data for {symbol}")
            return None
            
        data = response.get("data", {}).get("data", [])
        if not data:
            print(f"❌ No historical data found for {symbol}")
            return None
            
        # Convert to DataFrame
        df = pd.DataFrame(data)
        
        # Convert timestamp to datetime
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        df.set_index('timestamp', inplace=True)
        
        # Convert price columns to numeric
        price_columns = ['open', 'high', 'low', 'close', 'adj_close', 'volume']
        for col in price_columns:
            if col in df.columns:
                df[col] = pd.to_numeric(df[col], errors='coerce')
        
        # Sort by date (oldest first)
        df.sort_index(inplace=True)
        
        return df
    
    def get_bulk_historical_data(self, symbols: List[str], interval: str = "1d", limit: int = 100) -> Dict[str, pd.DataFrame]:
        """
        Fetch historical data for multiple symbols using bulk API.
        
        Args:
            symbols: List of stock symbols
            interval: Time interval
            limit: Number of records per symbol
            
        Returns:
            Dictionary mapping symbols to DataFrames
        """
        # Split into chunks of 20 (API limit)
        symbol_chunks = [symbols[i:i+20] for i in range(0, len(symbols), 20)]
        all_data = {}
        
        for chunk in symbol_chunks:
            symbols_str = ",".join(chunk)
            response = self.make_request("/api/bulk/historical", {
                "symbols": symbols_str,
                "interval": interval,
                "limit": limit
            })
            
            if not response:
                print(f"❌ Failed to fetch bulk data for chunk: {chunk}")
                continue
            
            # Handle the actual API response format
            results = {}
            
            if isinstance(response, dict) and response.get("success") and "data" in response:
                # Standard API response format with data array
                data_array = response.get("data", [])
                if isinstance(data_array, list):
                    # Each item in data array is a symbol result
                    for symbol_result in data_array:
                        if isinstance(symbol_result, dict) and "symbol" in symbol_result:
                            symbol = symbol_result.get("symbol")
                            symbol_data = symbol_result.get("data", [])
                            symbol_success = symbol_result.get("success", True)
                            
                            if symbol and symbol_data:
                                results[symbol] = {
                                    "success": symbol_success,
                                    "data": symbol_data
                                }
                else:
                    print(f"❌ Expected data array in response for chunk: {chunk}")
                    continue
            elif isinstance(response, list):
                # Direct list format - group by symbol
                for record in response:
                    symbol = record.get("symbol")
                    if symbol:
                        if symbol not in results:
                            results[symbol] = {"success": True, "data": []}
                        results[symbol]["data"].append(record)
            else:
                print(f"❌ Unexpected response format for chunk: {chunk} - {type(response)}")
                continue
            
            for symbol, result in results.items():
                if result.get("success") and result.get("data"):
                    df = pd.DataFrame(result["data"])
                    
                    # Handle different timestamp column names
                    timestamp_col = None
                    for col in ['timestamp', 'date', 'time']:
                        if col in df.columns:
                            timestamp_col = col
                            break
                    
                    if timestamp_col:
                        df[timestamp_col] = pd.to_datetime(df[timestamp_col])
                        df.set_index(timestamp_col, inplace=True)
                    else:
                        print(f"⚠️ No timestamp column found for {symbol}, columns: {list(df.columns)}")
                        continue
                    
                    # Convert price columns to numeric
                    price_columns = ['open', 'high', 'low', 'close', 'adj_close', 'adjusted_close', 'volume']
                    for col in price_columns:
                        if col in df.columns:
                            df[col] = pd.to_numeric(df[col], errors='coerce')
                    
                    # Handle adjusted_close vs adj_close naming
                    if 'adjusted_close' in df.columns and 'adj_close' not in df.columns:
                        df['adj_close'] = df['adjusted_close']
                    
                    df.sort_index(inplace=True)
                    all_data[symbol] = df
                else:
                    print(f"❌ Failed to load data for {symbol}")
        
        return all_data
    
    def calculate_returns(self, df: pd.DataFrame, price_column: str = 'close') -> pd.DataFrame:
        """
        Calculate various return metrics.
        
        Args:
            df: DataFrame with price data
            price_column: Column to use for calculations
            
        Returns:
            DataFrame with return calculations
        """
        result_df = df.copy()
        
        # Daily returns
        result_df['daily_return'] = result_df[price_column].pct_change()
        
        # Cumulative returns
        result_df['cumulative_return'] = (1 + result_df['daily_return']).cumprod() - 1
        
        # Log returns
        result_df['log_return'] = np.log(result_df[price_column] / result_df[price_column].shift(1))
        
        # Rolling returns (5, 20, 60 days)
        for window in [5, 20, 60]:
            result_df[f'return_{window}d'] = result_df[price_column].pct_change(window)
        
        return result_df
    
    def calculate_technical_indicators(self, df: pd.DataFrame, price_column: str = 'close') -> pd.DataFrame:
        """
        Calculate common technical indicators.
        
        Args:
            df: DataFrame with price data
            price_column: Column to use for calculations
            
        Returns:
            DataFrame with technical indicators
        """
        result_df = df.copy()
        
        # Moving averages
        for window in [5, 10, 20, 50, 200]:
            result_df[f'sma_{window}'] = result_df[price_column].rolling(window=window).mean()
            result_df[f'ema_{window}'] = result_df[price_column].ewm(span=window).mean()
        
        # Bollinger Bands (20-day)
        sma_20 = result_df[price_column].rolling(window=20).mean()
        std_20 = result_df[price_column].rolling(window=20).std()
        result_df['bb_upper'] = sma_20 + (std_20 * 2)
        result_df['bb_lower'] = sma_20 - (std_20 * 2)
        result_df['bb_width'] = result_df['bb_upper'] - result_df['bb_lower']
        
        # RSI (14-day)
        delta = result_df[price_column].diff()
        gain = (delta.where(delta > 0, 0)).rolling(window=14).mean()
        loss = (-delta.where(delta < 0, 0)).rolling(window=14).mean()
        rs = gain / loss
        result_df['rsi'] = 100 - (100 / (1 + rs))
        
        # MACD
        ema_12 = result_df[price_column].ewm(span=12).mean()
        ema_26 = result_df[price_column].ewm(span=26).mean()
        result_df['macd'] = ema_12 - ema_26
        result_df['macd_signal'] = result_df['macd'].ewm(span=9).mean()
        result_df['macd_histogram'] = result_df['macd'] - result_df['macd_signal']
        
        # Volatility (20-day rolling) - calculate from price changes
        daily_returns = result_df[price_column].pct_change()
        result_df['volatility_20d'] = daily_returns.rolling(window=20).std() * np.sqrt(252)
        
        return result_df
    
    def get_backend_technical_indicators(self, symbol: str, days: int = 100) -> Optional[Dict]:
        """
        Get technical indicators from the backend API endpoint.
        
        Args:
            symbol: Stock symbol
            days: Number of days for analysis
            
        Returns:
            Technical indicators data from backend
        """
        response = self.make_request(f"/api/symbols/{symbol}/indicators", {
            "days": days
        })
        
        if not response or not response.get("success"):
            print(f"❌ Failed to fetch technical indicators for {symbol}")
            return None
            
        return response.get("data", {})

    def compare_symbols(self, symbols: List[str], interval: str = "1d") -> Optional[Dict]:
        """
        Compare multiple symbols using the backend comparison endpoint.
        
        Args:
            symbols: List of stock symbols to compare
            interval: Time interval for comparison
            
        Returns:
            Comparison data from backend
        """
        symbols_str = ",".join(symbols)
        response = self.make_request(f"/api/compare", {
            "symbols": symbols_str,
            "interval": interval
        })
        
        if not response or not response.get("success"):
            print(f"❌ Failed to compare symbols: {symbols}")
            return None
            
        return response.get("data", {})
    
    def calculate_risk_metrics(self, df: pd.DataFrame, benchmark_returns: Optional[pd.Series] = None) -> Dict[str, float]:
        """
        Calculate risk and performance metrics.
        
        Args:
            df: DataFrame with return data
            benchmark_returns: Benchmark returns for comparison
            
        Returns:
            Dictionary of risk metrics
        """
        returns = df['daily_return'].dropna()
        
        metrics = {
            'total_return': df['cumulative_return'].iloc[-1] if 'cumulative_return' in df else np.nan,
            'annualized_return': returns.mean() * 252,
            'annualized_volatility': returns.std() * np.sqrt(252),
            'sharpe_ratio': (returns.mean() * 252) / (returns.std() * np.sqrt(252)) if returns.std() > 0 else np.nan,
            'max_drawdown': self._calculate_max_drawdown(df['cumulative_return'] if 'cumulative_return' in df else returns.cumsum()),
            'var_95': np.percentile(returns, 5),
            'var_99': np.percentile(returns, 1),
            'skewness': returns.skew(),
            'kurtosis': returns.kurtosis(),
            'positive_days': (returns > 0).sum() / len(returns),
            'negative_days': (returns < 0).sum() / len(returns),
        }
        
        # Calculate beta if benchmark provided
        if benchmark_returns is not None:
            aligned_returns = returns.align(benchmark_returns, join='inner')
            if len(aligned_returns[0]) > 1:
                covariance = np.cov(aligned_returns[0], aligned_returns[1])[0][1]
                benchmark_variance = np.var(aligned_returns[1])
                metrics['beta'] = covariance / benchmark_variance if benchmark_variance > 0 else np.nan
        
        return metrics
    
    def _calculate_max_drawdown(self, cumulative_returns: pd.Series) -> float:
        """Calculate maximum drawdown."""
        peak = cumulative_returns.expanding().max()
        drawdown = (cumulative_returns - peak) / peak
        return drawdown.min()
    
    def analyze_portfolio(self, symbols: List[str], weights: Optional[List[float]] = None) -> Dict[str, Any]:
        """
        Analyze a portfolio of stocks.
        
        Args:
            symbols: List of stock symbols
            weights: Portfolio weights (equal weight if None)
            
        Returns:
            Portfolio analysis results
        """
        if weights is None:
            weights = [1.0 / len(symbols)] * len(symbols)
        
        if len(weights) != len(symbols):
            raise ValueError("Number of weights must match number of symbols")
        
        # Fetch data for all symbols
        portfolio_data = self.get_bulk_historical_data(symbols, limit=252)
        
        if not portfolio_data:
            print("❌ No data available for portfolio analysis")
            return {}
        
        # Calculate returns for each stock
        returns_data = {}
        for symbol in symbols:
            if symbol in portfolio_data:
                df_with_returns = self.calculate_returns(portfolio_data[symbol])
                returns_data[symbol] = df_with_returns['daily_return']
        
        if not returns_data:
            print("❌ No return data available")
            return {}
        
        # Create returns DataFrame
        returns_df = pd.DataFrame(returns_data)
        returns_df = returns_df.dropna()
        
        # Calculate portfolio returns
        portfolio_returns = (returns_df * weights).sum(axis=1)
        
        # Portfolio metrics
        portfolio_metrics = {
            'annualized_return': portfolio_returns.mean() * 252,
            'annualized_volatility': portfolio_returns.std() * np.sqrt(252),
            'sharpe_ratio': (portfolio_returns.mean() * 252) / (portfolio_returns.std() * np.sqrt(252)) if portfolio_returns.std() > 0 else np.nan,
            'max_drawdown': self._calculate_max_drawdown(portfolio_returns.cumsum()),
        }
        
        # Correlation matrix
        correlation_matrix = returns_df.corr()
        
        # Individual stock metrics
        individual_metrics = {}
        for symbol in symbols:
            if symbol in returns_data:
                stock_df = portfolio_data[symbol].copy()
                stock_df = self.calculate_returns(stock_df)
                individual_metrics[symbol] = self.calculate_risk_metrics(stock_df)
        
        return {
            'portfolio_metrics': portfolio_metrics,
            'correlation_matrix': correlation_matrix,
            'individual_metrics': individual_metrics,
            'weights': dict(zip(symbols, weights)),
            'returns_data': returns_df
        }
    
    def sector_analysis(self, symbols: List[str]) -> Dict[str, Any]:
        """
        Perform sector-based analysis.
        
        Args:
            symbols: List of stock symbols
            
        Returns:
            Sector analysis results
        """
        # Get company profiles to determine sectors
        sector_data = {}
        
        for symbol in symbols:
            response = self.make_request(f"/api/symbols/{symbol}/profile")
            if response and response.get("success"):
                profile = response.get("data", {})
                sector = profile.get("sector", "Unknown")
                
                if sector not in sector_data:
                    sector_data[sector] = []
                sector_data[sector].append(symbol)
        
        # Analyze each sector
        sector_analysis = {}
        
        for sector, sector_symbols in sector_data.items():
            if len(sector_symbols) > 1:
                portfolio_analysis = self.analyze_portfolio(sector_symbols)
                sector_analysis[sector] = {
                    'symbols': sector_symbols,
                    'count': len(sector_symbols),
                    'metrics': portfolio_analysis.get('portfolio_metrics', {}),
                    'avg_correlation': portfolio_analysis.get('correlation_matrix', pd.DataFrame()).mean().mean() if 'correlation_matrix' in portfolio_analysis else np.nan
                }
        
        return sector_analysis
    
    def generate_report(self, symbols: List[str], output_file: Optional[str] = None) -> str:
        """
        Generate a comprehensive analytics report.
        
        Args:
            symbols: List of stock symbols to analyze
            output_file: Optional file to save the report
            
        Returns:
            Report as string
        """
        report_lines = []
        report_lines.append("=" * 80)
        report_lines.append("MANGO DATA SERVICE - FINANCIAL ANALYTICS REPORT")
        report_lines.append("=" * 80)
        report_lines.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report_lines.append(f"Symbols Analyzed: {', '.join(symbols)}")
        report_lines.append("")
        
        # Portfolio Analysis
        portfolio_analysis = self.analyze_portfolio(symbols)
        
        if portfolio_analysis:
            report_lines.append("PORTFOLIO ANALYSIS")
            report_lines.append("-" * 40)
            
            metrics = portfolio_analysis.get('portfolio_metrics', {})
            for metric, value in metrics.items():
                if isinstance(value, float):
                    if 'return' in metric or 'volatility' in metric:
                        report_lines.append(f"{metric.replace('_', ' ').title()}: {value:.2%}")
                    else:
                        report_lines.append(f"{metric.replace('_', ' ').title()}: {value:.4f}")
                else:
                    report_lines.append(f"{metric.replace('_', ' ').title()}: {value}")
            
            report_lines.append("")
            
            # Correlation Analysis
            if 'correlation_matrix' in portfolio_analysis:
                corr_matrix = portfolio_analysis['correlation_matrix']
                report_lines.append("CORRELATION MATRIX")
                report_lines.append("-" * 40)
                report_lines.append(corr_matrix.round(3).to_string())
                report_lines.append("")
        
        # Individual Stock Analysis
        if portfolio_analysis and 'individual_metrics' in portfolio_analysis:
            report_lines.append("INDIVIDUAL STOCK ANALYSIS")
            report_lines.append("-" * 40)
            
            for symbol, metrics in portfolio_analysis['individual_metrics'].items():
                report_lines.append(f"\n{symbol}:")
                for metric, value in metrics.items():
                    if isinstance(value, float) and not np.isnan(value):
                        if 'return' in metric or 'volatility' in metric:
                            report_lines.append(f"  {metric.replace('_', ' ').title()}: {value:.2%}")
                        else:
                            report_lines.append(f"  {metric.replace('_', ' ').title()}: {value:.4f}")
        
        # Sector Analysis
        sector_analysis = self.sector_analysis(symbols)
        if sector_analysis:
            report_lines.append("\nSECTOR ANALYSIS")
            report_lines.append("-" * 40)
            
            for sector, data in sector_analysis.items():
                report_lines.append(f"\n{sector} ({data['count']} stocks):")
                report_lines.append(f"  Symbols: {', '.join(data['symbols'])}")
                
                metrics = data.get('metrics', {})
                for metric, value in metrics.items():
                    if isinstance(value, float) and not np.isnan(value):
                        if 'return' in metric or 'volatility' in metric:
                            report_lines.append(f"  {metric.replace('_', ' ').title()}: {value:.2%}")
                        else:
                            report_lines.append(f"  {metric.replace('_', ' ').title()}: {value:.4f}")
        
        report_lines.append("\n" + "=" * 80)
        report_lines.append("End of Report")
        report_lines.append("=" * 80)
        
        report = "\n".join(report_lines)
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"📄 Report saved to {output_file}")
        
        return report


def main():
    """Demonstrate comprehensive financial data analytics capabilities."""
    print("🥭 Mango Data Service - Financial Data Analytics")
    print("=" * 60)
    print("Demonstrating comprehensive analytics capabilities including:")
    print("• Historical data analysis")
    print("• Technical indicators (client-side and backend)")
    print("• Portfolio analysis and optimization")
    print("• Risk metrics and correlation analysis")
    print("• Symbol comparison")
    print("=" * 60)
    
    # Initialize analytics client
    analytics = MangoDataAnalytics()
    
    try:
        # Example 1: Single stock analysis with both methods
        print("\n1️⃣ SINGLE STOCK ANALYSIS - AAPL")
        print("-" * 40)
        
        # Get historical data
        aapl_data = analytics.get_historical_data("AAPL", limit=252)
        if aapl_data is None:
            print("❌ Could not fetch AAPL data")
            return
        
        # Method 1: Client-side technical indicators calculation
        print("\n📊 Method 1: Client-side Technical Indicators")
        aapl_with_indicators = analytics.calculate_technical_indicators(aapl_data)
        aapl_with_returns = analytics.calculate_returns(aapl_with_indicators)
            
        print("Recent AAPL data with client-side indicators:")
        columns_to_show = ['close', 'sma_20', 'rsi', 'macd']
        available_columns = [col for col in columns_to_show if col in aapl_with_indicators.columns]
        print(aapl_with_indicators[available_columns].tail().round(4))
        
        # Calculate risk metrics
        risk_metrics = analytics.calculate_risk_metrics(aapl_with_returns)
        print(f"\nAAPL Risk Metrics (Client-side):")
        for metric, value in risk_metrics.items():
            if isinstance(value, float) and not np.isnan(value):
                if 'return' in metric or 'volatility' in metric:
                    print(f"  {metric.replace('_', ' ').title()}: {value:.2%}")
                else:
                    print(f"  {metric.replace('_', ' ').title()}: {value:.4f}")
        
        # Method 2: Backend technical indicators (more robust with validation)
        print("\n🔧 Method 2: Backend Technical Indicators (Recommended)")
        backend_indicators = analytics.get_backend_technical_indicators("AAPL", days=100)
        if backend_indicators:
            indicators = backend_indicators.get("indicators", {})
            ma = indicators.get("moving_averages", {})
            momentum = indicators.get("momentum", {})
            macd = indicators.get("macd", {})
            signals = backend_indicators.get("signals", {})
            
            print(f"Backend indicators for AAPL:")
            print(f"  SMA-5: ${ma.get('sma_5', 0):.2f}")
            print(f"  SMA-20: ${ma.get('sma_20', 0):.2f}")
            print(f"  RSI: {momentum.get('rsi', 0):.1f} ({momentum.get('rsi_signal', 'N/A')})")
            print(f"  MACD: {macd.get('macd_line', 0):.4f} ({macd.get('signal', 'N/A')})")
            print(f"  Overall Trend: {signals.get('overall_trend', 'N/A')}")
            print(f"  Data points: {backend_indicators.get('data_points', 0)}")
        
        # Example 2: Portfolio analysis
        portfolio_symbols = ["AAPL", "MSFT", "GOOGL"]
        print(f"\n2️⃣ PORTFOLIO ANALYSIS: {portfolio_symbols}")
        print("-" * 40)
        
        # Method 1: Client-side portfolio analysis
        print("📊 Client-side Portfolio Analysis:")
        portfolio_results = analytics.analyze_portfolio(portfolio_symbols)
        if portfolio_results:
            portfolio_metrics = portfolio_results['portfolio_metrics']
            print(f"  Portfolio Return: {portfolio_metrics['annualized_return']:.2%}")
            print(f"  Portfolio Risk: {portfolio_metrics['annualized_volatility']:.2%}")
            print(f"  Portfolio Sharpe: {portfolio_metrics['sharpe_ratio']:.2f}")
            
            # Display correlation matrix (abbreviated)
            print("  Top correlations:")
            corr_items = []
            for symbol1, correlations in portfolio_results['correlation_matrix'].items():
                for symbol2, corr in correlations.items():
                    if symbol1 < symbol2:  # Avoid duplicates
                        corr_items.append((symbol1, symbol2, corr))
            for symbol1, symbol2, corr in sorted(corr_items, key=lambda x: abs(x[2]), reverse=True)[:3]:
                print(f"    {symbol1} vs {symbol2}: {corr:.3f}")
        
        # Method 2: Backend comparison (more efficient for large datasets)
        print(f"\n🔧 Backend Symbol Comparison:")
        comparison_data = analytics.compare_symbols(portfolio_symbols, interval="1d")
        if comparison_data:
            comp_data = comparison_data.get("comparison_data", {})
            corr_matrix = comparison_data.get("correlation_matrix", {})
            
            print("  Backend comparison results:")
            for symbol, data in comp_data.items():
                change_pct = data.get("price_change_percent", 0)
                volatility = data.get("volatility", 0)
                latest_price = data.get("latest_price", 0)
                print(f"    {symbol}: ${latest_price:.2f} ({change_pct:.2%} change, {volatility:.2%} volatility)")
            
            # Display correlation matrix from backend
            print("  Backend correlations:")
            for symbol1, correlations in corr_matrix.items():
                for symbol2, corr in correlations.items():
                    if symbol1 < symbol2 and isinstance(corr, (int, float)):  # Avoid duplicates
                        print(f"    {symbol1} vs {symbol2}: {corr:.3f}")
        
        # Example 3: Generate comprehensive report
        print(f"\n3️⃣ COMPREHENSIVE REPORT")
        print("-" * 40)
        
        report_file = analytics.generate_report(portfolio_symbols, "analytics_report.txt")
        if report_file:
            print(f"✅ Report saved to: {report_file}")
            
            # Show brief preview
            with open(report_file, 'r') as f:
                lines = f.readlines()
                print("\nReport preview (first 10 lines):")
                for line in lines[:10]:
                    print(f"  {line.strip()}")
        
        print("\n🎯 Analysis complete!")
        print("\n💡 Key Insights:")
        print("   • Backend methods are recommended for production use")
        print("   • Server-side calculations provide better error handling")
        print("   • Consistent results across all clients")
        print("   • Optimized performance with caching")
        print("\n📚 Methods Available:")
        print("   • Client-side: calculate_technical_indicators(), analyze_portfolio()")
        print("   • Backend: get_backend_technical_indicators(), compare_symbols()")
        
    except Exception as e:
        print(f"❌ Error during analysis: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main() 