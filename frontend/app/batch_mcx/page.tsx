'use client';

import React, { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import {
  ArrowLeft,
  Play,
  AlertCircle,
  Loader2,
  Clock,
  CheckCircle,
  XCircle,
  Coins,
  Filter,
  Search,
  Download,
  RefreshCw,
  Database,
  Eye,
  // TrendingUp
} from 'lucide-react';
import { 
  mcxApiClient, 
  handleMcxApiError,
} from '@/app/lib/api_mcx';
import { 
  getAlertBadgeClass, 
  formatPercentage 
} from '@/app/lib/api_nse';
import { db } from '@/app/lib/db';
import {
  McxDataWithAge,
  McxBatchAnalysisResponse,
  RulesOutput,
  Alert
} from '@/app/types/api_mcx_type';

export default function McxBatchAnalysisPage() {
  const [batchData, setBatchData] = useState<McxDataWithAge<McxBatchAnalysisResponse> | null>(null);
  const [loading, setLoading] = useState(true);
  const [newAnalysisLoading, setNewAnalysisLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterAlertType, setFilterAlertType] = useState<string>('all');
  const [searchSymbol, setSearchSymbol] = useState('');
  const [hasExistingData, setHasExistingData] = useState(false);

  // Apply MCX theme when component mounts
  useEffect(() => {
    document.body.classList.add('mcx-theme');
    
    return () => {
      document.body.classList.remove('mcx-theme');
    };
  }, []);

  const checkExistingData = useCallback(async () => {
    try {
      setLoading(true);
      const hasData = await mcxApiClient.hasBatchAnalysis();
      setHasExistingData(hasData);
      
      if (hasData) {
        // Load existing data
        loadExistingResults();
      } else {
        setLoading(false);
      }
    } catch (err) {
      setError(handleMcxApiError(err));
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    checkExistingData();
  }, [checkExistingData]);

  const loadExistingResults = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await mcxApiClient.getBatchAnalysis(false);
      
      if (response.success && response.data) {
        const dataWithAge: McxDataWithAge<McxBatchAnalysisResponse> = {
          data: response.data as McxBatchAnalysisResponse,
          age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
          lastUpdated: response.lastUpdated || Date.now(),
          fromCache: response.fromCache || false
        };
        setBatchData(dataWithAge);
      } else {
        setError(response.error || 'Failed to load existing MCX batch analysis');
      }
    } catch (err) {
      setError(handleMcxApiError(err));
    } finally {
      setLoading(false);
    }
  };

  const runNewBatchAnalysis = async () => {
    try {
      setNewAnalysisLoading(true);
      setError(null);
      
      const response = await mcxApiClient.getBatchAnalysis(true);
      
      if (response.success && response.data) {
        const dataWithAge: McxDataWithAge<McxBatchAnalysisResponse> = {
          data: response.data as McxBatchAnalysisResponse,
          age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
          lastUpdated: response.lastUpdated || Date.now(),
          fromCache: false
        };
        setBatchData(dataWithAge);
        setHasExistingData(true);
      } else {
        setError(response.error || 'Failed to run MCX batch analysis');
      }
    } catch (err) {
      setError(handleMcxApiError(err));
    } finally {
      setNewAnalysisLoading(false);
    }
  };

  const filteredResults = React.useMemo(() => {
    if (!batchData) return [];

    let filtered = batchData.data.rules_output;

    // Filter by symbol search
    if (searchSymbol) {
      filtered = filtered.filter(result =>
        result.symbol.toLowerCase().includes(searchSymbol.toLowerCase())
      );
    }

    // Filter by alert type
    if (filterAlertType !== 'all') {
      filtered = filtered.filter(result =>
        result.alerts.some(alert => alert.alert_type === filterAlertType)
      );
    }

    return filtered;
  }, [batchData, searchSymbol, filterAlertType]);

  const alertTypes = React.useMemo(() => {
    if (!batchData) return [];

    const types = new Set<string>();
    batchData.data.rules_output.forEach(result => {
      result.alerts.forEach(alert => {
        types.add(alert.alert_type);
      });
    });

    return Array.from(types);
  }, [batchData]);

  const downloadResults = () => {
    if (!batchData) return;

    const dataStr = JSON.stringify(batchData.data, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    
    const exportFileDefaultName = `mcx_batch_analysis_${new Date().toISOString().split('T')[0]}.json`;
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  return (
    <div className="min-h-screen py-8 px-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <header className="mb-8">
          <Link 
            href="/" 
            className="inline-flex items-center text-gray-400 hover:text-nse-accent transition-colors mb-4"
          >
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Home
          </Link>
          
          <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
            <div>
              <h1 className="text-4xl font-display font-bold text-gradient mb-2">
                MCX Batch Analysis
              </h1>
              <p className="text-gray-400">
                Comprehensive analysis across all commodity options with intelligent alerts
              </p>
            </div>

            <div className="flex gap-3">
              {batchData && (
                <button
                  onClick={downloadResults}
                  className="btn-primary inline-flex items-center"
                >
                  <Download className="w-4 h-4 mr-2" />
                  Download Results
                </button>
              )}
            </div>
          </div>
        </header>

        {/* Loading State for Initial Check */}
        {loading && !batchData && (
          <div className="card-glow rounded-lg p-12 text-center mb-8">
            <div className="max-w-md mx-auto">
              <Loader2 className="w-16 h-16 animate-spin mx-auto text-nse-accent mb-6" />
              <h3 className="text-xl font-semibold text-gray-100 mb-3">
                Checking for Existing MCX Data
              </h3>
              <p className="text-gray-400 mb-4">
                Looking for cached MCX batch analysis results...
              </p>
            </div>
          </div>
        )}

        {/* Error State */}
        {error && !batchData && (
          <div className="card-glow rounded-lg p-8 text-center mb-8 border border-red-900/50">
            <AlertCircle className="w-16 h-16 text-nse-error mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-gray-100 mb-2">MCX Analysis Failed</h3>
            <p className="text-gray-400 mb-6">{error}</p>
            <button onClick={() => checkExistingData()} className="btn-primary">
              Try Again
            </button>
          </div>
        )}

        {/* Action Buttons - Show when not loading initially */}
        {!loading && (
          <div className="mb-8">
            {!batchData && !newAnalysisLoading && (
              <div className="card-glow rounded-lg p-12 text-center">
                <Coins className="w-16 h-16 text-nse-accent mx-auto mb-6 glow-effect" />
                <h3 className="text-2xl font-bold text-gray-100 mb-3">
                  MCX Batch Analysis Options
                </h3>
                <p className="text-gray-400 max-w-2xl mx-auto mb-8">
                  {hasExistingData 
                    ? "You can view existing MCX batch results or run a fresh analysis across all commodity options."
                    : "Run a comprehensive batch analysis to identify unusual commodity options activity, low-price opportunities, and other trading alerts across the entire MCX commodity universe."
                  }
                </p>
                
                <div className="flex flex-col sm:flex-row gap-4 justify-center">
                  {hasExistingData && (
                    <button
                      onClick={loadExistingResults}
                      className="btn-secondary inline-flex items-center text-lg px-8 py-4"
                    >
                      <Eye className="w-5 h-5 mr-2" />
                      Show Existing MCX Results
                    </button>
                  )}
                  
                  <button
                    onClick={runNewBatchAnalysis}
                    className="btn-success inline-flex items-center text-lg px-8 py-4"
                  >
                    <Play className="w-5 h-5 mr-2" />
                    Start New MCX Batch Analysis
                  </button>
                </div>
              </div>
            )}

            {/* New Analysis Loading State */}
            {newAnalysisLoading && (
              <div className="card-glow rounded-lg p-12 text-center mb-8">
                <div className="max-w-md mx-auto">
                  <Loader2 className="w-16 h-16 animate-spin mx-auto text-nse-accent mb-6" />
                  <h3 className="text-xl font-semibold text-gray-100 mb-3">
                    Running New MCX Batch Analysis
                  </h3>
                  <p className="text-gray-400 mb-4">
                    Processing all MCX commodity options... This may take up to 2 minutes.
                  </p>
                  <div className="bg-slate-800 rounded-lg p-4">
                    <div className="flex items-center justify-between text-sm text-gray-400 mb-2">
                      <span>Progress</span>
                      <span>Analyzing commodities...</span>
                    </div>
                    <div className="w-full bg-gray-700 rounded-full h-2">
                      <div className="bg-gradient-to-r from-nse-accent to-amber-500 h-2 rounded-full animate-pulse-glow" 
                           style={{ width: '60%' }}></div>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Results */}
        {batchData && (
          <div className="space-y-8">
            {/* Data Age and Actions */}
            <div className="card-glow rounded-lg p-4">
              <div className="flex flex-col lg:flex-row items-center justify-between gap-4">
                <div className="flex items-center gap-3 text-sm">
                  <div className="flex items-center gap-2">
                    {batchData.fromCache ? (
                      <Database className="w-4 h-4 text-blue-400" />
                    ) : (
                      <Clock className="w-4 h-4 text-green-400" />
                    )}
                    <span className="text-gray-400">
                      MCX Analysis {batchData.fromCache ? 'from cache' : 'freshly completed'}
                    </span>
                  </div>
                  <span className="text-gray-500">•</span>
                  <div className="flex items-center gap-2">
                    <Clock className="w-4 h-4 text-gray-500" />
                    <span className="text-gray-400">Updated {batchData.age}</span>
                  </div>
                </div>
                
                <button
                  onClick={runNewBatchAnalysis}
                  disabled={newAnalysisLoading}
                  className="btn-success inline-flex items-center"
                >
                  {newAnalysisLoading ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <RefreshCw className="w-4 h-4 mr-2" />
                  )}
                  {newAnalysisLoading ? 'Running MCX Analysis...' : 'Start New MCX Analysis'}
                </button>
              </div>
            </div>

            {/* Summary Cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6">
              <div className="card-glow rounded-lg p-6">
                <div className="flex items-center gap-3 mb-2">
                  <Coins className="w-5 h-5 text-nse-accent" />
                  <span className="text-sm text-gray-400">Total Commodities</span>
                </div>
                <p className="text-2xl font-bold text-gray-100">
                  {batchData.data.summary.total_securities}
                </p>
              </div>

              <div className="card-glow rounded-lg p-6">
                <div className="flex items-center gap-3 mb-2">
                  <CheckCircle className="w-5 h-5 text-green-500" />
                  <span className="text-sm text-gray-400">Successful</span>
                </div>
                <p className="text-2xl font-bold text-green-400">
                  {batchData.data.summary.successful}
                </p>
              </div>

              <div className="card-glow rounded-lg p-6">
                <div className="flex items-center gap-3 mb-2">
                  <XCircle className="w-5 h-5 text-red-500" />
                  <span className="text-sm text-gray-400">Failed</span>
                </div>
                <p className="text-2xl font-bold text-red-400">
                  {batchData.data.summary.failed}
                </p>
              </div>

              <div className="card-glow rounded-lg p-6">
                <div className="flex items-center gap-3 mb-2">
                  <AlertCircle className="w-5 h-5 text-nse-warning" />
                  <span className="text-sm text-gray-400">With Alerts</span>
                </div>
                <p className="text-2xl font-bold text-yellow-400">
                  {batchData.data.summary.securities_with_alerts}
                </p>
              </div>

              <div className="card-glow rounded-lg p-6">
                <div className="flex items-center gap-3 mb-2">
                  <Clock className="w-5 h-5 text-nse-secondary" />
                  <span className="text-sm text-gray-400">Processing Time</span>
                </div>
                <p className="text-2xl font-bold text-blue-400">
                  {Math.round(batchData.data.summary.processing_time_ms / 1000)}s
                </p>
              </div>
            </div>

            {/* Filter Controls */}
            {batchData.data.rules_output.length > 0 && (
              <div className="card-glow rounded-lg p-6">
                <div className="flex flex-col lg:flex-row gap-4">
                  {/* Search */}
                  <div className="relative flex-1 max-w-md">
                    <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                    <input
                      type="text"
                      placeholder="Search commodities..."
                      value={searchSymbol}
                      onChange={(e) => setSearchSymbol(e.target.value)}
                      className="w-full pl-10 pr-4 py-3 bg-slate-800 border border-gray-600 rounded-lg 
                               focus:border-nse-accent focus:ring-2 focus:ring-nse-accent/20 
                               text-gray-100 placeholder-gray-400 transition-colors"
                    />
                  </div>

                  {/* Alert Type Filter */}
                  <div className="flex items-center gap-3">
                    <Filter className="w-5 h-5 text-gray-400" />
                    <select
                      value={filterAlertType}
                      onChange={(e) => setFilterAlertType(e.target.value)}
                      className="bg-slate-800 border border-gray-600 rounded-lg px-4 py-3 text-gray-100 
                               focus:border-nse-accent focus:ring-2 focus:ring-nse-accent/20 transition-colors"
                    >
                      <option value="all">All Alert Types</option>
                      {alertTypes.map(type => (
                        <option key={type} value={type}>
                          {type.replace(/_/g, ' ')}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                <div className="mt-4 flex items-center gap-2 text-sm text-gray-400">
                  <span>Showing {filteredResults.length} of {batchData.data.rules_output.length} commodities with alerts</span>
                </div>
              </div>
            )}

            {/* Results List */}
            {filteredResults.length > 0 ? (
              <div className="space-y-6">
                {filteredResults.map((result) => (
                  <McxCommodityResultCard key={result.symbol} result={result} />
                ))}
              </div>
            ) : batchData.data.rules_output.length === 0 ? (
              <div className="card-glow rounded-lg p-12 text-center">
                <CheckCircle className="w-16 h-16 text-green-500 mx-auto mb-4" />
                <h3 className="text-xl font-semibold text-gray-100 mb-2">
                  All Clear! 🎉
                </h3>
                <p className="text-gray-400 max-w-md mx-auto">
                  No alerts found across all {batchData.data.summary.total_securities} MCX commodities. 
                  All commodity options are trading within normal parameters.
                </p>
              </div>
            ) : (
              <div className="card-glow rounded-lg p-8 text-center">
                <Search className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-300 mb-2">No Results Found</h3>
                <p className="text-gray-500">
                  Try adjusting your search or filter criteria
                </p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// MCX Commodity Result Card Component - Updated to use query parameters
function McxCommodityResultCard({ result }: { result: RulesOutput }) {
  return (
    <div className="card-glow rounded-lg overflow-hidden">
      <div className="p-6 border-b border-gray-700/50">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div>
            <div className="flex items-center gap-3 mb-2">
              <h3 className="text-xl font-bold text-gradient flex items-center">
                <span className="mr-2">📈</span>
                {result.symbol}
              </h3>
              <span className="px-3 py-1 bg-nse-accent/20 text-nse-accent rounded-full text-sm font-medium">
                {result.alerts.length} Alert{result.alerts.length !== 1 ? 's' : ''}
              </span>
            </div>
            <div className="flex items-center gap-4 text-sm text-gray-400">
              <span>Underlying: ₹{result.underlyingValue.toFixed(2)}</span>
              <span>•</span>
              
              <span>Updated: {new Date(result.timestamp).toLocaleString()}</span>
            </div>
          </div>
          
          <Link 
            href={`/commodity/?symbol=${encodeURIComponent(result.symbol)}`}
            className="btn-primary text-sm"
          >
            View Details
          </Link>
        </div>
      </div>

      <div className="p-6">
        <div className="space-y-4">
          {result.alerts.map((alert, index) => (
            <McxAlertCard key={index} alert={alert} />
          ))}
        </div>
      </div>
    </div>
  );
}

// MCX Alert Card Component
function McxAlertCard({ alert }: { alert: Alert }) {
  return (
    <div className="bg-slate-800/50 rounded-lg p-4 border border-gray-700/50">
      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-3">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <span className={getAlertBadgeClass(alert.alert_type)}>
              {alert.alert_type.replace(/_/g, ' ')}
            </span>
            <span className={`font-medium ${
                (alert.option_type === 'CE' && alert.alert_type === 'HUGE_OI_INCREASE') ||
                (alert.option_type === 'PE' && alert.alert_type === 'HUGE_OI_DECREASE')
                ? 'text-red-400': 'text-green-400'}`}>
              {alert.option_type}
            </span>
            <span className="text-gray-400">
              Strike: ₹{alert.strikePrice}
            </span>
            <span className="text-gray-400">
              Expiry: {alert.expiryDates}
            </span>
          </div>
          
          <p className="text-gray-300 text-sm">{alert.description}</p>
        </div>

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 text-sm lg:min-w-[400px]">
          <div>
            <span className="text-gray-500">Time Value:</span>
            <p className="text-gray-200 font-medium">₹{alert.values.time_val.toFixed(2)}</p>
          </div>
          <div>
            <span className="text-gray-500">Position:</span>
            <p className="text-gray-200 font-medium">{alert.values.the_money}</p>
          </div>
          {alert.values.lastPrice && (
            <div>
              <span className="text-gray-500">Last Price:</span>
              <p className="text-gray-200 font-medium">₹{alert.values.lastPrice.toFixed(2)}</p>
            </div>
          )}
          {alert.values.pchangeinOpenInterest && (
            <div>
              <span className="text-gray-500">OI Change:</span>
              <p className={`font-medium ${
                alert.values.pchangeinOpenInterest > 0 ? 'text-green-400' : 'text-red-400'
              }`}>
                {formatPercentage(alert.values.pchangeinOpenInterest)}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}