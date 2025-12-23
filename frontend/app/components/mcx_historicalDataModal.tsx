'use client';

import React, { useState, useEffect } from 'react';
import {
  X,
  Calendar,
  TrendingUp,
  // TrendingDown,
  Clock,
  Loader2,
  AlertCircle,
  BarChart3,
  Activity,
  DollarSign
} from 'lucide-react';
import { mcxApiClient, handleMcxApiError, getMcxCommodityIcon } from '@/app/lib/api_mcx';
// import { McxHistoricalDataResponse } from '@/app/types/api_mcx_type';

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
  const [chartData, setChartData] = useState<any[]>([]);

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

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return {
        formattedDate: date.toLocaleDateString('en-IN'),
        formattedTime: date.toLocaleTimeString('en-IN', { 
          hour: '2-digit', 
          minute: '2-digit' 
        })
      };
    } catch {
      return {
        formattedDate: timestamp,
        formattedTime: ''
      };
    }
  };

  const fetchHistoricalData = async () => {
    if (!symbol || !expiry) return;

    try {
      setLoading(true);
      setError(null);

      // Calculate date range (20 days back from today)
      const toDate = new Date();
      const fromDate = new Date();
      fromDate.setDate(fromDate.getDate() - 20);

      const params = {
        symbol,
        expiry: formatExpiryForAPI(expiry),
        from_date: formatDateForAPI(fromDate),
        to_date: formatDateForAPI(toDate)
      };

      const response = await mcxApiClient.getHistoricalData(params);

      if (response.success && response.data) {
        // Process the historical data
        const processedData: HistoricalDataPoint[] = response.data.data.map(item => {
          const { formattedDate, formattedTime } = formatTimestamp(item.FH_TIMESTAMP);
          
          return {
            timestamp: item.FH_TIMESTAMP,
            underlyingValue: item.FH_UNDERLYING_VALUE,
            openInterest: item.FH_OPEN_INT,
            changeInOI: item.FH_CHANGE_IN_OI,
            settlePrice: item.FH_SETTLE_PRICE,
            strikePrice: item.FH_STRIKE_PRICE,
            optionType: item.FH_OPTION_TYPE,
            formattedDate,
            formattedTime
          };
        });

        // Sort by timestamp (most recent first)
        processedData.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

        setHistoricalData(processedData);
        setChartData(prepareChartData(processedData));
      } else {
        setError(response.error || 'Failed to fetch historical data');
      }
    } catch (err) {
      setError(handleMcxApiError(err));
    } finally {
      setLoading(false);
    }
  };

  const prepareChartData = (data: HistoricalDataPoint[]) => {
    return data
      .slice()
      .reverse() // Reverse for chronological order in chart
      .map((item, index) => ({
        index: index + 1,
        date: item.formattedDate,
        time: item.formattedTime,
        underlyingValue: item.underlyingValue,
        openInterest: item.openInterest,
        settlePrice: item.settlePrice,
        changeInOI: item.changeInOI
      }));
  };

  useEffect(() => {
    if (isOpen) {
      fetchHistoricalData();
    }
  }, [isOpen, symbol, expiry, dataType, optionType, strikePrice]);

  // Calculate summary statistics
  const summaryStats = React.useMemo(() => {
    if (historicalData.length === 0) return null;

    const latest = historicalData[0];
    const oldest = historicalData[historicalData.length - 1];
    
    const priceChange = latest.settlePrice - oldest.settlePrice;
    const priceChangePercent = ((priceChange / oldest.settlePrice) * 100);
    
    const oiChange = latest.openInterest - oldest.openInterest;
    const oiChangePercent = oldest.openInterest > 0 ? ((oiChange / oldest.openInterest) * 100) : 0;

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
                Last 20 days data
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
                <BarChart3 className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-300 mb-2">No Data Available</h3>
                <p className="text-gray-500">
                  No historical data found for this {dataType} contract.
                </p>
              </div>
            </div>
          ) : (
            <div className="space-y-6">
              {/* Summary Statistics */}
              {summaryStats && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="card-glow rounded-lg p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <DollarSign className="w-4 h-4 text-nse-accent" />
                      <span className="text-sm text-gray-400">Latest Price</span>
                    </div>
                    <p className="text-xl font-bold text-gray-100">
                      ₹{summaryStats.latest.settlePrice.toFixed(2)}
                    </p>
                    <p className={`text-sm ${summaryStats.priceChange >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {summaryStats.priceChange >= 0 ? '+' : ''}₹{summaryStats.priceChange.toFixed(2)} 
                      ({summaryStats.priceChangePercent >= 0 ? '+' : ''}{summaryStats.priceChangePercent.toFixed(2)}%)
                    </p>
                  </div>

                  <div className="card-glow rounded-lg p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <Activity className="w-4 h-4 text-nse-accent" />
                      <span className="text-sm text-gray-400">Open Interest</span>
                    </div>
                    <p className="text-xl font-bold text-gray-100">
                      {summaryStats.latest.openInterest.toLocaleString('en-IN')}
                    </p>
                    <p className={`text-sm ${summaryStats.oiChange >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {summaryStats.oiChange >= 0 ? '+' : ''}{summaryStats.oiChange.toLocaleString('en-IN')} 
                      ({summaryStats.oiChangePercent >= 0 ? '+' : ''}{summaryStats.oiChangePercent.toFixed(2)}%)
                    </p>
                  </div>

                  <div className="card-glow rounded-lg p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <BarChart3 className="w-4 h-4 text-nse-accent" />
                      <span className="text-sm text-gray-400">Avg Price (20D)</span>
                    </div>
                    <p className="text-xl font-bold text-gray-100">
                      ₹{summaryStats.avgPrice.toFixed(2)}
                    </p>
                  </div>

                  <div className="card-glow rounded-lg p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <TrendingUp className="w-4 h-4 text-nse-accent" />
                      <span className="text-sm text-gray-400">Avg OI (20D)</span>
                    </div>
                    <p className="text-xl font-bold text-gray-100">
                      {summaryStats.avgOI.toLocaleString('en-IN')}
                    </p>
                  </div>
                </div>
              )}

              {/* Simple Chart Visualization */}
              <div className="card-glow rounded-lg p-6">
                <h3 className="text-lg font-semibold text-gray-100 mb-4 flex items-center">
                  <BarChart3 className="w-5 h-5 mr-2 text-nse-accent" />
                  Price & Open Interest Trend
                </h3>
                
                {/* Mini Chart using CSS */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  {/* Price Chart */}
                  <div>
                    <h4 className="text-md font-medium text-gray-200 mb-3">Settlement Price</h4>
                    <div className="h-32 flex items-end gap-1 bg-slate-800/50 rounded p-3">
                      {chartData.slice(-10).map((item, index) => {
                        const maxPrice = Math.max(...chartData.map(d => d.settlePrice));
                        const height = (item.settlePrice / maxPrice) * 100;
                        return (
                          <div
                            key={index}
                            className="flex-1 bg-nse-accent opacity-70 hover:opacity-100 transition-opacity rounded-t"
                            style={{ height: `${height}%` }}
                            title={`${item.date}: ₹${item.settlePrice}`}
                          />
                        );
                      })}
                    </div>
                    <div className="text-xs text-gray-500 mt-2 text-center">
                      Last 10 days
                    </div>
                  </div>

                  {/* OI Chart */}
                  <div>
                    <h4 className="text-md font-medium text-gray-200 mb-3">Open Interest</h4>
                    <div className="h-32 flex items-end gap-1 bg-slate-800/50 rounded p-3">
                      {chartData.slice(-10).map((item, index) => {
                        const maxOI = Math.max(...chartData.map(d => d.openInterest));
                        const height = maxOI > 0 ? (item.openInterest / maxOI) * 100 : 0;
                        return (
                          <div
                            key={index}
                            className="flex-1 bg-blue-500 opacity-70 hover:opacity-100 transition-opacity rounded-t"
                            style={{ height: `${height}%` }}
                            title={`${item.date}: ${item.openInterest.toLocaleString('en-IN')}`}
                          />
                        );
                      })}
                    </div>
                    <div className="text-xs text-gray-500 mt-2 text-center">
                      Last 10 days
                    </div>
                  </div>
                </div>
              </div>

              {/* Historical Data Table */}
              <div className="card-glow rounded-lg overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-700">
                  <h3 className="text-lg font-semibold text-gray-100 flex items-center">
                    <Calendar className="w-5 h-5 mr-2 text-nse-accent" />
                    Historical Data ({historicalData.length} records)
                  </h3>
                </div>
                
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead className="bg-slate-800">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                          Date & Time
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                          Settlement Price
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                          Underlying Value
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                          Open Interest
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                          OI Change
                        </th>
                        {dataType === 'options' && (
                          <>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                              Strike Price
                            </th>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                              Option Type
                            </th>
                          </>
                        )}
                      </tr>
                    </thead>
                    <tbody className="bg-slate-900 divide-y divide-gray-700">
                      {historicalData.map((item, index) => (
                        <tr key={index} className="hover:bg-slate-800/50 transition-colors">
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="flex flex-col">
                              <span className="text-sm font-medium text-gray-100">
                                {item.formattedDate}
                              </span>
                              <span className="text-xs text-gray-400">
                                {item.formattedTime}
                              </span>
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm font-medium text-gray-100">
                              ₹{item.settlePrice.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm text-gray-300">
                              ₹{item.underlyingValue.toFixed(2)}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className="text-sm text-gray-300">
                              {item.openInterest.toLocaleString('en-IN')}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`text-sm ${
                              item.changeInOI > 0 ? 'text-green-400' : 
                              item.changeInOI < 0 ? 'text-red-400' : 'text-gray-400'
                            }`}>
                              {item.changeInOI > 0 ? '+' : ''}{item.changeInOI.toLocaleString('en-IN')}
                            </span>
                          </td>
                          {dataType === 'options' && (
                            <>
                              <td className="px-6 py-4 whitespace-nowrap">
                                <span className="text-sm text-gray-300">
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
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-between items-center p-6 border-t border-gray-700 bg-slate-800/50">
          <div className="flex items-center gap-2 text-sm text-gray-400">
            <Clock className="w-4 h-4" />
            Data refreshed automatically
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