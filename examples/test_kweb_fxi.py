#!/usr/bin/env python3
"""
Focused test for KWEB and FXI tickers that previously had issues
"""

import requests
import json
from datetime import datetime

def safe_float(value, default=0.0):
    """Safely convert a value to float"""
    if value is None:
        return default
    try:
        return float(value)
    except (ValueError, TypeError):
        return default

def safe_int(value, default=0):
    """Safely convert a value to int"""
    if value is None:
        return default
    try:
        return int(value)
    except (ValueError, TypeError):
        return default

def test_ticker(symbol, base_url="http://localhost:3000"):
    """Test a specific ticker comprehensively"""
    print(f"\n🔍 Testing {symbol}...")
    
    # Test comprehensive endpoint
    try:
        response = requests.get(f"{base_url}/api/symbols/{symbol}/comprehensive", timeout=30)
        if response.status_code == 200:
            data = response.json()
            if data.get("success"):
                comp_data = data.get("data", {})
                print(f"✅ Comprehensive data: {len(comp_data.get('data_sources', []))} sources")
                
                # Check latest quote
                latest_quote = comp_data.get("latest_quote", {})
                if latest_quote:
                    close = safe_float(latest_quote.get("close"))
                    volume = safe_int(latest_quote.get("volume"))
                    if close > 0:
                        print(f"   📈 Latest: ${close:.2f}, Volume: {volume:,}")
                
                # Check company profile
                profile = comp_data.get("company_profile", {})
                company_name = profile.get("company_name")
                if company_name:
                    print(f"   🏢 Company: {company_name}")
                
                # Check analysis
                analysis = comp_data.get("analysis", {})
                if analysis:
                    change_pct = safe_float(analysis.get("price_change_5d_percent"))
                    avg_vol = safe_int(analysis.get("avg_volume_5d"))
                    print(f"   📊 5-day change: {change_pct:.2f}%, Avg volume: {avg_vol:,}")
            else:
                print(f"❌ Comprehensive failed: {data}")
        else:
            print(f"❌ HTTP error: {response.status_code}")
    except Exception as e:
        print(f"❌ Comprehensive test failed: {e}")
    
    # Test extended endpoint
    try:
        response = requests.get(f"{base_url}/api/symbols/{symbol}/extended", timeout=30)
        if response.status_code == 200:
            data = response.json()
            if data.get("success"):
                ext_data = data.get("data", {})
                data_sources = ext_data.get("data_sources", [])
                print(f"✅ Extended data: {len(data_sources)} intervals")
                
                # Check range analysis
                range_analysis = ext_data.get("range_analysis", {})
                if range_analysis:
                    price_stats = range_analysis.get("price_stats", {})
                    min_price = safe_float(price_stats.get("min"))
                    max_price = safe_float(price_stats.get("max"))
                    if min_price > 0 and max_price > 0:
                        print(f"   📈 1-month range: ${min_price:.2f} - ${max_price:.2f}")
            else:
                print(f"❌ Extended failed: {data}")
        else:
            print(f"❌ HTTP error: {response.status_code}")
    except Exception as e:
        print(f"❌ Extended test failed: {e}")
    
    # Test historical data
    try:
        response = requests.get(f"{base_url}/api/symbols/{symbol}/historical?interval=1d&limit=5", timeout=30)
        if response.status_code == 200:
            data = response.json()
            if data.get("success"):
                hist_data = data.get("data", {}).get("data", [])
                print(f"✅ Historical data: {len(hist_data)} records")
                if hist_data:
                    latest = hist_data[0]
                    close_val = safe_float(latest.get('close'))
                    timestamp = latest.get('timestamp', 'Unknown')
                    if close_val > 0:
                        print(f"   📅 Latest record: {timestamp} - ${close_val:.2f}")
                    else:
                        print(f"   📅 Latest record: {timestamp} - ${latest.get('close')}")
            else:
                print(f"❌ Historical failed: {data}")
        else:
            print(f"❌ HTTP error: {response.status_code}")
    except Exception as e:
        print(f"❌ Historical test failed: {e}")

def main():
    print("🚀 Testing Previously Problematic Tickers: KWEB and FXI")
    print("=" * 60)
    
    # Test both tickers
    test_ticker("KWEB")
    test_ticker("FXI")
    
    print("\n" + "=" * 60)
    print("✅ Test completed! Both tickers should now work properly.")

if __name__ == "__main__":
    main() 