#!/usr/bin/env python3
"""
Mango Data Service - Simple Analytics Demo
==========================================

A concise demonstration of the data analytics capabilities with minimal output.

Usage:
    python simple_analytics_demo.py
"""

import pandas as pd
import numpy as np
from data_analytics_example import MangoDataAnalytics

def main():
    """Simple analytics demonstration with concise output."""
    print("🥭 Mango Data Service - Simple Analytics Demo")
    print("=" * 50)
    
    # Initialize analytics client
    analytics = MangoDataAnalytics()
    
    # Test with a single stock
    symbol = "AAPL"
    print(f"\n📊 Analyzing {symbol}...")
    
    try:
        # Get historical data
        data = analytics.get_historical_data(symbol, limit=50)
        if data is None:
            print(f"❌ Could not fetch data for {symbol}")
            return
        
        print(f"\n📊 Comparing Client-side vs Backend calculations for {symbol}:")
        
        # Method 1: Client-side calculation
        print("\n🔧 Client-side Technical Indicators:")
        data_with_indicators = analytics.calculate_technical_indicators(data)
        data_with_returns = analytics.calculate_returns(data_with_indicators)
        risk_metrics = analytics.calculate_risk_metrics(data_with_returns)
        
        print(f"   Latest Price: ${data['close'].iloc[-1]:.2f}")
        print(f"   20-day SMA: ${data_with_indicators['sma_20'].iloc[-1]:.2f}")
        print(f"   RSI: {data_with_indicators['rsi'].iloc[-1]:.1f}")
        print(f"   Annualized Return: {risk_metrics['annualized_return']:.1%}")
        print(f"   Volatility: {risk_metrics['annualized_volatility']:.1%}")
        print(f"   Sharpe Ratio: {risk_metrics['sharpe_ratio']:.2f}")
        
        # Method 2: Backend calculation (recommended)
        print("\n🚀 Backend Technical Indicators (Recommended):")
        backend_indicators = analytics.get_backend_technical_indicators(symbol, days=50)
        if backend_indicators:
            indicators = backend_indicators.get("indicators", {})
            ma = indicators.get("moving_averages", {})
            momentum = indicators.get("momentum", {})
            signals = backend_indicators.get("signals", {})
            
            print(f"   20-day SMA: ${ma.get('sma_20', 0):.2f}")
            print(f"   RSI: {momentum.get('rsi', 0):.1f} ({momentum.get('rsi_signal', 'N/A')})")
            print(f"   Overall Trend: {signals.get('overall_trend', 'N/A')}")
            print(f"   Trend Strength: {signals.get('strength', 'N/A')}")
            print(f"   Data Points Used: {backend_indicators.get('data_points', 0)}")
        else:
            print("   ❌ Backend indicators not available")
            
        print(f"\n💡 Backend advantages: Better error handling, caching, and consistency")
        
        # Portfolio analysis with 3 stocks
        print(f"\n📊 Portfolio Analysis...")
        portfolio_symbols = ["AAPL", "MSFT", "GOOGL"]
        
        # Method 1: Client-side portfolio analysis
        print(f"\n🔧 Client-side Portfolio Analysis:")
        portfolio_data = {}
        for sym in portfolio_symbols:
            stock_data = analytics.get_historical_data(sym, limit=30)
            if stock_data is not None:
                portfolio_data[sym] = stock_data
        
        if len(portfolio_data) >= 2:
            # Calculate correlations manually
            returns_data = {}
            for sym, df in portfolio_data.items():
                returns_data[sym] = df['close'].pct_change().dropna()
            
            returns_df = pd.DataFrame(returns_data).dropna()
            correlation_matrix = returns_df.corr()
            
            print(f"   Stocks analyzed: {list(portfolio_data.keys())}")
            print(f"   Average correlation: {correlation_matrix.values[np.triu_indices_from(correlation_matrix.values, k=1)].mean():.3f}")
            
            # Show correlation pairs
            print(f"   Key Correlations:")
            for i in range(len(correlation_matrix.columns)):
                for j in range(i+1, len(correlation_matrix.columns)):
                    corr = correlation_matrix.iloc[i, j]
                    stock1 = correlation_matrix.columns[i]
                    stock2 = correlation_matrix.columns[j]
                    print(f"     {stock1}-{stock2}: {corr:.3f}")
        
        # Method 2: Backend comparison (recommended)
        print(f"\n🚀 Backend Portfolio Comparison (Recommended):")
        comparison_data = analytics.compare_symbols(portfolio_symbols, interval="1d")
        if comparison_data:
            comp_data = comparison_data.get("comparison_data", {})
            corr_matrix = comparison_data.get("correlation_matrix", {})
            
            print(f"   Backend comparison results:")
            for symbol, data in comp_data.items():
                change_pct = data.get("price_change_percent", 0)
                volatility = data.get("volatility", 0)
                print(f"     {symbol}: {change_pct:.2%} change, {volatility:.2%} volatility")
            
            # Show backend correlations
            if corr_matrix:
                print(f"   Backend Correlations:")
                for symbol1, correlations in corr_matrix.items():
                    for symbol2, corr in correlations.items():
                        if symbol1 < symbol2 and isinstance(corr, (int, float)):
                            print(f"     {symbol1}-{symbol2}: {corr:.3f}")
        else:
            print("   ❌ Backend comparison not available")
        
        print(f"\n✅ Analysis completed successfully!")
        print(f"\n💡 For detailed analysis, run: python data_analytics_example.py")
        print(f"📄 For interactive analysis, use: jupyter notebook data_analytics_notebook.ipynb")
        
    except Exception as e:
        print(f"❌ Error during analysis: {e}")

if __name__ == "__main__":
    main() 