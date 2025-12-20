'use client';

import React, { useState } from 'react';
import { X, Loader2, TrendingUp, TrendingDown, BarChart3, Table2 } from 'lucide-react';
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer 
} from 'recharts';
import { HistoricalDataPoint } from '@/app/types/api_nse_type';

interface HistoricalDataModalProps {
  isOpen: boolean;
  onClose: () => void;
  data: HistoricalDataPoint[] | null;
  loading: boolean;
  error: string | null;
  title: string;
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
          {entry.name === 'openInterest' ? 'Open Interest' : 'Settle Price'}:
          {entry.name === 'settlePrice' ? ' ₹' : ' '}
          {formatNumber(entry.value)}
        </p>
      ))}
    </div>
  );
}
return null;
};

export default function HistoricalDataModal({
  isOpen,
  onClose,
  data,
  loading,
  error,
  title
}: HistoricalDataModalProps) {
  const [activeView, setActiveView] = useState<ViewType>('table');

  if (!isOpen) return null;

  const formatNumber = (value: number): string => {
    return value.toLocaleString('en-IN', { maximumFractionDigits: 2 });
  };

  const getOIChangeColor = (value: number): string => {
    if (value > 0) return 'text-green-400';
    if (value < 0) return 'text-red-400';
    return 'text-gray-400';
  };

  // Format data for chart - sorted by date from oldest to latest
  const chartData = data?.map(item => ({
    date: new Date(item.FH_TIMESTAMP).toLocaleDateString('en-IN', { 
      day: '2-digit', 
      month: 'short' 
    }),
    openInterest: item.FH_OPEN_INT / (item.FH_MARKET_LOT || 1), // Normalize by market lot
    settlePrice: item.FH_SETTLE_PRICE,
    fullDate: item.FH_TIMESTAMP,
    sortDate: new Date(item.FH_TIMESTAMP).getTime() // For sorting
  })).sort((a, b) => a.sortDate - b.sortDate) || [];

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
              <h2 className="text-2xl font-bold text-gray-100">
                {title}
              </h2>
              
              {/* View Toggle Tabs */}
              <div className="flex bg-slate-800 rounded-lg p-1">
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

            {!loading && !error && data && data.length === 0 && (
              <div className="text-center py-12">
                <p className="text-gray-400 text-lg">No historical data available for the selected period</p>
              </div>
            )}

            {!loading && !error && data && data.length > 0 && (
              <>
                {/* Table View */}
                {activeView === 'table' && (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b border-gray-700">
                          <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Date</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Underlying Value</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Open Interest</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Change in OI</th>
                          <th className="px-4 py-3 text-right text-sm font-semibold text-gray-300">Settle Price</th>
                        </tr>
                      </thead>
                      <tbody>
                        {data.map((row, index) => (
                          <tr 
                            key={index} 
                            className="border-b border-gray-700/50 hover:bg-slate-800/50 transition-colors"
                          >
                            <td className="px-4 py-3 text-sm text-gray-300">
                              {row.FH_TIMESTAMP}
                            </td>
                            <td className="px-4 py-3 text-sm text-right text-gray-100 font-medium">
                              ₹{formatNumber(row.FH_UNDERLYING_VALUE)}
                            </td>
                            <td className="px-4 py-3 text-sm text-right text-gray-100">
                              {formatNumber(row.FH_OPEN_INT / (row.FH_MARKET_LOT || 1))}
                            </td>
                            <td className={`px-4 py-3 text-sm text-right font-medium ${getOIChangeColor(row.FH_CHANGE_IN_OI)}`}>
                              <div className="flex items-center justify-end gap-1">
                                {row.FH_CHANGE_IN_OI > 0 && <TrendingUp className="w-4 h-4" />}
                                {row.FH_CHANGE_IN_OI < 0 && <TrendingDown className="w-4 h-4" />}
                                {row.FH_CHANGE_IN_OI > 0 ? '+' : ''}
                                {formatNumber(row.FH_CHANGE_IN_OI / (row.FH_MARKET_LOT || 1))}
                              </div>
                            </td>
                            <td className="px-4 py-3 text-sm text-right text-gray-100">
                              ₹{formatNumber(row.FH_SETTLE_PRICE)}
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
                    {/* Chart Legend */}
                    <div className="flex items-center justify-center gap-6 p-4 bg-slate-800/30 rounded-lg">
                      <div className="flex items-center gap-2">
                        <div className="w-4 h-0.5 bg-blue-400"></div>
                        <span className="text-sm text-gray-300">Open Interest</span>
                        <span className="text-xs text-gray-500">(Left Axis)</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-4 h-0.5 bg-emerald-400"></div>
                        <span className="text-sm text-gray-300">Settle Price</span>
                        <span className="text-xs text-gray-500">(Right Axis)</span>
                      </div>
                    </div>

                    {/* Chart Container */}
                    <div className="h-96 w-full bg-slate-800/20 rounded-lg p-4">
                      <ResponsiveContainer width="100%" height="100%">
                        <LineChart data={chartData} margin={{ top: 20, right: 60, left: 60, bottom: 20 }}>
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
                          />
                          <YAxis 
                            yAxisId="right"
                            orientation="right"
                            axisLine={false}
                            tickLine={false}
                            tick={{ fontSize: 12, fill: '#34D399' }}
                            tickFormatter={(value) => `₹${formatNumber(value)}`}
                          />
                          <Tooltip content={CustomTooltip} />
                          <Line 
                            yAxisId="left"
                            type="monotone" 
                            dataKey="openInterest" 
                            stroke="#60A5FA"
                            strokeWidth={2.5}
                            dot={{ fill: '#60A5FA', strokeWidth: 0, r: 4 }}
                            activeDot={{ r: 6, stroke: '#60A5FA', strokeWidth: 2, fill: '#1E293B' }}
                          />
                          <Line 
                            yAxisId="right"
                            type="monotone" 
                            dataKey="settlePrice" 
                            stroke="#34D399"
                            strokeWidth={2.5}
                            dot={{ fill: '#34D399', strokeWidth: 0, r: 4 }}
                            activeDot={{ r: 6, stroke: '#34D399', strokeWidth: 2, fill: '#1E293B' }}
                          />
                        </LineChart>
                      </ResponsiveContainer>
                    </div>

                    {/* Chart Insights */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
                      <div className="bg-slate-800/40 rounded-lg p-4 border border-slate-700/50">
                        <h4 className="text-sm font-semibold text-gray-300 mb-2">Open Interest Trend</h4>
                        <div className="flex items-center gap-2">
                          {chartData.length > 1 && (
                            <>
                              {chartData[chartData.length - 1].openInterest > chartData[0].openInterest ? (
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
                          {chartData.length > 1 && (
                            <>
                              {chartData[chartData.length - 1].settlePrice > chartData[0].settlePrice ? (
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
                        <h4 className="text-sm font-semibold text-gray-300 mb-2">Data Points</h4>
                        <div className="flex items-center gap-2">
                          <span className="text-nse-accent text-xl font-bold">{chartData.length}</span>
                          <span className="text-gray-400 text-sm">records</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {/* Summary Stats - Only show in table view */}
                {activeView === 'table' && (
                  <div className="mt-6 grid grid-cols-2 md:grid-cols-4 gap-4">
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Total Records</p>
                      <p className="text-xl font-bold text-gray-100">{data.length}</p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Avg OI</p>
                      <p className="text-xl font-bold text-gray-100">
                        {formatNumber(data.reduce((sum, d) => sum + d.FH_OPEN_INT, 0) / data.length)}
                      </p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Total OI Change</p>
                      <p className={`text-xl font-bold ${getOIChangeColor(data.reduce((sum, d) => sum + d.FH_CHANGE_IN_OI, 0))}`}>
                        {formatNumber(data.reduce((sum, d) => sum + d.FH_CHANGE_IN_OI, 0))}
                      </p>
                    </div>
                    <div className="bg-slate-800/50 rounded-lg p-4">
                      <p className="text-sm text-gray-400 mb-1">Avg Settle Price</p>
                      <p className="text-xl font-bold text-gray-100">
                        ₹{formatNumber(data.reduce((sum, d) => sum + d.FH_SETTLE_PRICE, 0) / data.length)}
                      </p>
                    </div>
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