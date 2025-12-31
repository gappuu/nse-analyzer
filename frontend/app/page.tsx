'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { 
  Search, 
  BarChart3, 
  AlertCircle, 
  Loader2,
  ArrowRight,
  Target,
  RefreshCw,
  Clock,
  Database,
  TrendingUp,
  Coins,
  Trash2
} from 'lucide-react';
import { apiClient, handleApiError } from '@/app/lib/api_nse';
import { mcxApiClient, handleMcxApiError, getMcxCommodityIcon, getMcxCommodityLetters } from '@/app/lib/api_mcx';
import { db } from '@/app/lib/db';
import { SecurityInfo, SecurityListResponse, DataWithAge } from '@/app/types/api_nse_type';
import { McxDataWithAge, McxTickersResponse, McxFutureSymbolsResponse } from '@/app/types/api_mcx_type';

type TabType = 'nse' | 'mcx';

interface CombinedMcxData {
  tickers: McxTickersResponse;
  futureSymbols: McxFutureSymbolsResponse;
}

export default function HomePage() {
  const [activeTab, setActiveTab] = useState<TabType>('nse');
  
  // NSE State
  const [securitiesData, setSecuritiesData] = useState<DataWithAge<SecurityListResponse> | null>(null);
  const [nseLoading, setNseLoading] = useState(false);
  const [nseFetching, setNseFetching] = useState(false);
  const [nseError, setNseError] = useState<string | null>(null);
  
  // MCX State
  const [mcxData, setMcxData] = useState<McxDataWithAge<CombinedMcxData> | null>(null);
  const [mcxLoading, setMcxLoading] = useState(false);
  const [mcxFetching, setMcxFetching] = useState(false);
  const [mcxError, setMcxError] = useState<string | null>(null);
  
  // Shared state
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedLetter, setSelectedLetter] = useState<string | null>(null);
  const [mcxSelectedLetter, setMcxSelectedLetter] = useState<string | null>(null);
  const [isClearing, setIsClearing] = useState(false);

  // Theme switching effect
  useEffect(() => {
    if (activeTab === 'mcx') {
      document.body.classList.add('mcx-theme');
    } else {
      document.body.classList.remove('mcx-theme');
    }
    // Cleanup on unmount
    return () => {
      document.body.classList.remove('mcx-theme');
    };
  }, [activeTab]);

  useEffect(() => {
      // Check for tab parameter in URL
      const urlParams = new URLSearchParams(window.location.search);
      const tabParam = urlParams.get('tab');
      if (tabParam === 'mcx' || tabParam === 'nse') {
        setActiveTab(tabParam as TabType);
      }
    }, []);

  useEffect(() => {
    if (activeTab === 'nse') {
      loadNseSecurities();
    } else {
      loadMcxData();
    }
  }, [activeTab]);

  const loadNseSecurities = async (forceRefresh = false) => {
    try {
      if (forceRefresh) {
        setNseFetching(true);
      } else {
        setNseLoading(true);
      }
      setNseError(null);
      
      const response = await apiClient.getSecurities(forceRefresh);
      
      if (response.success && response.data) {
        const dataWithAge: DataWithAge<SecurityListResponse> = {
          data: response.data,
          age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
          lastUpdated: response.lastUpdated || Date.now(),
          fromCache: response.fromCache || false
        };
        setSecuritiesData(dataWithAge);
      } else {
        setNseError(response.error || 'Failed to fetch NSE securities');
      }
    } catch (err) {
      setNseError(handleApiError(err));
    } finally {
      setNseLoading(false);
      setNseFetching(false);
    }
  };

  const loadMcxData = async (forceRefresh = false) => {
    try {
      if (forceRefresh) {
        setMcxFetching(true);
      } else {
        setMcxLoading(true);
      }
      setMcxError(null);
      
      // Fetch both tickers and future symbols
      const [tickersResponse, futureSymbolsResponse] = await Promise.all([
        mcxApiClient.getTickers(forceRefresh),
        mcxApiClient.getFutureSymbols(forceRefresh)
      ]);
      
      if (tickersResponse.success && futureSymbolsResponse.success && 
          tickersResponse.data && futureSymbolsResponse.data) {
        
        const combinedData: CombinedMcxData = {
          tickers: tickersResponse.data,
          futureSymbols: futureSymbolsResponse.data
        };
        
        const dataWithAge: McxDataWithAge<CombinedMcxData> = {
          data: combinedData,
          age: tickersResponse.lastUpdated ? db.getDataAge(tickersResponse.lastUpdated) : 'just now',
          lastUpdated: tickersResponse.lastUpdated || Date.now(),
          fromCache: tickersResponse.fromCache || false
        };
        setMcxData(dataWithAge);
      } else {
        setMcxError(tickersResponse.error || futureSymbolsResponse.error || 'Failed to fetch MCX data');
      }
    } catch (err) {
      setMcxError(handleMcxApiError(err));
    } finally {
      setMcxLoading(false);
      setMcxFetching(false);
    }
  };

  const handleNseFetchLatest = () => {
    loadNseSecurities(true);
  };

  const handleMcxFetchLatest = () => {
    loadMcxData(true);
  };

  const handleClearAllCache = async () => {
    if (!confirm('Are you sure you want to clear all cached data? This will remove all stored NSE and MCX data.')) {
      return;
    }

    setIsClearing(true);
    try {
      await db.clearAllData();
      
      // Clear state
      setSecuritiesData(null);
      setMcxData(null);
      
      alert('All cached data has been cleared successfully!');
    } catch (error) {
      console.error('Error clearing cache:', error);
      alert('Failed to clear cache. Please try again.');
    } finally {
      setIsClearing(false);
    }
  };

  const filteredNseSecurities = React.useMemo(() => {
    if (!securitiesData) return null;

    let filtered: SecurityInfo[] = [];

    // Add indices first
    filtered = [...securitiesData.data.indices];

    // Add filtered equities
    if (selectedLetter) {
      const letterSecurities = securitiesData.data.equities[selectedLetter] || [];
      filtered = [...filtered, ...letterSecurities];
    } else {
      // Add all equities if no letter selected
      Object.values(securitiesData.data.equities).forEach(letterGroup => {
        filtered = [...filtered, ...letterGroup];
      });
    }

    // Apply search filter
    if (searchTerm) {
      filtered = filtered.filter(security =>
        security.symbol.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }

    return filtered;
  }, [securitiesData, searchTerm, selectedLetter]);

  const filteredMcxCommodities = React.useMemo(() => {
    if (!mcxData) return [];

    // Combine tickers and future symbols
    const allCommodities: Array<{name: string, type: 'option' | 'future', expiryDates: string[]}> = [];
    
    // Add option tickers
    mcxData.data.tickers.Symbols.forEach(symbol => {
      allCommodities.push({
        name: symbol.SymbolValue,
        type: 'option',
        expiryDates: symbol.ExpiryDates
      });
    });
    
    // Add future symbols
    mcxData.data.futureSymbols.Products.forEach(product => {
      allCommodities.push({
        name: product.Product,
        type: 'future',
        expiryDates: product.ExpiryDates
      });
    });

    // Remove duplicates and apply search filter
    const uniqueCommodities = allCommodities.reduce((acc, curr) => {
      const existing = acc.find(item => item.name === curr.name);
      if (!existing) {
        acc.push(curr);
      } else {
        // Merge expiry dates and keep both types
        existing.expiryDates = [...new Set([...existing.expiryDates, ...curr.expiryDates])];
      }
      return acc;
    }, [] as Array<{name: string, type: 'option' | 'future', expiryDates: string[]}>);

    // Apply letter filter for MCX
    let letterFiltered = uniqueCommodities;
    if (mcxSelectedLetter) {
      letterFiltered = uniqueCommodities.filter(commodity =>
        commodity.name.charAt(0).toUpperCase() === mcxSelectedLetter
      );
    }

    // Apply search filter
    if (searchTerm) {
      letterFiltered = letterFiltered.filter(commodity =>
        commodity.name.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }

    return letterFiltered;
  }, [mcxData, searchTerm, mcxSelectedLetter]);

  const availableLetters = securitiesData ? Object.keys(securitiesData.data.equities).sort() : [];
  const mcxAvailableLetters = mcxData ? getMcxCommodityLetters(
    [...mcxData.data.tickers.Symbols.map(s => ({name: s.SymbolValue})),
     ...mcxData.data.futureSymbols.Products.map(p => ({name: p.Product}))]
  ) : [];

  const isLoading = activeTab === 'nse' ? nseLoading : mcxLoading;
  // const isFetching = activeTab === 'nse' ? nseFetching : mcxFetching;
  const error = activeTab === 'nse' ? nseError : mcxError;
  const hasData = activeTab === 'nse' ? !!securitiesData : !!mcxData;

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
          <p className="text-gray-400 text-lg">
            Loading {activeTab === 'nse' ? 'NSE securities' : 'MCX commodities'} from database...
          </p>
        </div>
      </div>
    );
  }

  if (error && !hasData) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="w-16 h-16 text-nse-error mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-100 mb-2">Error Loading Data</h1>
          <p className="text-gray-400 mb-6">{error}</p>
          <button 
            onClick={() => activeTab === 'nse' ? loadNseSecurities(true) : loadMcxData(true)}
            className="btn-primary"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="py-8 px-4 border-b border-gray-700/50">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-8">
            <h1 className="text-4xl md:text-6xl font-display font-bold mb-4">
              <span className="text-gradient">Multi-Market</span>
              <br />
              <span className="text-gradient-blue">Options Analyzer</span>
            </h1>
            <p className="text-gray-400 text-lg max-w-2xl mx-auto">
              Advanced F&O analysis platform with real-time options chain data for NSE & MCX, 
              intelligent alerts, and comprehensive market insights.
            </p>
          </div>

          {/* Market Tabs */}
          <div className="flex justify-center mb-8">
            <div className="card-glow rounded-lg p-2 inline-flex">
              <button
                onClick={() => setActiveTab('nse')}
                className={`inline-flex items-center px-6 py-3 rounded-lg font-medium transition-all ${
                  activeTab === 'nse'
                    ? 'bg-nse-accent text-white shadow-lg'
                    : 'text-gray-400 hover:text-gray-200 hover:bg-slate-700/50'
                }`}
              >
                <TrendingUp className="w-5 h-5 mr-2" />
                NSE (Equity & Indices)
              </button>
              <button
                onClick={() => setActiveTab('mcx')}
                className={`inline-flex items-center px-6 py-3 rounded-lg font-medium transition-all ${
                  activeTab === 'mcx'
                    ? 'bg-nse-accent text-white shadow-lg'
                    : 'text-gray-400 hover:text-gray-200 hover:bg-slate-700/50'
                }`}
              >
                <Coins className="w-5 h-5 mr-2" />
                MCX (Commodities)
              </button>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-8">
            <Link 
              href={activeTab === 'nse' ? '/batch/' : '/batch_mcx/'} 
              className="btn-success inline-flex items-center"
            >
              <BarChart3 className="w-5 h-5 mr-2" />
              Batch Analysis - All {activeTab === 'nse' ? 'F&O' : 'Commodities'}
              <ArrowRight className="w-4 h-4 ml-2" />
            </Link>
            <div className="text-gray-500">or select individual {activeTab === 'nse' ? 'security' : 'commodity'} below</div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="py-8 px-4">
        <div className="max-w-7xl mx-auto">
          {/* Search and Filter Controls */}
          <div className="card-glow rounded-lg p-6 mb-8">
            <div className="flex flex-col lg:flex-row gap-4 items-center">
              {/* Search */}
              <div className="relative flex-1 max-w-md">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  placeholder={`Search ${activeTab === 'nse' ? 'securities' : 'commodities'}...`}
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full pl-10 pr-4 py-3 bg-slate-800 border border-gray-600 rounded-lg 
                           focus:border-nse-accent focus:ring-2 focus:ring-nse-accent/20 
                           text-gray-100 placeholder-gray-400 transition-colors"
                />
              </div>

              {/* Letter Filter */}
              <div className="flex flex-wrap gap-2">
                <button
                  onClick={() => activeTab === 'nse' ? setSelectedLetter(null) : setMcxSelectedLetter(null)}
                  className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    (activeTab === 'nse' ? selectedLetter : mcxSelectedLetter) === null
                      ? 'bg-nse-accent text-white'
                      : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                  }`}
                >
                  All
                </button>
                {(activeTab === 'nse' ? availableLetters : mcxAvailableLetters).map((letter) => (
                  <button
                    key={letter}
                    onClick={() => activeTab === 'nse' ? setSelectedLetter(letter) : setMcxSelectedLetter(letter)}
                    className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                      (activeTab === 'nse' ? selectedLetter : mcxSelectedLetter) === letter
                        ? 'bg-nse-accent text-white'
                        : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                    }`}
                  >
                    {letter}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Content based on active tab */}
          {activeTab === 'nse' ? (
            <NseContent 
              filteredSecurities={filteredNseSecurities}
              securitiesData={securitiesData}
              searchTerm={searchTerm}
              selectedLetter={selectedLetter}
            />
          ) : (
            <McxContent 
              filteredCommodities={filteredMcxCommodities}
              searchTerm={searchTerm}
            />
          )}
        </div>
      </main>

      {/* Footer */}
      <footer className="page-footer">
        <div className="max-w-7xl mx-auto px-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* NSE Data Status */}
            <div className="footer-card">
              <h3 className="text-lg font-semibold text-gray-100 mb-3 flex items-center">
                <TrendingUp className="w-5 h-5 mr-2 text-nse-accent" />
                NSE Data Status
              </h3>
              {securitiesData ? (
                <div className="space-y-3">
                  <div className="flex items-center gap-3 text-sm">
                    <div className="flex items-center gap-2">
                      {securitiesData.fromCache ? (
                        <Database className="w-4 h-4 text-blue-400" />
                      ) : (
                        <Clock className="w-4 h-4 text-green-400" />
                      )}
                      <span className="text-gray-400">
                        Data {securitiesData.fromCache ? 'from cache' : 'freshly fetched'}
                      </span>
                    </div>
                    <span className="text-gray-500">•</span>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4 text-gray-500" />
                      <span className="text-gray-400">Updated {securitiesData.age}</span>
                    </div>
                  </div>
                  <button
                    onClick={handleNseFetchLatest}
                    disabled={nseFetching}
                    className="btn-secondary w-full inline-flex items-center justify-center text-sm"
                  >
                    {nseFetching ? (
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <RefreshCw className="w-4 h-4 mr-2" />
                    )}
                    {nseFetching ? 'Fetching...' : 'Fetch Latest NSE Data'}
                  </button>
                </div>
              ) : (
                <div className="text-center py-4">
                  <AlertCircle className="w-8 h-8 text-gray-500 mx-auto mb-2" />
                  <p className="text-gray-500 text-sm">No NSE data loaded</p>
                  <button
                    onClick={handleNseFetchLatest}
                    disabled={nseFetching}
                    className="btn-primary mt-2 text-sm"
                  >
                    Load NSE Data
                  </button>
                </div>
              )}
            </div>

            {/* MCX Data Status */}
            <div className="footer-card">
              <h3 className="text-lg font-semibold text-gray-100 mb-3 flex items-center">
                <Coins className="w-5 h-5 mr-2 text-nse-accent" />
                MCX Data Status
              </h3>
              {mcxData ? (
                <div className="space-y-3">
                  <div className="flex items-center gap-3 text-sm">
                    <div className="flex items-center gap-2">
                      {mcxData.fromCache ? (
                        <Database className="w-4 h-4 text-blue-400" />
                      ) : (
                        <Clock className="w-4 h-4 text-green-400" />
                      )}
                      <span className="text-gray-400">
                        Data {mcxData.fromCache ? 'from cache' : 'freshly fetched'}
                      </span>
                    </div>
                    <span className="text-gray-500">•</span>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4 text-gray-500" />
                      <span className="text-gray-400">Updated {mcxData.age}</span>
                    </div>
                  </div>
                  <button
                    onClick={handleMcxFetchLatest}
                    disabled={mcxFetching}
                    className="btn-secondary w-full inline-flex items-center justify-center text-sm"
                  >
                    {mcxFetching ? (
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <RefreshCw className="w-4 h-4 mr-2" />
                    )}
                    {mcxFetching ? 'Fetching...' : 'Fetch Latest MCX Data'}
                  </button>
                </div>
              ) : (
                <div className="text-center py-4">
                  <AlertCircle className="w-8 h-8 text-gray-500 mx-auto mb-2" />
                  <p className="text-gray-500 text-sm">No MCX data loaded</p>
                  <button
                    onClick={handleMcxFetchLatest}
                    disabled={mcxFetching}
                    className="btn-primary mt-2 text-sm"
                  >
                    Load MCX Data
                  </button>
                </div>
              )}
            </div>
          </div>

          {/* Footer Bottom */}
          <div className="border-t border-gray-600/30 mt-8 pt-6">
            <div className="text-center mb-4">
              <p className="text-gray-400 text-sm">
                Multi-Market Options Analyzer • NSE & MCX Data Platform
              </p>
              <p className="text-gray-500 text-xs mt-1">
                Real-time analysis with intelligent caching
              </p>
            </div>
            
            {/* Clear Cache Button */}
            <div className="flex justify-center mt-4">
              <button
                onClick={handleClearAllCache}
                disabled={isClearing}
                className="inline-flex items-center px-4 py-2 rounded-lg font-medium transition-all text-sm
                         bg-red-600/20 text-red-400 hover:bg-red-600/30 border border-red-500/50
                         hover:border-red-400 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isClearing ? (
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                ) : (
                  <Trash2 className="w-4 h-4 mr-2" />
                )}
                {isClearing ? 'Clearing Cache...' : 'Clear All Cached Data'}
              </button>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

// NSE Content Component
function NseContent({ 
  filteredSecurities, 
  securitiesData, 
  searchTerm, 
  selectedLetter 
}: {
  filteredSecurities: SecurityInfo[] | null;
  securitiesData: DataWithAge<SecurityListResponse> | null;
  searchTerm: string;
  selectedLetter: string | null;
}) {
  if (!filteredSecurities || !securitiesData) return null;

  return (
    <div className="space-y-8">
      {securitiesData.data.indices && securitiesData.data.indices.length > 0 && (
        selectedLetter === null || securitiesData.data.indices.some(s => 
          s.symbol.toLowerCase().includes(searchTerm.toLowerCase())
        )
      ) && (
        <section className="animate-fade-in">
          <h2 className="text-2xl font-bold text-gray-100 mb-4 flex items-center">
            <Target className="w-6 h-6 mr-2 text-nse-accent" />
            Indices
          </h2>
          <div className="security-grid">
            {securitiesData.data.indices
              .filter(security => 
                searchTerm === '' || 
                security.symbol.toLowerCase().includes(searchTerm.toLowerCase())
              )
              .map((security, index) => (
                <SecurityCard 
                  key={security.symbol} 
                  security={security} 
                  index={index}
                />
              ))}
          </div>
        </section>
      )}

      {/* Equities by Letter or All */}
      {selectedLetter ? (
        <section className="animate-fade-in">
          <h2 className="text-2xl font-bold text-gray-100 mb-4">
            Securities - {selectedLetter}
          </h2>
          <div className="security-grid">
            {(securitiesData.data.equities[selectedLetter] || [])
              .filter(security => 
                searchTerm === '' || 
                security.symbol.toLowerCase().includes(searchTerm.toLowerCase())
              )
              .map((security, index) => (
                <SecurityCard 
                  key={security.symbol} 
                  security={security} 
                  index={index}
                />
              ))}
          </div>
        </section>
      ) : (
        <section className="animate-fade-in">
          <h2 className="text-2xl font-bold text-gray-100 mb-4">
            All Equities
          </h2>
          <div className="security-grid">
            {Object.entries(securitiesData.data.equities || {})
              .flatMap(([letter, letterSecurities]) => letterSecurities)
              .filter(security => 
                searchTerm === '' || 
                security.symbol.toLowerCase().includes(searchTerm.toLowerCase())
              )
              .map((security, index) => (
                <SecurityCard 
                  key={security.symbol} 
                  security={security} 
                  index={index}
                />
              ))}
          </div>
        </section>
      )}

      {filteredSecurities.length === 0 && (
        <div className="text-center py-12">
          <Search className="w-16 h-16 text-gray-600 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-gray-400 mb-2">No securities found</h3>
          <p className="text-gray-500">Try adjusting your search or filter criteria</p>
        </div>
      )}
    </div>
  );
}

// MCX Content Component
function McxContent({ 
  filteredCommodities,
  // searchTerm
}: {
  filteredCommodities: Array<{name: string, type: 'option' | 'future', expiryDates: string[]}>;
  searchTerm: string;
}) {
  return (
    <div className="space-y-8">
      <section className="animate-fade-in">
        <h2 className="text-2xl font-bold text-gray-100 mb-4 flex items-center">
          <Coins className="w-6 h-6 mr-2 text-nse-accent" />
          Commodities
        </h2>
        
        {filteredCommodities.length > 0 ? (
          <div className="security-grid">
            {filteredCommodities.map((commodity, index) => (
              <CommodityCard 
                key={commodity.name} 
                commodity={commodity} 
                index={index}
              />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <Search className="w-16 h-16 text-gray-600 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-gray-400 mb-2">No commodities found</h3>
            <p className="text-gray-500">Try adjusting your search criteria</p>
          </div>
        )}
      </section>
    </div>
  );
}

// Security Card Component - Updated to use query parameters for static export
function SecurityCard({ security, index }: { security: SecurityInfo; index: number }) {
  return (
    <Link 
      href={`/security/?symbol=${encodeURIComponent(security.symbol)}`} 
      className="security-card group"
      style={{ animationDelay: `${index * 0.05}s` }}
    >
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-display font-semibold text-lg text-gray-100 group-hover:text-nse-accent transition-colors">
            {security.symbol}
          </h3>
          <p className="text-sm text-gray-400 mt-1">
            {security.security_type === 'Indices' ? '📊 Index' : '🏢 Equity'}
          </p>
        </div>
        <div className="opacity-0 group-hover:opacity-100 transition-opacity">
          <ArrowRight className="w-5 h-5 text-nse-accent" />
        </div>
      </div>
    </Link>
  );
}

// Commodity Card Component
function CommodityCard({ 
  commodity, 
  index 
}: { 
  commodity: {name: string, type: 'option' | 'future', expiryDates: string[]}, 
  index: number 
}) {
  return (
    <Link 
      href={`/commodity/?symbol=${encodeURIComponent(commodity.name)}`} 
      className="security-card group"
      style={{ animationDelay: `${index * 0.05}s` }}
    >
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-display font-semibold text-lg text-gray-100 group-hover:text-nse-accent transition-colors flex items-center">
            <span className="mr-2">{getMcxCommodityIcon(commodity.name)}</span>
            {commodity.name}
          </h3>
          <p className="text-sm text-gray-400 mt-1">
            {commodity.type === 'option' ? '⚙️ Options Available' : '📈 Futures Available'}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {commodity.expiryDates.length} expiry dates
          </p>
        </div>
        <div className="opacity-0 group-hover:opacity-100 transition-opacity">
          <ArrowRight className="w-5 h-5 text-nse-accent" />
        </div>
      </div>
    </Link>
  );
}