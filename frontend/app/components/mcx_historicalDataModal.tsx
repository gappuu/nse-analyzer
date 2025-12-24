'use client';

import React, { useState, useEffect, useMemo } from 'react';
import {
  X,
  Clock,
  TrendingUp,
  // TrendingDown,
  Activity,
  Loader2,
  AlertCircle,
  BarChart,
  // Minus
} from 'lucide-react';
import { mcxApiClient, getMcxCommodityIcon, handleMcxApiError } from '@/app/lib/api_mcx';
import { McxHistoricalDataParams } from '@/app/types/api_mcx_type';

interface McxHistoricalDataModalProps {
  isOpen: boolean;
  onClose: () => void;
  symbol: string;
  expiry: string;
  dataType: 'futures' | 'options';
  optionType?: 'CE' | 'PE';
  strikePrice?: number;
}

interface HistoricalDataPoint {
  timestamp: string;
  underlyingValue: number;
  openInterest: number;
  changeInOI: number;
  settlePrice: number;
  strikePrice?: number;
  optionType?: string;
  formattedDate: string;
  formattedTime: string;
  high: number;
  low: number;
  open: number;
  volume: number;
  value: number;
  previousClose: number;
}

export default function McxHistoricalDataModal({
  isOpen,
  onClose,
  symbol,
  expiry,
  dataType,
  optionType,
  strikePrice
}: McxHistoricalDataModalProps) {
  const [historicalData, setHistoricalData] = useState<HistoricalDataPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Helper functions for date formatting
  const formatDateForAPI = (date: Date): string => {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}${month}${day}`;
  };

  const formatExpiryForAPI = (expiryString: string): string => {
    try {
      // Parse expiry format like "23-Dec-2025" to "23DEC2025"
      const parts = expiryString.split('-');
      if (parts.length !== 3) return expiryString;
      
      const day = parts[0].padStart(2, '0');
      const month = parts[1].toUpperCase();
      const year = parts[2];
      
      return `${day}${month}${year}`;
    } catch {
      return expiryString;
    }
  };

  const formatTimestamp = (dateString: string) => {
    try {
      // Parse date format like "12/22/2025"
      const [month, day, year] = dateString.split('/');
      const date = new Date(parseInt(year), parseInt(month) - 1, parseInt(day));
      
      return {
        formattedDate: date.toLocaleDateString('en-IN'),
        formattedTime: date.toLocaleTimeString('en-IN', { 
          hour: '2-digit', 
          minute: '2-digit' 
        })
      };
    } catch {
      return {
        formattedDate: 'Invalid Date',
        formattedTime: 'Invalid Time'
      };
    }
  };

  // Calculate change in OI from current and previous day data
  const calculateChangeInOI = (data: any[], currentIndex: number): number => {
    if (currentIndex >= data.length - 1) return 0;
    const current = data[currentIndex];
    const previous = data[currentIndex + 1];
    return current.OpenInterest - previous.OpenInterest;
  };

  // Fetch historical data with corrected API response handling
  const fetchHistoricalData = async () => {
    if (!symbol || !expiry) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // Calculate date range (20 days back)
      const toDate = new Date();
      const fromDate = new Date();
      fromDate.setDate(toDate.getDate() - 20);
      
      // Format expiry for API
      const formattedExpiry = formatExpiryForAPI(expiry);
      
      // Build API parameters with new required fields
      const params: McxHistoricalDataParams = {
        symbol,
        expiry: formattedExpiry,
        from_date: formatDateForAPI(fromDate),
        to_date: formatDateForAPI(toDate),
        instrument_name: dataType === 'futures' ? 'FUTCOM' : 'OPTFUT'
      };

      // Add option-specific parameters if it's an option
      if (dataType === 'options') {
        if (!optionType || strikePrice === undefined) {
          setError('Option type and strike price are required for options data');
          return;
        }
        params.option_type = optionType;
        params.strike_price = strikePrice.toString();
      }

      // console.log('Fetching historical data with params:', params);
      
      const response = await mcxApiClient.getHistoricalData(params);
      
      if (response.success && response.data) {
        // Access the correct data structure: response.data.d.Data
        const rawData = response.data.d.Data;
        
        if (!rawData || !Array.isArray(rawData)) {
          setError('Invalid data format received from API');
          return;
        }

        const processedData: HistoricalDataPoint[] = rawData.map((item, index) => {
          const timeFormatting = formatTimestamp(item.Date);
          const changeInOI = calculateChangeInOI(rawData, index);
          
          return {
            timestamp: item.Date,
            underlyingValue: item.Close,
            openInterest: item.OpenInterest,
            changeInOI: changeInOI,
            settlePrice: item.Close,
            strikePrice: item.StrikePrice > 0 ? item.StrikePrice : undefined,
            optionType: item.OptionType !== '-' ? item.OptionType : undefined,
            formattedDate: timeFormatting.formattedDate,
            formattedTime: timeFormatting.formattedTime,
            high: item.High,
            low: item.Low,
            open: item.Open,
            volume: item.Volume,
            value: item.Value,
            previousClose: item.PreviousClose
          };
        });
        
        // Sort by timestamp (most recent first)
        processedData.sort((a, b) => {
          const dateA = new Date(a.timestamp);
          const dateB = new Date(b.timestamp);
          return dateB.getTime() - dateA.getTime();
        });
        
        setHistoricalData(processedData);
      } else {
        setError(response.error || 'Failed to fetch historical data');
      }
    } catch (err) {
      setError(handleMcxApiError(err));
      console.error('Error fetching historical data:', err);
    } finally {
      setLoading(false);
    }
  };

  // Fetch data when modal opens or parameters change
  useEffect(() => {
    if (isOpen && symbol && expiry) {
      fetchHistoricalData();
    }
  }, [isOpen, symbol, expiry, dataType, optionType, strikePrice]);

  // Calculate summary statistics
  const summaryStats = useMemo(() => {
    if (historicalData.length === 0) {
      return {
        priceChange: 0,
        priceChangePercent: 0,
        oiChange: 0,
        oiChangePercent: 0,
        avgPrice: 0,
        avgOI: 0,
        latest: null,
        oldest: null
      };
    }

    const latest = historicalData[0];
    const oldest = historicalData[historicalData.length - 1];
    
    const priceChange = latest.settlePrice - oldest.settlePrice;
    const priceChangePercent = oldest.settlePrice ? ((priceChange / oldest.settlePrice) * 100) : 0;
    
    const oiChange = latest.openInterest - oldest.openInterest;
    const oiChangePercent = oldest.openInterest ? ((oiChange / oldest.openInterest) * 100) : 0;

    const avgPrice = historicalData.reduce((sum, item) => sum + item.settlePrice, 0) / historicalData.length;
    const avgOI = historicalData.reduce((sum, item) => sum + item.openInterest, 0) / historicalData.length;

    return {
      priceChange,
      priceChangePercent,
      oiChange,
      oiChangePercent,
      avgPrice,
      avgOI,
      latest,
      oldest
    };
  }, [historicalData]);

  // Create mini chart data
  const createMiniChart = (values: number[], type: 'price' | 'oi') => {
    if (values.length === 0) return [];
    
    const max = Math.max(...values);
    const min = Math.min(...values);
    const range = max - min;
    
    return values.reverse().map((value, index) => {
      const normalizedHeight = range > 0 ? ((value - min) / range) * 100 : 50;
      const isIncreasing = index === 0 ? false : value >= values[index - 1];
      
      return {
        height: normalizedHeight,
        color: type === 'price' 
          ? (isIncreasing ? '#10b981' : '#ef4444')
          : (isIncreasing ? '#3b82f6' : '#f59e0b')
      };
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black bg-opacity-50"
        onClick={onClose}
      />
      
      {/* Modal */}
      <div className="relative bg-slate-900 rounded-lg shadow-2xl w-full max-w-7xl max-h-[90vh] overflow-hidden mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <div className="flex items-center gap-4">
            <span className="text-2xl">{getMcxCommodityIcon(symbol)}</span>
            <div>
              <h2 className="text-xl font-bold text-gray-100">
                {symbol} Historical Data
              </h2>
              <p className="text-sm text-gray-400 mt-1">
                {dataType === 'futures' ? (
                  `Futures Contract - ${expiry}`
                ) : (
                  `${optionType} Option - Strike ₹${strikePrice} - ${expiry}`
                )}
              </p>
              <p className="text-xs text-gray-500 mt-1">
                Last 20 days data • Instrument: {dataType === 'futures' ? 'FUTCOM' : 'OPTFUT'}
              </p>
            </div>
          </div>
          
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-200 transition-colors p-2"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-100px)]">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                <Loader2 className="w-8 h-8 animate-spin mx-auto text-nse-accent mb-4" />
                <p className="text-gray-400">Loading historical data...</p>
                <p className="text-xs text-gray-500 mt-2">
                  Fetching {dataType} data for {symbol} ({expiry})
                  {dataType === 'options' && ` - ${optionType} ${strikePrice}`}
                </p>
              </div>
            </div>
          ) : error ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-center max-w-md">
                <AlertCircle className="w-12 h-12 text-red-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-100 mb-2">Error Loading Data</h3>
                <p className="text-gray-400 mb-4">{error}</p>
                <button
                  onClick={fetchHistoricalData}
                  className="btn-primary"
                >
                  Try Again
                </button>
              </div>
            </div>
          ) : historicalData.length === 0 ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                <BarChart className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-300 mb-2">No Data Available</h3>
                <p className="text-gray-500">
                  No historical data found for the specified parameters
                </p>
              </div>
            </div>
          ) : (
            <>
              {/* Summary Cards */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                <div className="card-glow rounded-lg p-6">
                  <div className="flex items-center gap-3 mb-2">
                    <TrendingUp className="w-5 h-5 text-nse-accent" />
                    <span className="text-sm text-gray-400">Latest Price</span>
                  </div>
                  <p className="text-2xl font-bold text-gray-100">
                    ₹{summaryStats.latest?.settlePrice.toFixed(2)}
                  </p>
                  <p className={`text-sm mt-1 ${summaryStats.priceChange >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {summaryStats.priceChange >= 0 ? '+' : ''}₹{summaryStats.priceChange.toFixed(2)} 
                    ({summaryStats.priceChangePercent >= 0 ? '+' : ''}{summaryStats.priceChangePercent.toFixed(2)}%)
                  </p>
                </div>

                <div className="card-glow rounded-lg p-6">
                  <div className="flex items-center gap-3 mb-2">
                    <BarChart className="w-5 h-5 text-blue-400" />
                    <span className="text-sm text-gray-400">Open Interest</span>
                  </div>
                  <p className="text-2xl font-bold text-gray-100">
                    {summaryStats.latest?.openInterest.toLocaleString('en-IN')}
                  </p>
                  <p className={`text-sm mt-1 ${summaryStats.oiChange >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {summaryStats.oiChange >= 0 ? '+' : ''}{summaryStats.oiChange.toLocaleString('en-IN')} 
                    ({summaryStats.oiChangePercent >= 0 ? '+' : ''}{summaryStats.oiChangePercent.toFixed(2)}%)
                  </p>
                </div>

                <div className="card-glow rounded-lg p-6">
                  <div className="flex items-center gap-3 mb-2">
                    <Activity className="w-5 h-5 text-purple-400" />
                    <span className="text-sm text-gray-400">Average Price</span>
                  </div>
                  <p className="text-2xl font-bold text-gray-100">
                    ₹{summaryStats.avgPrice.toFixed(2)}
                  </p>
                  <p className="text-sm text-gray-500 mt-1">20-day average</p>
                </div>

                <div className="card-glow rounded-lg p-6">
                  <div className="flex items-center gap-3 mb-2">
                    <BarChart className="w-5 h-5 text-orange-400" />
                    <span className="text-sm text-gray-400">Average OI</span>
                  </div>
                  <p className="text-2xl font-bold text-gray-100">
                    {summaryStats.avgOI.toLocaleString('en-IN')}
                  </p>
                  <p className="text-sm text-gray-500 mt-1">20-day average</p>
                </div>
              </div>

              {/* Mini Charts */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                <div className="card-glow rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-gray-100 mb-4 flex items-center gap-2">
                    <TrendingUp className="w-5 h-5" />
                    Price Trend (₹)
                  </h3>
                  <div className="flex items-end gap-1 h-20">
                    {createMiniChart(historicalData.map(d => d.settlePrice), 'price').map((bar, index) => (
                      <div
                        key={index}
                        className="flex-1 rounded-t transition-all hover:opacity-80"
                        style={{
                          height: `${bar.height}%`,
                          backgroundColor: bar.color,
                          minHeight: '4px'
                        }}
                      />
                    ))}
                  </div>
                </div>

                <div className="card-glow rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-gray-100 mb-4 flex items-center gap-2">
                    <BarChart className="w-5 h-5" />
                    Open Interest Trend
                  </h3>
                  <div className="flex items-end gap-1 h-20">
                    {createMiniChart(historicalData.map(d => d.openInterest), 'oi').map((bar, index) => (
                      <div
                        key={index}
                        className="flex-1 rounded-t transition-all hover:opacity-80"
                        style={{
                          height: `${bar.height}%`,
                          backgroundColor: bar.color,
                          minHeight: '4px'
                        }}
                      />
                    ))}
                  </div>
                </div>
              </div>

              {/* Data Table */}
              <div className="card-glow rounded-lg overflow-hidden">
                <div className="p-6 border-b border-gray-700">
                  <h3 className="text-lg font-semibold text-gray-100 flex items-center gap-2">
                    <Clock className="w-5 h-5" />
                    Historical Data ({historicalData.length} records)
                  </h3>
                </div>
                
                <div className="overflow-x-auto">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Date</th>
                        <th>Open</th>
                        <th>High</th>
                        <th>Low</th>
                        <th>Close</th>
                        <th>Open Interest</th>
                        <th>Change in OI</th>
                        <th>Volume</th>
                        {dataType === 'options' && (
                          <>
                            <th>Strike Price</th>
                            <th>Option Type</th>
                          </>
                        )}
                      </tr>
                    </thead>
                    <tbody>
                      {historicalData.map((item, index) => (
                        <tr key={index}>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm font-medium text-gray-100">
                              {item.formattedDate}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm text-gray-300">
                              ₹{item.open.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm font-medium text-green-400">
                              ₹{item.high.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm font-medium text-red-400">
                              ₹{item.low.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm font-medium text-gray-100">
                              ₹{item.settlePrice.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm text-gray-100">
                              {item.openInterest.toLocaleString('en-IN')}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`text-sm font-medium ${
                              item.changeInOI > 0 ? 'text-green-400' : 
                              item.changeInOI < 0 ? 'text-red-400' : 'text-gray-400'
                            }`}>
                              {item.changeInOI > 0 ? '+' : ''}{item.changeInOI.toLocaleString('en-IN')}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm text-gray-100">
                              {item.volume.toLocaleString('en-IN')}
                            </span>
                          </td>
                          {dataType === 'options' && (
                            <>
                              <td className="px-6 py-4 whitespace-nowrap">
                                <span className="text-sm text-gray-100">
                                  {item.strikePrice ? `₹${item.strikePrice}` : '-'}
                                </span>
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap">
                                <span className={`text-sm font-medium ${
                                  item.optionType === 'CE' ? 'text-green-400' : 
                                  item.optionType === 'PE' ? 'text-red-400' : 'text-gray-400'
                                }`}>
                                  {item.optionType || '-'}
                                </span>
                              </td>
                            </>
                          )}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-between items-center p-6 border-t border-gray-700 bg-slate-800/50">
          <div className="flex items-center gap-2 text-sm text-gray-400">
            <Clock className="w-4 h-4" />
            Data refreshed automatically
            <span className="text-xs text-gray-500 ml-2">
              • Instrument: {dataType === 'futures' ? 'FUTCOM' : 'OPTFUT'}
            </span>
          </div>
          <div className="flex gap-2">
            <button
              onClick={fetchHistoricalData}
              disabled={loading}
              className="btn-secondary inline-flex items-center text-sm"
            >
              {loading ? (
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Activity className="w-4 h-4 mr-2" />
              )}
              {loading ? 'Refreshing...' : 'Refresh Data'}
            </button>
            <button
              onClick={onClose}
              className="btn-outline-primary text-sm"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}