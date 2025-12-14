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
  Database
} from 'lucide-react';
import { apiClient, handleApiError } from '@/app/lib/api';
import { db } from '@/app/lib/db';
import { SecurityInfo, SecurityListResponse, DataWithAge } from '@/app/types/api';

export default function HomePage() {
  const [securitiesData, setSecuritiesData] = useState<DataWithAge<SecurityListResponse> | null>(null);
  const [loading, setLoading] = useState(true);
  const [fetching, setFetching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedLetter, setSelectedLetter] = useState<string | null>(null);

  useEffect(() => {
    loadSecurities();
  }, []);

  const loadSecurities = async (forceRefresh = false) => {
    try {
      if (forceRefresh) {
        setFetching(true);
      } else {
        setLoading(true);
      }
      setError(null);
      
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
        setError(response.error || 'Failed to fetch securities');
      }
    } catch (err) {
      setError(handleApiError(err));
    } finally {
      setLoading(false);
      setFetching(false);
    }
  };

  const handleFetchLatest = () => {
    loadSecurities(true);
  };

  const filteredSecurities = React.useMemo(() => {
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

  const availableLetters = securitiesData ? Object.keys(securitiesData.data.equities).sort() : [];

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
          <p className="text-gray-400 text-lg">Loading securities from database...</p>
        </div>
      </div>
    );
  }

  if (error && !securitiesData) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="w-16 h-16 text-nse-error mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-100 mb-2">Error Loading Data</h1>
          <p className="text-gray-400 mb-6">{error}</p>
          <button 
            onClick={() => loadSecurities(true)}
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
              <span className="text-gradient">NSE Options</span>
              <br />
              <span className="text-gradient-blue">Analyzer</span>
            </h1>
            <p className="text-gray-400 text-lg max-w-2xl mx-auto">
              Advanced F&O analysis platform with real-time options chain data, 
              intelligent alerts, and comprehensive market insights.
            </p>
          </div>

          {/* Data Age and Fetch Controls */}
          {securitiesData && (
            <div className="card-glow rounded-lg p-4 mb-6">
              <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
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
                  onClick={handleFetchLatest}
                  disabled={fetching}
                  className="btn-secondary inline-flex items-center text-sm"
                >
                  {fetching ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <RefreshCw className="w-4 h-4 mr-2" />
                  )}
                  {fetching ? 'Fetching...' : 'Fetch Latest'}
                </button>
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-8">
            <Link href="/batch" className="btn-success inline-flex items-center">
              <BarChart3 className="w-5 h-5 mr-2" />
              Batch Analysis - All F&O
              <ArrowRight className="w-4 h-4 ml-2" />
            </Link>
            <div className="text-gray-500">or select individual security below</div>
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
                  placeholder="Search securities..."
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
                  onClick={() => setSelectedLetter(null)}
                  className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    selectedLetter === null
                      ? 'bg-nse-accent text-white'
                      : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                  }`}
                >
                  All
                </button>
                {availableLetters.map((letter) => (
                  <button
                    key={letter}
                    onClick={() => setSelectedLetter(letter)}
                    className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                      selectedLetter === letter
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

          {/* Securities Grid */}
          {filteredSecurities && (
            <div className="space-y-8">
              {securitiesData?.data.indices && securitiesData.data.indices.length > 0 && (
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
                    {(securitiesData?.data.equities[selectedLetter] || [])
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
                    {Object.entries(securitiesData?.data.equities || {})
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
          )}
        </div>
      </main>
    </div>
  );
}

// Security Card Component
function SecurityCard({ security, index }: { security: SecurityInfo; index: number }) {
  return (
    <Link 
      href={`/security/${security.symbol}`} 
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