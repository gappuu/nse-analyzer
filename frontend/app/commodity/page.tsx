'use client';

import React, { useState, useEffect, Suspense } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import {
  ArrowLeft,
  Calendar,
  TrendingUp,
  AlertCircle,
  Loader2,
  Clock,
  Coins,
  RefreshCw,
  Database,
  Layers,
  BarChart
} from 'lucide-react';
import { 
  mcxApiClient, 
  handleMcxApiError, 
  getMcxCommodityIcon,
} from '@/app/lib/api_mcx';
import { 
  getAlertBadgeClass, 
  formatCurrency, 
  getMoneyStatusColor, 
  // get20DaysAgo, 
  // getToday 
} from '@/app/lib/api_nse';
import { db } from '@/app/lib/db';
import {
  McxDataWithAge,
  // McxTickersResponse,
  // McxFutureSymbolsResponse,
  McxOptionChainResponse,
  McxFutureQuoteResponse,
  McxFutureAnalysis,
  CombinedCommodityData,
  // ProcessedOptionData,
  // ProcessedOptionDetail,
  // Alert
} from '@/app/types/api_mcx_type';

// Separate component that uses useSearchParams - wrapped in Suspense
function CommodityPageContent() {
  const searchParams = useSearchParams();
  const symbol = searchParams.get('symbol');
  
  const [commodityData, setCommodityData] = useState<McxDataWithAge<CombinedCommodityData> | null>(null);
  const [selectedExpiry, setSelectedExpiry] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<'options' | 'futures'>('options');
  const [analysisData, setAnalysisData] = useState<McxDataWithAge<McxOptionChainResponse> | null>(null);
  const [futuresQuote, setFuturesQuote] = useState<McxFutureAnalysis | null>(null);
  const [latestFuturesData, setLatestFuturesData] = useState<McxFutureAnalysis | null>(null);
  const [futuresData, setFuturesData] = useState<McxDataWithAge<McxFutureQuoteResponse> | null>(null);
  const [loading, setLoading] = useState(true);
  const [analysisLoading, setAnalysisLoading] = useState(false);
  const [fetching, setFetching] = useState(false);
  const [analysisFetching, setAnalysisFetching] = useState(false);
  const [futuresLoading, setFuturesLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentTime, setCurrentTime] = useState(Date.now());

  // Apply MCX theme when component mounts
  useEffect(() => {
    document.body.classList.add('mcx-theme');
    
    return () => {
      document.body.classList.remove('mcx-theme');
    };
  }, []);

  // Real-time duration update effect
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(Date.now());
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  // Function to calculate real-time age from lastUpdated timestamp
  const getRealTimeAge = (lastUpdated: number) => {
    return db.getDataAge(lastUpdated, currentTime);
  };

  const getDataTimestampAge = (timestampString?: string) => {
    try {
      if (!timestampString) return 'unknown';
      const dataTime = new Date(timestampString).getTime();
      return db.getDataAge(dataTime, currentTime);
    } catch {
      return 'unknown';
    }
  };

  // Function to format expiry date for API (DD-MMM-YYYY to DDMMMYYYY)
  const formatExpiryForAPI = (expiryDate: string): string => {
    try {
      // Parse different date formats and convert to DDMMMYYYY
      const date = new Date(expiryDate);
      const day = String(date.getDate()).padStart(2, '0');
      const month = date.toLocaleString('en-US', { month: 'short' }).toUpperCase();
      const year = date.getFullYear();
      return `${day}${month}${year}`;
    } catch (error) {
      console.error('Error formatting expiry date:', error);
      return expiryDate; // Return as-is if formatting fails
    }
  };

  // Function to fetch futures quote with new MCX API structure
  const fetchFuturesQuote = async (expiry: string, forceRefresh = false) => {
    if (!symbol || !expiry) return;
    
    try {
      setFuturesLoading(true);
      
      // Format expiry for API (DDMMMYYYY)
      const formattedExpiry = formatExpiryForAPI(expiry);
      
      const response = await mcxApiClient.getFutureQuote(symbol, formattedExpiry, forceRefresh);
      
      if (response.success && response.data) {
        // The response.data is the content from the MCX API
        if (response.data.d && response.data.d.Data && response.data.d.Data.length > 0) {
          const data = response.data.d.Data[0];
          const summary = response.data.d.Summary;

          const {
            PercentChange: pchange,
            ChangeInOpenInterest: changeInOI,
            OpenInterest: openInterest,
            PreviousClose: previousClose,
            AbsoluteChange: absoluteChange,
            ExpiryDate: expiryDate,
            Productdesc: Productdesc,
            LifeTimeHigh: LifeTimeHigh,
            AveragePrice: AveragePrice,
            LifeTimeLow: LifeTimeLow,
            LTP: LTP,
            pchangeinOpenInterest: pchangeinOpenInterest,
            TradingUnit: TradingUnit,
            Category: Category,
            action:action
          } = data;  
          
          const futureAnalysis: McxFutureAnalysis = {
            action,
            underlyingValue: previousClose, 
            timestamp:summary.AsOn,
            lastPrice: previousClose + absoluteChange, 
            openInterest,
            changeinOpenInterest: changeInOI,
            expiryDate,
            percentChange: pchange,
            absoluteChange,
            previousClose,
            pchangeinOpenInterest,
            Productdesc,
            LifeTimeHigh,
            AveragePrice,
            LifeTimeLow,
            LTP,
            TradingUnit,
            Category
          };
          
          setFuturesQuote(futureAnalysis);
          
          const dataWithAge: McxDataWithAge<McxFutureQuoteResponse> = {
            data: response.data,
            age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
            lastUpdated: response.lastUpdated || Date.now(),
            fromCache: response.fromCache || false
          };
          setFuturesData(dataWithAge);
        } else {
          console.error('Invalid futures data structure:', response.data);
          setFuturesQuote({
            action: 'No Data',
            underlyingValue: 0,
            timestamp: '',
            lastPrice: 0,
            openInterest: 0,
            changeinOpenInterest: 0
          });
          setFuturesData(null);
        }
      } else {
        console.error('No futures data available or API error:', response.error);
        setFuturesQuote({
          action: 'No Data',
          underlyingValue: 0,
          timestamp: '',
          lastPrice: 0,
          openInterest: 0,
          changeinOpenInterest: 0
        });
        setFuturesData(null);
      }
    } catch (error) {
      console.error('Error fetching futures quote:', error);
      setFuturesQuote({
        action: 'Error',
        underlyingValue: 0,
        timestamp: '',
        lastPrice: 0,
        openInterest: 0,
        changeinOpenInterest: 0
      });
      setFuturesData(null);
    } finally {
      setFuturesLoading(false);
    }
  };

  // Function to fetch latest futures data for header display
  const fetchLatestFuturesData = async () => {
    if (!symbol || !commodityData?.data.futureExpiries.length) return;
    
    try {
      // Get the latest (first) futures expiry
      const latestExpiry = commodityData.data.futureExpiries[0];
      const formattedExpiry = formatExpiryForAPI(latestExpiry);
      
      const response = await mcxApiClient.getFutureQuote(symbol, formattedExpiry, false);
      
      if (response.success && response.data) {
        if (response.data.d && response.data.d.Data && response.data.d.Data.length > 0) {
          const data = response.data.d.Data[0];
          const summary = response.data.d.Summary;
          
          const {
            PercentChange: pchange,
            ChangeInOpenInterest: changeInOI,
            OpenInterest: openInterest,
            PreviousClose: previousClose,
            AbsoluteChange: absoluteChange,
            ExpiryDate: expiryDate
          } = data;
          
          setLatestFuturesData({
            underlyingValue: previousClose,
            timestamp: summary.AsOn,
            lastPrice: previousClose + absoluteChange,
            openInterest,
            changeinOpenInterest: changeInOI,
            expiryDate,
            percentChange: pchange,
            absoluteChange,
            previousClose
          });
        }
      }
    } catch (error) {
      console.error('Error fetching latest futures data:', error);
    }
  };

  // Function to get OI rank styling
  const getOIRankStyling = (oiRank?: number) => {
    if (!oiRank || ![1, 2, 3].includes(oiRank)) {
      return { className: '', showRank: false };
    }
    
    let className = 'relative ';
    switch (oiRank) {
      case 1:
        className += 'bg-yellow-500/20  border-yellow-500 font-semibold';
        break;
      case 2:
        className += 'bg-yellow-500/20  border-orange-500 font-semibold';
        break;
      case 3:
        className += 'bg-yellow-500/20 border-blue-500 font-semibold';
        break;
    }
    
    return { className, showRank: true };
  };

  useEffect(() => {
    const loadCommodityData = async () => {
      if (!symbol) {
        setError('No symbol provided in URL');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);
        
        // Load cached tickers and future symbols
        const [tickersResponse, futureSymbolsResponse] = await Promise.all([
          mcxApiClient.getTickers(false),
          mcxApiClient.getFutureSymbols(false)
        ]);
        
        if (tickersResponse.success && futureSymbolsResponse.success && 
            tickersResponse.data && futureSymbolsResponse.data) {
          
          // Find expiries for this symbol
          const optionSymbol = tickersResponse.data.Symbols.find(s => s.SymbolValue === symbol);
          const futureSymbol = futureSymbolsResponse.data.Products.find(p => p.Product === symbol);
          
          const combinedData: CombinedCommodityData = {
            optionExpiries: optionSymbol?.ExpiryDates || [],
            futureExpiries: futureSymbol?.ExpiryDates || []
          };
          
          const dataWithAge: McxDataWithAge<CombinedCommodityData> = {
            data: combinedData,
            age: tickersResponse.lastUpdated ? db.getDataAge(tickersResponse.lastUpdated) : 'just now',
            lastUpdated: tickersResponse.lastUpdated || Date.now(),
            fromCache: tickersResponse.fromCache || false
          };
          setCommodityData(dataWithAge);
          
          // Auto-select first available expiry
          if (combinedData.optionExpiries.length > 0) {
            setSelectedExpiry(combinedData.optionExpiries[0]);
            setSelectedType('options');
          } else if (combinedData.futureExpiries.length > 0) {
            setSelectedExpiry(combinedData.futureExpiries[0]);
            setSelectedType('futures');
          }
          
          // Fetch latest futures data for header display
          if (combinedData.futureExpiries.length > 0) {
            setTimeout(() => fetchLatestFuturesData(), 100);
          }
        } else {
          setError(tickersResponse.error || futureSymbolsResponse.error || 'Failed to load commodity data');
        }
      } catch (err) {
        setError(handleMcxApiError(err));
      } finally {
        setLoading(false);
      }
    };

    loadCommodityData();
  }, [symbol]);

  const fetchOptionChainAnalysis = async (expiry: string, forceRefresh = false) => {
    if (!symbol) return;

    try {
      if (forceRefresh) {
        setAnalysisFetching(true);
      } else {
        setAnalysisLoading(true);
      }
      
      // Format expiry date for API
      const formattedExpiry = formatExpiryForAPI(expiry);
      
      const response = await mcxApiClient.getOptionChain(symbol, formattedExpiry, forceRefresh);
      
      if (response.success && response.data) {
        const dataWithAge: McxDataWithAge<McxOptionChainResponse> = {
          data: response.data as McxOptionChainResponse,
          age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
          lastUpdated: response.lastUpdated || Date.now(),
          fromCache: response.fromCache || false
        };
        setAnalysisData(dataWithAge);
      } else {
        setError(response.error || 'Failed to fetch option chain');
      }
    } catch (err) {
      setError(handleMcxApiError(err));
    } finally {
      setAnalysisLoading(false);
      setAnalysisFetching(false);
    }
  };

  const handleExpirySelect = (expiry: string, type: 'options' | 'futures') => {
    setSelectedExpiry(expiry);
    setSelectedType(type);
    setAnalysisData(null);
    setFuturesQuote(null);
    
    if (type === 'options') {
      fetchOptionChainAnalysis(expiry);
    } else {
      fetchFuturesQuote(expiry);
    }
  };

  const handleFetchCommodityData = async () => {
    if (!symbol) return;
    
    try {
      setFetching(true);
      
      // Force refresh both tickers and future symbols
      const [tickersResponse, futureSymbolsResponse] = await Promise.all([
        mcxApiClient.getTickers(true),
        mcxApiClient.getFutureSymbols(true)
      ]);
      
      if (tickersResponse.success && futureSymbolsResponse.success && 
          tickersResponse.data && futureSymbolsResponse.data) {
        
        const optionSymbol = tickersResponse.data.Symbols.find(s => s.SymbolValue === symbol);
        const futureSymbol = futureSymbolsResponse.data.Products.find(p => p.Product === symbol);
        
        const combinedData: CombinedCommodityData = {
          optionExpiries: optionSymbol?.ExpiryDates || [],
          futureExpiries: futureSymbol?.ExpiryDates || []
        };
        
        const dataWithAge: McxDataWithAge<CombinedCommodityData> = {
          data: combinedData,
          age: tickersResponse.lastUpdated ? db.getDataAge(tickersResponse.lastUpdated) : 'just now',
          lastUpdated: tickersResponse.lastUpdated || Date.now(),
          fromCache: false
        };
        setCommodityData(dataWithAge);
      }
    } catch (err) {
      setError(handleMcxApiError(err));
    } finally {
      setFetching(false);
    }
  };

  const handleFetchAnalysis = () => {
    if (selectedExpiry) {
      if (selectedType === 'options') {
        fetchOptionChainAnalysis(selectedExpiry, true);
      } else {
        fetchFuturesQuote(selectedExpiry, true);
      }
    }
  };

  const parseExpiry = (expiry: string): number => {
    return new Date(expiry.replace(/-/g, ' ')).getTime();
  };

  if (!symbol) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="w-16 h-16 text-nse-error mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-100 mb-2">No Commodity Specified</h1>
          <p className="text-gray-400 mb-6">Please provide a commodity symbol in the URL parameters.</p>
          <Link href="/" className="btn-primary">
            Back to Home
          </Link>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
          <p className="text-gray-400 text-lg">Loading commodity info from database...</p>
        </div>
      </div>
    );
  }

  if (error && !commodityData) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="w-16 h-16 text-nse-error mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-100 mb-2">Error Loading Data</h1>
          <p className="text-gray-400 mb-6">{error}</p>
          <Link href="/" className="btn-primary">
            Back to Home
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen py-8 px-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <header className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <Link href="/" className="inline-flex items-center text-gray-400 hover:text-nse-accent transition-colors">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Commodities
            </Link>

            <Link href="/batch_mcx/" className="inline-flex items-center text-gray-400 hover:text-nse-accent transition-colors">
              Go to MCX Batch
            </Link>
          </div>
          
          <div className="flex items-center gap-4 mb-6">
            <h1 className="text-4xl font-display font-bold text-gradient flex items-center">
              <span className="mr-3">{getMcxCommodityIcon(symbol)}</span>
              {symbol}
            </h1>
            {commodityData && (
              <span className="px-3 py-1 bg-nse-surface rounded-full text-sm text-gray-300">
                {commodityData.data.optionExpiries.length + commodityData.data.futureExpiries.length} Expiries Available
              </span>
            )}
          </div>

          {/* Commodity Info Age and Fetch Controls */}
          {commodityData && (
            <div className="card-glow rounded-lg p-4 mb-6">
              <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                <div className="flex items-center gap-3 text-sm">
                  <div className="flex items-center gap-2">
                    {commodityData.fromCache ? (
                      <Database className="w-4 h-4 text-blue-400" />
                    ) : (
                      <Clock className="w-4 h-4 text-green-400" />
                    )}
                    <span className="text-gray-400">
                      Commodity info {commodityData.fromCache ? 'from cache' : 'freshly fetched'}
                    </span>
                  </div>
                  <span className="text-gray-500">•</span>
                  <div className="flex items-center gap-2">
                    <Clock className="w-4 h-4 text-gray-500" />
                    <span className="text-gray-400">Updated {getRealTimeAge(commodityData.lastUpdated)}</span>
                  </div>
                </div>
                
                <button
                  onClick={handleFetchCommodityData}
                  disabled={fetching}
                  className="btn-secondary inline-flex items-center text-sm"
                >
                  {fetching ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <RefreshCw className="w-4 h-4 mr-2" />
                  )}
                  {fetching ? 'Fetching...' : 'Update Expiries'}
                </button>
              </div>
            </div>
          )}
        </header>

        {/* Expiry Selection */}
        {commodityData && (
          <section className="card-glow rounded-lg p-6 mb-8">
            <div className="flex items-center gap-4 mb-6">
              <Calendar className="w-5 h-5 text-nse-accent" />
              <h2 className="text-xl font-semibold text-gray-100">Select Expiry Date & Type</h2>
            </div>
            
            {/* Options Expiries */}
            {commodityData.data.optionExpiries.length > 0 && (
              <div className="mb-6">
                <div className="flex items-center gap-3 mb-3">
                  <Layers className="w-4 h-4 text-nse-accent" />
                  <h3 className="text-lg font-medium text-gray-200">Options Expiries ({commodityData.data.optionExpiries.length})</h3>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3">
                  {[...commodityData.data.optionExpiries]
                    .sort((a, b) => parseExpiry(a) - parseExpiry(b))
                    .map((expiry) => (
                    <button
                      key={`option-${expiry}`}
                      onClick={() => handleExpirySelect(expiry, 'options')}
                      disabled={analysisLoading || analysisFetching}
                      className={`p-3 rounded-lg font-medium transition-all ${
                        selectedExpiry === expiry && selectedType === 'options'
                          ? 'bg-nse-accent text-white shadow-lg'
                          : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                      } ${(analysisLoading || analysisFetching) ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                      <div className="text-xs text-gray-400 mb-1">OPT</div>
                      {expiry}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Futures Expiries */}
            {commodityData.data.futureExpiries.length > 0 && (
              <div>
                <div className="flex items-center gap-3 mb-3">
                  <BarChart className="w-4 h-4 text-nse-accent" />
                <h3 className="text-lg font-medium text-gray-200">
                  Futures Expiries ({commodityData.data.futureExpiries.length})
                </h3>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3">
                {[...commodityData.data.futureExpiries]
                  .sort((a, b) => parseExpiry(a) - parseExpiry(b))
                  .map((expiry) => (
                    <button
                      key={`future-${expiry}`}
                      onClick={() => handleExpirySelect(expiry, 'futures')}
                      disabled={futuresLoading}
                      className={`p-3 rounded-lg font-medium transition-all ${
                        selectedExpiry === expiry && selectedType === 'futures'
                          ? 'bg-nse-accent text-white shadow-lg'
                          : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                      } ${futuresLoading ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                      <div className="text-xs text-gray-400 mb-1">FUT</div>
                      {expiry}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </section>
        )}

        {/* Analysis Results */}
        {selectedExpiry && (
          <section className="space-y-6">
            {/* Options Analysis */}
            {selectedType === 'options' && (
              <>
                {/* Analysis Data Age and Fetch Controls */}
                {analysisData && (
                  <div className="card-glow rounded-lg p-4">
                    <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                      <div className="flex items-center gap-3 text-sm">
                        <div className="flex items-center gap-2">
                          {analysisData.fromCache ? (
                            <Database className="w-4 h-4 text-blue-400" />
                          ) : (
                            <Clock className="w-4 h-4 text-green-400" />
                          )}
                          <span className="text-gray-400">
                            Options analysis {analysisData.fromCache ? 'from cache' : 'freshly fetched'} for {selectedExpiry}
                          </span>
                        </div>
                        <span className="text-gray-500">•</span>
                        <div className="flex items-center gap-2">
                          <Clock className="w-4 h-4 text-gray-500" />
                          <span className="text-gray-400">Updated {getRealTimeAge(analysisData.lastUpdated)}</span>
                        </div>
                      </div>
                      
                      <button
                        onClick={handleFetchAnalysis}
                        disabled={analysisFetching}
                        className="btn-secondary inline-flex items-center text-sm"
                      >
                        {analysisFetching ? (
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        ) : (
                          <RefreshCw className="w-4 h-4 mr-2" />
                        )}
                        {analysisFetching ? 'Fetching...' : 'Fetch Data'}
                      </button>
                    </div>
                  </div>
                )}

                {(analysisLoading || analysisFetching) ? (
                  <div className="card-glow rounded-lg p-12 text-center">
                    <Loader2 className="w-8 h-8 animate-spin mx-auto text-nse-accent mb-4" />
                    <p className="text-gray-400">
                      {analysisFetching ? 'Fetching latest analysis' : 'Loading cached analysis'} for {symbol} ({selectedExpiry})...
                    </p>
                  </div>
                ) : analysisData ? (
                  <>
                    {/* Summary Cards */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="text-sm text-gray-400">Underlying Value</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          {formatCurrency(analysisData.data.underlyingValue)}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">as of {analysisData.data.timestamp}</p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <Coins className="w-5 h-5 text-nse-accent" />
                          <span className="text-sm text-gray-400">Spread</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          {analysisData.data.spread.toFixed(2)}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">between strikes</p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <Clock className="w-5 h-5 text-nse-accent" />
                          <span className="text-sm text-gray-400">Days to Expiry</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          {analysisData.data.days_to_expiry}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">trading days left</p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <TrendingUp className="w-5 h-5 text-nse-accent" />
                          <span className="text-sm text-gray-400">Total OI</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          CE: {Number(analysisData.data.ce_oi).toLocaleString("en-IN", {maximumFractionDigits: 0})}
                        </p>
                        <p className="text-2xl font-bold text-gray-100">
                          PE: {Number(analysisData.data.pe_oi).toLocaleString("en-IN", {maximumFractionDigits: 0})}
                        </p>
                      </div>
                    </div>

                    {/* Alerts */}
                    {analysisData.data.alerts && analysisData.data.alerts.alerts && analysisData.data.alerts.alerts.length > 0 && (
                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-4">
                          <AlertCircle className="w-5 h-5 text-nse-warning" />
                          <h2 className="text-xl font-semibold text-gray-100">
                            Active Alerts ({analysisData.data.alerts.alerts.length}) 
                          </h2>
                        </div>
                        
                        <div className="space-y-4">
                          {analysisData.data.alerts.alerts.map((alert, index) => (
                            <div key={index} className="bg-slate-800/50 rounded-lg p-4 border border-gray-700/50">
                              <div className="flex items-start justify-between mb-3">
                                <div className="flex items-center gap-3">
                                  <span className={getAlertBadgeClass(alert.alert_type)}>
                                    {alert.alert_type.replace('_', ' ')}
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
                                  <span>
                                    Expiry: {alert.expiryDates}
                                  </span>
                                </div>
                              </div>
                              
                              <p className="text-gray-300 mb-3">{alert.description}</p>
                              
                              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                                <div>
                                  <span className="text-gray-500">Time Value:</span>
                                  <p className="text-gray-200 font-medium">₹{alert.values.time_val.toFixed(2)}</p>
                                </div>
                                <div>
                                  <span className="text-gray-500">The Money:</span>
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
                                      {alert.values.pchangeinOpenInterest.toFixed(2)}%
                                    </p>
                                  </div>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Summary Table - Commodity Header */}
                    <div className="card-glow rounded-t-lg overflow-hidden" style={{marginBottom:0}}>
                      <div className="overflow-x-auto">
                        <table className="w-full">
                          <tbody>
                            {/* Row 1: Symbol, Futures Info, Fetch Timestamp */}
                            <tr className="border-b border-gray-700/50">
                              <td className="px-6 py-4">
                                <div className="text-gradient text-xl font-bold flex items-center">
                                  <span className="mr-2">{getMcxCommodityIcon(symbol)}</span>
                                  {symbol}
                                </div>
                                <div className="font-bold">
                                  ₹ {analysisData.data.underlyingValue.toLocaleString("en-IN")}
                                </div>
                                <div className="text-l"><sub>{analysisData.data.timestamp}: {getDataTimestampAge(analysisData.data.timestamp)}</sub></div>
                              </td>

                              <td>
                                <div className="font-bold">
                                  FUTURES
                                  {latestFuturesData && (
                                    <span className={`text-l `}>
                                      {latestFuturesData.action}
                                    </span>
                                  )}
                                </div>
                                <div>
                                  {latestFuturesData && `₹ ${latestFuturesData.lastPrice.toLocaleString("en-IN")}`}
                                </div>
                                <div><sub>{latestFuturesData?.expiryDate}: {getDataTimestampAge(latestFuturesData?.timestamp)}</sub></div>
                              </td>

                              <td className="px-6 py-4 text-right" colSpan={2}>
                                <div className="font-medium text-gray-100">
                                  Fetched {getRealTimeAge(analysisData.lastUpdated)}
                                </div>
                                <div className="text-sm text-gray-400">
                                  <button
                                    onClick={handleFetchAnalysis}
                                    disabled={analysisFetching}
                                    className="btn-outline-primary inline-flex items-center text-sm"
                                  >
                                    {analysisFetching ? (
                                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                                    ) : (
                                      <RefreshCw className="w-4 h-4 mr-2" />
                                    )}
                                    {analysisFetching ? 'Fetching...' : 'Fetch Latest'}
                                  </button>
                                </div>
                              </td>
                            </tr>
                            
                            {/* Row 2: CE OI, Expiry, PE OI */}
                            <tr>
                              <td className="px-6 py-4 text-center">
                                <div className="font-bold text-lg">
                                  Total CE: {analysisData.data.ce_oi.toLocaleString("en-IN")}
                                </div>
                              </td>
                              <td className="px-6 py-4 text-center" colSpan={2}>
                                <div className="font-bold text-lg text-nse-accent">
                                  Expiry: {selectedExpiry} <sub>({analysisData.data.days_to_expiry} days left)</sub>
                                </div>
                              </td>
                              <td className="px-6 py-4 text-center">
                                <div className="font-bold text-lg">
                                  Total PE: {analysisData.data.pe_oi.toLocaleString("en-IN")}
                                </div>
                              </td>
                            </tr>
                          </tbody>
                        </table>
                      </div>
                    </div>

                    {/* Options Chain Table */}
                    <div className="card-glow rounded-b-lg overflow-hidden -mt-0">
                      <div className="overflow-x-auto">
                        <table className="data-table">
                          <thead>
                            <tr>
                              <th>CE OI</th>
                              <th>CE %OI</th>
                              <th>CE Money</th>
                              <th>CE %LTP</th>
                              <th>CE LTP</th>
                              <th>CE Tambu</th>
                              <th>STRIKE</th>
                              <th>PE Tambu</th>
                              <th>PE LTP</th>
                              <th>PE %LTP</th>
                              <th>PE Money</th>
                              <th>PE %OI</th>
                              <th>PE OI</th>
                            </tr>
                          </thead>
                          <tbody>
                            {analysisData.data.processed_data
                              .sort((a, b) => (b.strikePrice || 0) - (a.strikePrice || 0))
                              .map((option, index) => {
                                const ceOIRank = getOIRankStyling(option.CE?.oiRank);
                                const peOIRank = getOIRankStyling(option.PE?.oiRank);
                                
                                return (
                                  <tr key={index}>
                                    {/* CE Data */}
                                    <td className={`${option.CE?.openInterest ? ceOIRank.className : 'text-gray-400'}`}>
                                      <div>
                                        {option.CE?.openInterest != null ? Math.trunc(Number(option.CE.openInterest)) : '-'}
                                        {ceOIRank.showRank && (
                                          <sup className="ml-1 text-xs">
                                            {option.CE?.oiRank}
                                          </sup>
                                        )}
                                      </div>
                                    </td>
                                    <td className={option.CE?.pchangeinOpenInterest ? 
                                      (option.CE.pchangeinOpenInterest > 0 ? 'pchange-positive' : 'pchange-negative') : 'pchange-neutral'}>
                                      {option.CE?.pchangeinOpenInterest ? 
                                        `${option.CE.pchangeinOpenInterest > 0 ? '+' : ''}${option.CE.pchangeinOpenInterest.toFixed(1)}%` : '0'}
                                    </td>
                                    <td className={`${option.CE ? getMoneyStatusColor(option.CE.the_money) : 'text-gray-400'}`}>
                                      <div>
                                        {option.CE?.the_money || '-'}
                                      </div>
                                    </td>
                                    <td className={option.CE?.pchange ? 
                                      (option.CE.pchange > 0 ? 'pchange-positive' : 'pchange-negative') : 'pchange-neutral'}>
                                      {option.CE?.pchange ? 
                                        `${option.CE.pchange > 0 ? '+' : ''}${option.CE.pchange.toFixed(1)}%` : '-'}
                                    </td>
                                    <td className="option-ce">
                                      {option.CE?.lastPrice ? `₹${option.CE.lastPrice.toFixed(2)}` : '-'}
                                    </td>
                                    <td className="text-gray-300">
                                      {option.CE?.tambu || '-'}
                                    </td>
                                    
                                    {/* Strike Price */}
                                    <td className={`font-bold text-center ${option.CE ? getMoneyStatusColor(option.CE.the_money) : ""}`} 
                                        style={{ backgroundColor: "rgba(51, 65, 85, 0.5)" }}> 
                                      {option.strikePrice}
                                    </td>
                                    
                                    {/* PE Data */}
                                    <td className="text-gray-300">
                                      {option.PE?.tambu || '-'}
                                    </td>
                                    <td className="option-pe">
                                      {option.PE?.lastPrice ? `₹${option.PE.lastPrice.toFixed(2)}` : '-'}
                                    </td>
                                    <td className={option.PE?.pchange ? 
                                      (option.PE.pchange > 0 ? 'pchange-positive' : 'pchange-negative') : 'pchange-neutral'}>
                                      {option.PE?.pchange ? 
                                        `${option.PE.pchange > 0 ? '+' : ''}${option.PE.pchange.toFixed(1)}%` : '-'}
                                    </td>
                                    <td className={`${option.PE ? getMoneyStatusColor(option.PE.the_money) : 'text-gray-400'}`}>
                                      <div>
                                        {option.PE?.the_money || '-'}
                                      </div>
                                    </td>
                                    <td className={option.PE?.pchangeinOpenInterest ? 
                                      (option.PE.pchangeinOpenInterest > 0 ? 'pchange-positive' : 'pchange-negative') : 'pchange-neutral'}>
                                      {option.PE?.pchangeinOpenInterest ? 
                                        `${option.PE.pchangeinOpenInterest > 0 ? '+' : ''}${option.PE.pchangeinOpenInterest.toFixed(1)}%` : '0'}
                                    </td>
                                    <td className={`${option.PE?.openInterest ? peOIRank.className : 'text-gray-400'}`}>
                                      <div>
                                        {option.PE?.openInterest != null ? Math.trunc(Number(option.PE.openInterest)) : '-'}
                                        {peOIRank.showRank && (
                                          <sup className="ml-1 text-xs">
                                            {option.PE?.oiRank}
                                          </sup>
                                        )}
                                      </div>
                                    </td>
                                  </tr>
                                );
                              })}
                          </tbody>
                        </table>
                      </div>
                    </div>

                    {/* No Alerts Message */}
                    {(!analysisData.data.alerts || !analysisData.data.alerts.alerts || analysisData.data.alerts.alerts.length === 0) && (
                      <div className="card-glow rounded-lg p-8 text-center">
                        <AlertCircle className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                        <h3 className="text-lg font-semibold text-gray-300 mb-2">No Alerts Found</h3>
                        <p className="text-gray-500">
                          All options for {symbol} ({selectedExpiry}) are within normal parameters
                        </p>
                      </div>
                    )}  
                  </>
                ) : null}
              </>
            )}

            {/* Futures Analysis */}
            {selectedType === 'futures' && (
              <div className="space-y-6">
                {/* Futures Data Age and Fetch Controls */}
                {futuresData && (
                  <div className="card-glow rounded-lg p-4">
                    <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                      <div className="flex items-center gap-3 text-sm">
                        <div className="flex items-center gap-2">
                          {futuresData.fromCache ? (
                            <Database className="w-4 h-4 text-blue-400" />
                          ) : (
                            <Clock className="w-4 h-4 text-green-400" />
                          )}
                          <span className="text-gray-400">
                            Futures data {futuresData.fromCache ? 'from cache' : 'freshly fetched'} for {selectedExpiry}
                          </span>
                        </div>
                        <span className="text-gray-500">•</span>
                        <div className="flex items-center gap-2">
                          <Clock className="w-4 h-4 text-gray-500" />
                          <span className="text-gray-400">Updated {getRealTimeAge(futuresData.lastUpdated)}</span>
                        </div>
                      </div>
                      
                      <button
                        onClick={handleFetchAnalysis}
                        disabled={futuresLoading}
                        className="btn-secondary inline-flex items-center text-sm"
                      >
                        {futuresLoading ? (
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        ) : (
                          <RefreshCw className="w-4 h-4 mr-2" />
                        )}
                        {futuresLoading ? 'Fetching...' : 'Fetch Data'}
                      </button>
                    </div>
                  </div>
                )}

                {futuresLoading ? (
                  <div className="card-glow rounded-lg p-12 text-center">
                    <Loader2 className="w-8 h-8 animate-spin mx-auto text-nse-accent mb-4" />
                    <p className="text-gray-400">Loading futures data for {symbol} ({selectedExpiry})...</p>
                  </div>
                ) : futuresQuote ? (
                  <>
                    {/* Futures Summary Cards */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="text-sm text-gray-400">Previous Close</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          ₹{futuresQuote.previousClose?.toFixed(2) || 0}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">as of {futuresQuote.timestamp}</p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="text-sm text-gray-400">Last Traded Price</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          ₹{futuresQuote.LTP?.toFixed(2)}
                        </p>
                        <p className={`text-sm mt-1 ${futuresQuote.percentChange && futuresQuote.percentChange >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                          {futuresQuote.absoluteChange && futuresQuote.absoluteChange >= 0 ? '+' : ''}₹{futuresQuote.absoluteChange?.toFixed(2)} or 
                          ({futuresQuote.percentChange && futuresQuote.percentChange >= 0 ? '+' : ''}
                          {futuresQuote.percentChange ? (futuresQuote.percentChange * 100)?.toFixed(2) : 0}%)

                        </p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="text-sm text-gray-400">Action Analysis</span>
                        </div>
                        <p className={`text-2xl font-bold`}>
                          {futuresQuote.action}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">market sentiment</p>
                      </div>

                      <div className="card-glow rounded-lg p-6">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="text-sm text-gray-400">Open Interest</span>
                        </div>
                        <p className="text-2xl font-bold text-gray-100">
                          {futuresQuote.openInterest.toLocaleString("en-IN")}
                        </p>
                        <p className={`text-sm mt-1 ${futuresQuote.changeinOpenInterest >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                          (
                            {futuresQuote.changeinOpenInterest >= 0 ? '+' : ' '}
                            {futuresQuote.changeinOpenInterest.toLocaleString("en-IN")} 
                          )
                          or 
                          ( 
                            {futuresQuote.changeinOpenInterest >= 0 ? '+' : ' '}
                            {futuresQuote.pchangeinOpenInterest ? (futuresQuote.pchangeinOpenInterest * 100)?.toFixed(2) : 0}%
                          )
                        </p>
                      </div>
                    </div>

                    {/* Futures Details */}
                    <div className="card-glow rounded-lg p-6">
                      <div className="flex items-center gap-3 mb-4">
                        <BarChart className="w-5 h-5 text-nse-accent" />
                        <h2 className="text-xl font-semibold text-gray-100 ">
                          {futuresQuote.Productdesc} <i>Futures Contract of </i> {futuresQuote.expiryDate}
                        </h2>
                      </div>
                      
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
                        <div className="text-center">
                          <p className="text-gray-400 text-sm">Unit</p>
                          <p className="text-xl font-bold text-gray-100">
                            {futuresQuote.TradingUnit}
                          </p>
                        </div>
                        <div className="text-center">
                          <p className="text-gray-400 text-sm">Data As On</p>
                          <p className="text-lg font-medium text-gray-100">
                            {futuresQuote.timestamp}
                          </p>
                        </div>
                        <div className="text-center">
                          <p className="text-gray-400 text-sm">Category</p>
                          <p className="text-lg font-medium text-nse-accent">
                            {futuresQuote.Category}
                          </p>
                        </div>
                      </div>
                    </div>

                  </>
                ) : (
                  <div className="card-glow rounded-lg p-8 text-center">
                    <BarChart className="w-8 h-8 text-nse-accent mx-auto mb-4" />
                    <h3 className="text-xl font-semibold text-gray-100 mb-4">
                      {symbol} Futures - {selectedExpiry}
                    </h3>
                    <p className="text-gray-500">No futures data available for this expiry</p>
                  </div>
                )}
              </div>
            )}
          </section>
        )}
      </div>
    </div>
  );
}

// Loading component for Suspense fallback
function CommodityPageLoading() {
  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
        <p className="text-gray-400 text-lg">Loading commodity page...</p>
      </div>
    </div>
  );
}

// Main component with Suspense wrapper
export default function CommodityPage() {
  return (
    <Suspense fallback={<CommodityPageLoading />}>
      <CommodityPageContent />
    </Suspense>
  );
}