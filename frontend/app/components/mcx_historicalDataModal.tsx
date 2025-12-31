'use client';

import React, { useState, useEffect } from 'react';
import { X, Loader2, TrendingUp, TrendingDown, BarChart3, Table2, ZoomIn } from 'lucide-react';
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  ReferenceArea,
  Brush
} from 'recharts';
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
  latestFuturesData?: any; // Latest futures data from parent component
  latestOptionData?: any; // Latest option data (CE or PE) from parent component
  optionsTimestamp?: string; // Timestamp from options analysis data
}

interface HistoricalDataPoint {
  DateDisplay: string;
  OpenInterest: number;
  ChangeInOI: number;
  Close: number;
  PreviousClose: number;
  timestamp: string;
  formattedDate: string;
  priceChange: number;
  high: number;
  low: number;
  open: number;
  volume: number;
  value: number;
}

type ViewType = 'table' | 'chart';

// Custom tooltip component defined outside render
const CustomTooltip = ({ active, payload, label }: any) => {
  const formatNumber = (value: number): string => {
    return value.toLocaleString('en-IN', { maximumFractionDigits: 2 });
  };

  if (active && payload && payload.length) {
    const uniquePayload = Array.from(
      new Map(payload.map((item: any) => [item.name, item])).values()
    );
    return (
      <div className="bg-slate-800 border border-gray-600 rounded-lg p-3 shadow-lg">
        <p className="text-gray-200 font-medium mb-2">{`Date: ${label}`}</p>
        {uniquePayload.map((entry: any, index: number) => (
          <p key={index} style={{ color: entry.color }} className="text-sm">
            {entry.name === 'openInterest' ? 'Open Interest' : 'Close Price'}:
            {entry.name === 'settlePrice' ? ' ₹' : ' '}
            {formatNumber(entry.value)}
          </p>
        ))}
      </div>
    );
  }
  return null;
};

export default function McxHistoricalDataModal({
  isOpen,
  onClose,
  symbol,
  expiry,
  dataType,
  optionType,
  strikePrice,
  latestFuturesData,
  latestOptionData,
  optionsTimestamp
}: McxHistoricalDataModalProps) {
  const [historicalData, setHistoricalData] = useState<HistoricalDataPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<ViewType>('table');
  const [refAreaLeft, setRefAreaLeft] = useState<string>('');
  const [refAreaRight, setRefAreaRight] = useState<string>('');
  const [dataRange, setDataRange] = useState<{ left?: string; right?: string }>({});

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

  const formatNumber = (value: number): string => {
    return value.toLocaleString('en-IN', { maximumFractionDigits: 2 });
  };

  const getChangeColor = (value: number): string => {
    if (value > 0) return 'text-green-400';
    if (value < 0) return 'text-red-400';
    return 'text-gray-400';
  };



  // Format data for chart - sorted by date from oldest to latest
  const chartData = historicalData?.map(item => ({
    date: new Date(item.timestamp).toLocaleDateString('en-IN', { 
      day: '2-digit', 
      month: 'short' 
    }),
    openInterest: item.OpenInterest,
    settlePrice: item.Close,
    fullDate: item.timestamp,
    sortDate: new Date(item.timestamp).getTime()
  })).sort((a, b) => a.sortDate - b.sortDate) || [];

  // Filter data based on zoom selection
  const getDisplayData = () => {
    if (!dataRange.left && !dataRange.right) return chartData;
    
    const leftIndex = dataRange.left ? chartData.findIndex(d => d.date === dataRange.left) : 0;
    const rightIndex = dataRange.right ? chartData.findIndex(d => d.date === dataRange.right) : chartData.length - 1;
    
    return chartData.slice(leftIndex, rightIndex + 1);
  };

  const displayData = getDisplayData();

  // Calculate dynamic Y-axis domains
  const getYAxisDomains = () => {
    if (displayData.length === 0) return { oiDomain: [0, 100], priceDomain: [0, 100] };
    
    const oiValues = displayData.map(d => d.openInterest);
    const priceValues = displayData.map(d => d.settlePrice);
    
    const oiMin = Math.min(...oiValues);
    const oiMax = Math.max(...oiValues);
    const oiPadding = (oiMax - oiMin) * 0.1;
    
    const priceMin = Math.min(...priceValues);
    const priceMax = Math.max(...priceValues);
    const pricePadding = (priceMax - priceMin) * 0.1;
    
    return {
      oiDomain: [Math.max(0, oiMin - oiPadding), oiMax + oiPadding],
      priceDomain: [priceMin - pricePadding, priceMax + pricePadding]
    };
  };

  const { oiDomain, priceDomain } = getYAxisDomains();

  const zoom = () => {
    if (refAreaLeft === refAreaRight || refAreaRight === '') {
      setRefAreaLeft('');
      setRefAreaRight('');
      return;
    }

    // Make sure left is before right
    const left = refAreaLeft < refAreaRight ? refAreaLeft : refAreaRight;
    const right = refAreaLeft < refAreaRight ? refAreaRight : refAreaLeft;

    setDataRange({ left, right });
    setRefAreaLeft('');
    setRefAreaRight('');
  };

  const zoomOut = () => {
    setDataRange({});
    setRefAreaLeft('');
    setRefAreaRight('');
  };

  // Fetch data when modal opens or parameters change
  useEffect(() => {
    // Fetch historical data
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
        
        // Build API parameters
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

        console.log('Fetching historical data with params:', params);
        
        const response = await mcxApiClient.getHistoricalData(params);
        
        if (response.success && response.data) {
          const rawData = response.data.d.Data;
          
          if (!rawData || !Array.isArray(rawData)) {
            setError('Invalid data format received from API');
            return;
          }

          const processedData: HistoricalDataPoint[] = rawData.map((item) => {
            const priceChange = item.Close - item.PreviousClose;
            
            return {
              DateDisplay: item.DateDisplay,
              OpenInterest: item.OpenInterest,
              ChangeInOI: item.ChangeInOI, // Now comes from backend
              Close: item.Close,
              PreviousClose: item.PreviousClose,
              timestamp: item.Date,
              formattedDate: item.DateDisplay,
              priceChange: priceChange,
              high: item.High,
              low: item.Low,
              open: item.Open,
              volume: item.Volume,
              value: item.Value
            };
          });

          // Helper function to normalize dates for comparison
          const normalizeDate = (dateStr: string): string => {
            try {
              const date = new Date(dateStr);
              if (isNaN(date.getTime())) return '';
              
              const year = date.getFullYear();
              const month = String(date.getMonth() + 1).padStart(2, '0');
              const day = String(date.getDate()).padStart(2, '0');
              return `${year}-${month}-${day}`;
            } catch {
              return '';
            }
          };

          // Append latest futures data if available and this is futures data
          if (dataType === 'futures' && latestFuturesData?.timestamp) {
            const latestDataPoint: HistoricalDataPoint = {
              DateDisplay: latestFuturesData.timestamp.split(' ')[0], // Extract date part only
              OpenInterest: latestFuturesData.openInterest,
              ChangeInOI: latestFuturesData.changeinOpenInterest,
              Close: latestFuturesData.LTP || latestFuturesData.lastPrice,
              PreviousClose: latestFuturesData.previousClose,
              timestamp: latestFuturesData.timestamp,
              formattedDate: latestFuturesData.timestamp.split(' ')[0],
              priceChange: latestFuturesData.absoluteChange,
              high: latestFuturesData.LTP || latestFuturesData.lastPrice, // Use LTP as high for latest data
              low: latestFuturesData.LTP || latestFuturesData.lastPrice, // Use LTP as low for latest data
              open: latestFuturesData.LTP || latestFuturesData.lastPrice, // Use LTP as open for latest data
              volume: 0, // Not available in latest data
              value: 0 // Not available in latest data
            };
            
            const latestDate = normalizeDate(latestDataPoint.timestamp);
            
            // Check if this date already exists in historical data
            const existsInHistory = processedData.some(item => {
              if (!item.timestamp && !item.DateDisplay) return false;
              const itemDate = normalizeDate(item.timestamp || item.DateDisplay);
              return itemDate === latestDate;
            });
            
            if (!existsInHistory && latestDate) {
              processedData.unshift(latestDataPoint); // Add to beginning (most recent)
            }
          }

          // Append latest option data if available and this is options data
          if (dataType === 'options' && latestOptionData && optionsTimestamp) {
            const latestDataPoint: HistoricalDataPoint = {
              DateDisplay: optionsTimestamp.split(' ')[0], // Extract date part only
              OpenInterest: latestOptionData.openInterest || 0,
              ChangeInOI: latestOptionData.changeinOpenInterest || 0,
              Close: latestOptionData.lastPrice || 0,
              PreviousClose: latestOptionData.lastPrice ? (latestOptionData.lastPrice - (latestOptionData.change || 0)) : 0,
              timestamp: optionsTimestamp,
              formattedDate: optionsTimestamp.split(' ')[0],
              priceChange: latestOptionData.change || 0,
              high: latestOptionData.lastPrice || 0, // Use lastPrice as high for latest data
              low: latestOptionData.lastPrice || 0, // Use lastPrice as low for latest data
              open: latestOptionData.lastPrice || 0, // Use lastPrice as open for latest data
              volume: 0, // Not available in latest data
              value: 0 // Not available in latest data
            };
            
            const latestDate = normalizeDate(latestDataPoint.timestamp);
            
            // Check if this date already exists in historical data
            const existsInHistory = processedData.some(item => {
              if (!item.timestamp && !item.DateDisplay) return false;
              const itemDate = normalizeDate(item.timestamp || item.DateDisplay);
              return itemDate === latestDate;
            });
            
            if (!existsInHistory && latestDate) {
              processedData.unshift(latestDataPoint); // Add to beginning (most recent)
            }
          }
          
          // Sort by timestamp (most recent first for table, but will be reversed for chart)
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

    if (isOpen && symbol && expiry) {
      fetchHistoricalData();
    }
  }, [isOpen, symbol, expiry, dataType, optionType, strikePrice, latestFuturesData, latestOptionData, optionsTimestamp]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      {/* Backdrop */}
      <div 
        className="fixed inset-0 bg-black/70 backdrop-blur-sm transition-opacity"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="flex min-h-full items-center justify-center p-4">
        <div className="relative w-full max-w-7xl bg-slate-900 rounded-xl shadow-2xl border border-gray-700/50">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-gray-700/50">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-4">
                <span className="text-2xl">{getMcxCommodityIcon(symbol)}</span>
                <div>
                  <h2 className="text-2xl font-bold text-gray-100">
                    {symbol} Historical Data
                  </h2>
                  <p className="text-sm text-gray-400 mt-1">
                    {dataType === 'futures' ? (
                      `Futures Contract - ${expiry}`
                    ) : (
                      `${optionType} Option - Strike ₹${strikePrice} - ${expiry}`
                    )}
                  </p>
                </div>
              </div>
              
              {/* View Toggle Tabs */}
              <div className="flex bg-slate-800 rounded-lg p-1 ml-8">
                <button
                  onClick={() => setActiveView('table')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                    activeView === 'table'
                      ? 'bg-nse-accent text-white shadow-sm'
                      : 'text-gray-400 hover:text-gray-300'
                  }`}
                >
                  <Table2 className="w-4 h-4" />
                  Table View
                </button>
                <button
                  onClick={() => setActiveView('chart')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                    activeView === 'chart'
                      ? 'bg-nse-accent text-white shadow-sm'
                      : 'text-gray-400 hover:text-gray-300'
                  }`}
                >
                  <BarChart3 className="w-4 h-4" />
                  Chart View
                </button>
              </div>
            </div>
            
            <button
              onClick={onClose}
              className="p-2 hover:bg-slate-800 rounded-lg transition-colors"
            >
              <X className="w-6 h-6 text-gray-400" />
            </button>
          </div>

          {/* Content */}
          <div className="p-6">
            {loading && (
              <div className="flex flex-col items-center justify-center py-12">
                <Loader2 className="w-12 h-12 animate-spin text-nse-accent mb-4" />
                <p className="text-gray-400">Loading historical data...</p>
              </div>
            )}

            {error && (
              <div className="bg-red-900/20 border border-red-500/50 rounded-lg p-4 text-red-300">
                <p className="font-medium">Error loading data</p>
                <p className="text-sm mt-1">{error}</p>
              </div>
            )}

            {!loading && !error && historicalData && historicalData.length === 0 && (
              <div className="text-center py-12">
                <p className="text-gray-400 text-lg">No historical data available for the selected period</p>
              </div>
            )}

            {!loading && !error && historicalData && historicalData.length > 0 && (
              <>
                {/* Table View */}
                {activeView === 'table' && (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b border-gray-700">
                          <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Date</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Open Interest</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Change in OI</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Settle Price</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Price Change</th>
                        </tr>
                      </thead>
                      <tbody>
                        {historicalData.map((row, index) => (
                          <tr 
                            key={index} 
                            className="border-b border-gray-700/50 hover:bg-slate-800/50 transition-colors"
                          >
                            <td className="px-4 py-3 text-sm text-gray-300">
                              {row.DateDisplay}
                            </td>
                            <td className="px-4 py-3 text-sm text-right text-gray-100">
                              {formatNumber(row.OpenInterest)}
                            </td>
                            <td className={`px-4 py-3 text-sm text-right font-medium ${getChangeColor(row.ChangeInOI)}`}>
                              <div className="flex items-center justify-end gap-1">
                                {row.ChangeInOI > 0 && <TrendingUp className="w-4 h-4" />}
                                {row.ChangeInOI < 0 && <TrendingDown className="w-4 h-4" />}
                                {row.ChangeInOI > 0 ? '+' : ''}
                                {formatNumber(row.ChangeInOI)}
                              </div>
                            </td>
                            <td className="px-4 py-3 text-sm text-right text-gray-100">
                              ₹{formatNumber(row.Close)}
                            </td>
                            <td className={`px-4 py-3 text-sm text-right font-medium ${getChangeColor(row.priceChange)}`}>
                              <div className="flex items-center justify-end gap-1">
                                {row.priceChange > 0 && <TrendingUp className="w-4 h-4" />}
                                {row.priceChange < 0 && <TrendingDown className="w-4 h-4" />}
                                {row.priceChange > 0 ? '+' : ''}
                                ₹{formatNumber(Math.abs(row.priceChange))}
                              </div>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}

                {/* Chart View */}
                {activeView === 'chart' && (
                  <div className="space-y-6">
                    {/* Chart Controls */}
                    <div className="flex items-center justify-between p-4 bg-slate-800/30 rounded-lg">
                      <div className="flex items-center gap-6">
                        <div className="flex items-center gap-2">
                          <div className="w-4 h-0.5 bg-blue-400"></div>
                          <span className="text-sm text-gray-300">Open Interest</span>
                          <span className="text-xs text-gray-500">(Left Axis)</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <div className="w-4 h-0.5 bg-emerald-400"></div>
                          <span className="text-sm text-gray-300">Close Price</span>
                          <span className="text-xs text-gray-500">(Right Axis)</span>
                        </div>
                      </div>
                      
                      {/* Zoom Controls */}
                      <div className="flex items-center gap-2">
                        <span className="text-xs text-gray-400">Click and drag to zoom</span>
                        {(dataRange.left || dataRange.right) && (
                          <button
                            onClick={zoomOut}
                            className="flex items-center gap-1 px-3 py-1 bg-slate-700 hover:bg-slate-600 rounded text-xs text-gray-300 transition-colors"
                          >
                            <ZoomIn className="w-3 h-3" />
                            Reset Zoom
                          </button>
                        )}
                      </div>
                    </div>

                    {/* Chart Container */}
                    <div className="h-96 w-full bg-slate-800/20 rounded-lg p-4">
                      <ResponsiveContainer width="100%" height="100%">
                        <LineChart 
                          data={displayData} 
                          margin={{ top: 20, right: 60, left: 60, bottom: 20 }}
                          onMouseDown={(e) => {
                            if (e && e.activeLabel) {
                              setRefAreaLeft(String(e.activeLabel));
                            }
                          }}
                          onMouseMove={(e) => {
                            if (refAreaLeft && e && e.activeLabel) {
                              setRefAreaRight(String(e.activeLabel));
                            }
                          }}
                          onMouseUp={zoom}
                        >
                          <CartesianGrid 
                            strokeDasharray="3 3" 
                            stroke="#374151" 
                            opacity={0.3}
                          />
                          <XAxis 
                            dataKey="date" 
                            axisLine={false}
                            tickLine={false}
                            tick={{ fontSize: 12, fill: '#9CA3AF' }}
                            angle={-45}
                            textAnchor="end"
                            height={60}
                          />
                          <YAxis 
                            yAxisId="left"
                            orientation="left"
                            axisLine={false}
                            tickLine={false}
                            tick={{ fontSize: 12, fill: '#60A5FA' }}
                            tickFormatter={(value) => formatNumber(value)}
                            domain={oiDomain}
                          />
                          <YAxis 
                            yAxisId="right"
                            orientation="right"
                            axisLine={false}
                            tickLine={false}
                            tick={{ fontSize: 12, fill: '#34D399' }}
                            tickFormatter={(value) => `₹${formatNumber(value)}`}
                            domain={priceDomain}
                          />
                          <Tooltip content={CustomTooltip} />
                          
                          {/* Reference Area for Zoom Selection */}
                          {refAreaLeft && refAreaRight && (
                            <ReferenceArea 
                              yAxisId="left" 
                              x1={refAreaLeft} 
                              x2={refAreaRight} 
                              strokeOpacity={0.3} 
                              fillOpacity={0.3}
                              fill="#8884d8"
                            />
                          )}
                          
                          <Line 
                            yAxisId="left"
                            type="monotone" 
                            dataKey="openInterest" 
                            stroke="#60A5FA"
                            strokeWidth={2.5}
                            dot={{ fill: '#60A5FA', strokeWidth: 0, r: 3 }}
                            activeDot={{ r: 5, stroke: '#60A5FA', strokeWidth: 2, fill: '#1E293B' }}
                          />
                          <Line 
                            yAxisId="right"
                            type="monotone" 
                            dataKey="settlePrice" 
                            stroke="#34D399"
                            strokeWidth={2.5}
                            dot={{ fill: '#34D399', strokeWidth: 0, r: 3 }}
                            activeDot={{ r: 5, stroke: '#34D399', strokeWidth: 2, fill: '#1E293B' }}
                          />
                        </LineChart>
                      </ResponsiveContainer>
                    </div>

                    {/* Brush for Timeline Navigation */}
                    {chartData.length > 20 && (
                      <div className="h-16 w-full bg-slate-800/10 rounded-lg p-2">
                        <ResponsiveContainer width="100%" height="100%">
                          <LineChart data={chartData}>
                            <XAxis 
                              dataKey="date" 
                              axisLine={false}
                              tickLine={false}
                              tick={false}
                            />
                            <Line 
                              type="monotone" 
                              dataKey="settlePrice" 
                              stroke="#34D399"
                              strokeWidth={1}
                              dot={false}
                            />
                            <Brush 
                              dataKey="date"
                              height={30}
                              stroke="#60A5FA"
                              fill="#1E293B"
                              onChange={(brushData) => {
                                if (brushData && chartData[brushData.startIndex] && chartData[brushData.endIndex]) {
                                  setDataRange({
                                    left: chartData[brushData.startIndex].date,
                                    right: chartData[brushData.endIndex].date
                                  });
                                }
                              }}
                            />
                          </LineChart>
                        </ResponsiveContainer>
                      </div>
                    )}

                    {/* Chart Insights */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
                      {/* <div className="bg-slate-800/40 rounded-lg p-4 border border-slate-700/50">
                        <h4 className="text-sm font-semibold text-gray-300 mb-2">Open Interest Trend</h4>
                        <div className="flex items-center gap-2">
                          {displayData.length > 1 && (
                            <>
                              {displayData[displayData.length - 1].openInterest > displayData[0].openInterest ? (
                                <>
                                  <TrendingUp className="w-4 h-4 text-green-400" />
                                  <span className="text-green-400 text-sm font-medium">Increasing</span>
                                </>
                              ) : (
                                <>
                                  <TrendingDown className="w-4 h-4 text-red-400" />
                                  <span className="text-red-400 text-sm font-medium">Decreasing</span>
                                </>
                              )}
                            </>
                          )}
                        </div>
                      </div>
                      
                      <div className="bg-slate-800/40 rounded-lg p-4 border border-slate-700/50">
                        <h4 className="text-sm font-semibold text-gray-300 mb-2">Price Trend</h4>
                        <div className="flex items-center gap-2">
                          {displayData.length > 1 && (
                            <>
                              {displayData[displayData.length - 1].settlePrice > displayData[0].settlePrice ? (
                                <>
                                  <TrendingUp className="w-4 h-4 text-green-400" />
                                  <span className="text-green-400 text-sm font-medium">Increasing</span>
                                </>
                              ) : (
                                <>
                                  <TrendingDown className="w-4 h-4 text-red-400" />
                                  <span className="text-red-400 text-sm font-medium">Decreasing</span>
                                </>
                              )}
                            </>
                          )}
                        </div>
                      </div> */}

                      <div className="bg-slate-800/40 rounded-lg p-4 border border-slate-700/50">
                        <h4 className="text-sm font-semibold text-gray-300 mb-2">Data Points</h4>
                        <div className="flex items-center gap-2">
                          <span className="text-nse-accent text-xl font-bold">{displayData.length}</span>
                          <span className="text-gray-400 text-sm">records</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {/* Summary Stats - Only show in table view */}
                {activeView === 'table' && (
                  <div className="mt-6 grid grid-cols-2 md:grid-cols-5 gap-4">
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Total Records</p>
                      <p className="text-xl font-bold text-gray-100">{historicalData.length}</p>
                    </div>
                    {/* <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Avg OI</p>
                      <p className="text-xl font-bold text-gray-100">
                        {formatNumber(historicalData.reduce((sum, d) => sum + d.OpenInterest, 0) / historicalData.length)}
                      </p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Total OI Change</p>
                      <p className={`text-xl font-bold ${getChangeColor(historicalData.reduce((sum, d) => sum + d.ChangeInOI, 0))}`}>
                        {formatNumber(historicalData.reduce((sum, d) => sum + d.ChangeInOI, 0))}
                      </p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Avg Close Price</p>
                      <p className="text-xl font-bold text-gray-100">
                        ₹{formatNumber(historicalData.reduce((sum, d) => sum + d.Close, 0) / historicalData.length)}
                      </p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Avg Price Change</p>
                      <p className={`text-xl font-bold ${getChangeColor(historicalData.reduce((sum, d) => sum + d.priceChange, 0))}`}>
                        ₹{formatNumber(Math.abs(historicalData.reduce((sum, d) => sum + d.priceChange, 0) / historicalData.length))}
                      </p>
                    </div> */}
                  </div>
                )}
              </>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700">
            <button
              onClick={onClose}
              className="px-6 py-2 bg-slate-700 hover:bg-slate-600 text-gray-100 rounded-lg transition-colors font-medium"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}