#!/usr/bin/env python3
"""
Comprehensive Yahoo Finance Data Service API Test Suite
Tests all endpoints and validates data quality and completeness.
"""

import requests
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
import sys

class YahooFinanceAPITester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.session = requests.Session()
        self.test_symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA", "AMZN"]
        self.results = {
            "passed": 0,
            "failed": 0,
            "warnings": 0,
            "tests": []
        }

    def log_test(self, test_name: str, status: str, message: str, data: Optional[Dict] = None):
        """Log test results"""
        result = {
            "test": test_name,
            "status": status,
            "message": message,
            "timestamp": datetime.now().isoformat(),
            "data": data
        }
        self.results["tests"].append(result)
        
        if status == "PASS":
            self.results["passed"] += 1
            print(f"✅ {test_name}: {message}")
        elif status == "FAIL":
            self.results["failed"] += 1
            print(f"❌ {test_name}: {message}")
        elif status == "WARN":
            self.results["warnings"] += 1
            print(f"⚠️  {test_name}: {message}")
        
        if data:
            print(f"   📊 Data sample: {json.dumps(data, indent=2)[:200]}...")

    def make_request(self, endpoint: str) -> Optional[Dict]:
        """Make API request with error handling"""
        try:
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url, timeout=30)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            print(f"❌ Request failed for {endpoint}: {e}")
            return None

    def test_health_check(self):
        """Test basic health check"""
        print("\n🔍 Testing Health Check...")
        response = self.make_request("/health")
        
        if not response:
            self.log_test("Health Check", "FAIL", "No response from health endpoint")
            return
            
        if response.get("success") and response.get("data", {}).get("status") == "healthy":
            version = response.get("data", {}).get("version", "unknown")
            self.log_test("Health Check", "PASS", f"Server healthy, version {version}")
        else:
            self.log_test("Health Check", "FAIL", "Health check failed", response)

    def test_comprehensive_quote_data(self):
        """Test our improved comprehensive quote data extraction"""
        print("\n🔍 Testing Comprehensive Quote Data...")
        
        for symbol in self.test_symbols[:3]:  # Test first 3 symbols
            response = self.make_request(f"/api/symbols/{symbol}/comprehensive")
            
            if not response or not response.get("success"):
                self.log_test(f"Comprehensive Quote - {symbol}", "FAIL", "No response or failed", response)
                continue
                
            data = response.get("data", {})
            data_sources = data.get("data_sources", [])
            
            # Check if we have multiple data sources
            if len(data_sources) >= 2:
                self.log_test(f"Comprehensive Quote - {symbol}", "PASS", 
                            f"Got {len(data_sources)} data sources: {data_sources}")
            else:
                self.log_test(f"Comprehensive Quote - {symbol}", "WARN", 
                            f"Limited data sources: {data_sources}")
            
            # Check for specific data components
            expected_components = ["latest_quote", "metadata", "analysis"]
            missing_components = [comp for comp in expected_components if comp not in data]
            
            if not missing_components:
                # Validate latest quote data
                latest_quote = data.get("latest_quote", {})
                required_fields = ["timestamp", "open", "high", "low", "close", "volume"]
                missing_fields = [field for field in required_fields if field not in latest_quote]
                
                if not missing_fields:
                    self.log_test(f"Quote Data Quality - {symbol}", "PASS", 
                                f"All required quote fields present: {required_fields}")
                else:
                    self.log_test(f"Quote Data Quality - {symbol}", "WARN", 
                                f"Missing quote fields: {missing_fields}")
                
                # Validate analysis data
                analysis = data.get("analysis", {})
                if "price_change_5d_percent" in analysis and "avg_volume_5d" in analysis:
                    change_pct = analysis.get("price_change_5d_percent")
                    avg_volume = analysis.get("avg_volume_5d")
                    self.log_test(f"Analysis Data - {symbol}", "PASS", 
                                f"5-day change: {change_pct}%, avg volume: {avg_volume:,}")
                else:
                    self.log_test(f"Analysis Data - {symbol}", "WARN", 
                                "Missing analysis components")
            else:
                self.log_test(f"Comprehensive Quote - {symbol}", "WARN", 
                            f"Missing components: {missing_components}")

    def test_extended_quote_data(self):
        """Test our improved extended quote data with multiple intervals"""
        print("\n🔍 Testing Extended Quote Data...")
        
        for symbol in self.test_symbols[:2]:  # Test first 2 symbols
            response = self.make_request(f"/api/symbols/{symbol}/extended")
            
            if not response or not response.get("success"):
                self.log_test(f"Extended Quote - {symbol}", "FAIL", "No response or failed", response)
                continue
                
            data = response.get("data", {})
            data_sources = data.get("data_sources", [])
            
            # Check for multiple interval data
            interval_data = [key for key in data.keys() if key.startswith("quotes_")]
            
            if len(interval_data) >= 2:
                self.log_test(f"Extended Quote - {symbol}", "PASS", 
                            f"Got data for intervals: {interval_data}")
                
                # Check range analysis
                if "range_analysis" in data:
                    range_analysis = data["range_analysis"]
                    price_stats = range_analysis.get("price_stats", {})
                    if "min" in price_stats and "max" in price_stats and "avg" in price_stats:
                        min_price = price_stats["min"]
                        max_price = price_stats["max"]
                        avg_price = price_stats["avg"]
                        range_pct = price_stats.get("range_percent", 0)
                        self.log_test(f"Range Analysis - {symbol}", "PASS", 
                                    f"Price range: ${min_price:.2f}-${max_price:.2f} (avg: ${avg_price:.2f}, range: {range_pct:.1f}%)")
                    else:
                        self.log_test(f"Range Analysis - {symbol}", "WARN", "Incomplete price statistics")
                else:
                    self.log_test(f"Extended Quote - {symbol}", "WARN", "No range analysis data")
            else:
                self.log_test(f"Extended Quote - {symbol}", "WARN", 
                            f"Limited interval data: {interval_data}")

    def test_historical_data_quality(self):
        """Test historical data quality and completeness"""
        print("\n🔍 Testing Historical Data Quality...")
        
        for symbol in self.test_symbols[:2]:
            # Test different intervals
            intervals = ["1d", "1wk"]
            
            for interval in intervals:
                response = self.make_request(f"/api/symbols/{symbol}/historical?interval={interval}&limit=30")
                
                if not response or not response.get("success"):
                    self.log_test(f"Historical Data - {symbol} ({interval})", "FAIL", "No response or failed")
                    continue
                    
                data = response.get("data", {})
                historical_data = data.get("data", [])
                
                if len(historical_data) >= 10:
                    # Check data quality
                    latest = historical_data[0]
                    required_fields = ["timestamp", "open", "high", "low", "close", "volume"]
                    missing_fields = [field for field in required_fields if field not in latest]
                    
                    if not missing_fields:
                        # Validate data consistency
                        valid_data = True
                        for record in historical_data[:5]:  # Check first 5 records
                            try:
                                high = float(record["high"]) if isinstance(record["high"], str) else record["high"]
                                low = float(record["low"]) if isinstance(record["low"], str) else record["low"]
                                close = float(record["close"]) if isinstance(record["close"], str) else record["close"]
                                volume = int(record["volume"]) if isinstance(record["volume"], str) else record["volume"]
                                
                                if (high < low or close < 0 or volume < 0):
                                    valid_data = False
                                    break
                            except (ValueError, TypeError):
                                valid_data = False
                                break
                        
                        if valid_data:
                            self.log_test(f"Historical Data - {symbol} ({interval})", "PASS", 
                                        f"Got {len(historical_data)} valid records, latest close: ${latest['close']}")
                        else:
                            self.log_test(f"Historical Data - {symbol} ({interval})", "WARN", 
                                        "Data validation failed - inconsistent values")
                    else:
                        self.log_test(f"Historical Data - {symbol} ({interval})", "WARN", 
                                    f"Missing fields: {missing_fields}")
                else:
                    self.log_test(f"Historical Data - {symbol} ({interval})", "WARN", 
                                f"Insufficient data: {len(historical_data)} records")

    def test_company_profiles(self):
        """Test company profile data extraction"""
        print("\n🔍 Testing Company Profiles...")
        
        for symbol in self.test_symbols[:3]:
            response = self.make_request(f"/api/symbols/{symbol}/profile")
            
            if not response or not response.get("success"):
                self.log_test(f"Company Profile - {symbol}", "FAIL", "No response or failed")
                continue
                
            data = response.get("data", {})
            profile = data.get("profile")
            
            if profile:
                company_name = profile.get("company_name")
                if company_name:
                    self.log_test(f"Company Profile - {symbol}", "PASS", 
                                f"Company: {company_name}")
                else:
                    self.log_test(f"Company Profile - {symbol}", "WARN", 
                                "Profile exists but missing company name")
            else:
                self.log_test(f"Company Profile - {symbol}", "WARN", 
                            "No profile data available")

    def test_bulk_operations(self):
        """Test bulk data fetching"""
        print("\n🔍 Testing Bulk Operations...")
        
        symbols_str = ",".join(self.test_symbols[:3])
        response = self.make_request(f"/api/bulk/historical?symbols={symbols_str}&interval=1d")
        
        if not response or not response.get("success"):
            self.log_test("Bulk Operations", "FAIL", "Bulk fetch failed")
            return
            
        data = response.get("data", [])
        successful_fetches = [item for item in data if item.get("success")]
        
        if len(successful_fetches) >= 2:
            total_records = sum(item.get("count", 0) for item in successful_fetches)
            self.log_test("Bulk Operations", "PASS", 
                        f"Successfully fetched {len(successful_fetches)} symbols, {total_records} total records")
        else:
            self.log_test("Bulk Operations", "WARN", 
                        f"Limited success: {len(successful_fetches)} out of {len(self.test_symbols[:3])} symbols")

    def test_price_analysis(self):
        """Test price analysis calculations"""
        print("\n🔍 Testing Price Analysis...")
        
        for symbol in self.test_symbols[:2]:
            response = self.make_request(f"/api/symbols/{symbol}/analysis?days=30")
            
            if not response or not response.get("success"):
                self.log_test(f"Price Analysis - {symbol}", "FAIL", "No response or failed")
                continue
                
            data = response.get("data", {})
            
            required_metrics = ["min_price", "max_price", "avg_price", "volatility", "price_change_percent"]
            missing_metrics = [metric for metric in required_metrics if metric not in data]
            
            if not missing_metrics:
                volatility = float(data.get("volatility", 0))
                change_pct = float(data.get("price_change_percent", 0))
                data_points = data.get("data_points", 0)
                
                self.log_test(f"Price Analysis - {symbol}", "PASS", 
                            f"30-day analysis: {change_pct:.2f}% change, {volatility:.2f} volatility, {data_points} data points")
            else:
                self.log_test(f"Price Analysis - {symbol}", "WARN", 
                            f"Missing metrics: {missing_metrics}")

    def test_database_stats(self):
        """Test database statistics"""
        print("\n🔍 Testing Database Statistics...")
        
        response = self.make_request("/api/stats")
        
        if not response or not response.get("success"):
            self.log_test("Database Stats", "FAIL", "No response or failed")
            return
            
        data = response.get("data", {})
        
        if "symbols_count" in data and "historical_records_count" in data:
            symbols_count = data.get("symbols_count", 0)
            records_count = data.get("historical_records_count", 0)
            profiles_count = data.get("company_profiles_count", 0)
            
            self.log_test("Database Stats", "PASS", 
                        f"Database contains: {symbols_count} symbols, {records_count} historical records, {profiles_count} profiles")
        else:
            self.log_test("Database Stats", "WARN", "Incomplete database statistics")

    def test_technical_indicators(self):
        """Test technical indicators endpoint"""
        print("\n🔍 Testing Technical Indicators...")
        
        for symbol in self.test_symbols[:2]:  # Test first 2 symbols
            response = self.make_request(f"/api/symbols/{symbol}/indicators?days=50")
            
            if not response or not response.get("success"):
                self.log_test(f"Technical Indicators - {symbol}", "FAIL", "No response or failed", response)
                continue
                
            data = response.get("data", {})
            indicators = data.get("indicators", {})
            
            # Check for required indicator categories
            required_categories = ["moving_averages", "momentum", "macd", "bollinger_bands"]
            missing_categories = [cat for cat in required_categories if cat not in indicators]
            
            if not missing_categories:
                # Check specific indicators
                ma = indicators.get("moving_averages", {})
                momentum = indicators.get("momentum", {})
                macd = indicators.get("macd", {})
                
                rsi = momentum.get("rsi", 0)
                sma_20 = ma.get("sma_20", 0)
                macd_line = macd.get("macd_line", 0)
                
                self.log_test(f"Technical Indicators - {symbol}", "PASS", 
                            f"RSI: {rsi:.1f}, SMA-20: {sma_20:.2f}, MACD: {macd_line:.4f}")
            else:
                self.log_test(f"Technical Indicators - {symbol}", "FAIL", 
                            f"Missing indicator categories: {missing_categories}")

    def test_symbol_comparison(self):
        """Test symbol comparison endpoint"""
        print("\n🔍 Testing Symbol Comparison...")
        
        symbols = "AAPL,MSFT,GOOGL"
        response = self.make_request(f"/api/compare?symbols={symbols}&interval=1d")
        
        if not response or not response.get("success"):
            self.log_test("Symbol Comparison", "FAIL", "No response or failed", response)
            return
            
        data = response.get("data", {})
        comparison_data = data.get("comparison_data", {})
        correlation_matrix = data.get("correlation_matrix", {})
        
        if len(comparison_data) >= 2 and len(correlation_matrix) >= 2:
            symbols_compared = list(comparison_data.keys())
            correlations = len([k for k in correlation_matrix.keys() if correlation_matrix[k]])
            
            self.log_test("Symbol Comparison", "PASS", 
                        f"Compared {len(symbols_compared)} symbols, {correlations} correlation pairs calculated")
        else:
            self.log_test("Symbol Comparison", "FAIL", 
                        f"Insufficient comparison data: {len(comparison_data)} symbols, {len(correlation_matrix)} correlations")

    def run_all_tests(self):
        """Run comprehensive test suite"""
        print("🚀 Starting Comprehensive Yahoo Finance API Test Suite")
        print(f"📅 Test started at: {datetime.now().isoformat()}")
        print(f"🎯 Testing symbols: {', '.join(self.test_symbols)}")
        print("=" * 80)
        
        start_time = time.time()
        
        # Run all tests
        self.test_health_check()
        self.test_comprehensive_quote_data()
        self.test_extended_quote_data()
        self.test_historical_data_quality()
        self.test_company_profiles()
        self.test_bulk_operations()
        self.test_price_analysis()
        self.test_database_stats()
        self.test_technical_indicators()
        self.test_symbol_comparison()
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Print summary
        print("\n" + "=" * 80)
        print("📊 TEST SUMMARY")
        print("=" * 80)
        print(f"✅ Passed: {self.results['passed']}")
        print(f"❌ Failed: {self.results['failed']}")
        print(f"⚠️  Warnings: {self.results['warnings']}")
        print(f"⏱️  Duration: {duration:.2f} seconds")
        print(f"📈 Success Rate: {(self.results['passed'] / (self.results['passed'] + self.results['failed']) * 100):.1f}%")
        
        if self.results['failed'] > 0:
            print("\n❌ FAILED TESTS:")
            for test in self.results['tests']:
                if test['status'] == 'FAIL':
                    print(f"   - {test['test']}: {test['message']}")
        
        if self.results['warnings'] > 0:
            print("\n⚠️  WARNINGS:")
            for test in self.results['tests']:
                if test['status'] == 'WARN':
                    print(f"   - {test['test']}: {test['message']}")
        
        # Save detailed results
        with open('test_results.json', 'w') as f:
            json.dump(self.results, f, indent=2, default=str)
        print(f"\n💾 Detailed results saved to: test_results.json")
        
        return self.results['failed'] == 0

def main():
    """Main test runner"""
    if len(sys.argv) > 1:
        base_url = sys.argv[1]
    else:
        base_url = "http://localhost:3000"
    
    tester = YahooFinanceAPITester(base_url)
    success = tester.run_all_tests()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main() 