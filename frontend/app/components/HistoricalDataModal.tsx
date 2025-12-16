'use client';

import React from 'react';
import { X, Loader2, TrendingUp, TrendingDown } from 'lucide-react';
import { HistoricalDataPoint } from '@/app/types/api';

interface HistoricalDataModalProps {
  isOpen: boolean;
  onClose: () => void;
  data: HistoricalDataPoint[] | null;
  loading: boolean;
  error: string | null;
  title: string;
}

export default function HistoricalDataModal({
  isOpen,
  onClose,
  data,
  loading,
  error,
  title
}: HistoricalDataModalProps) {
  if (!isOpen) return null;

  const formatNumber = (value: number): string => {
    return value.toLocaleString('en-IN', { maximumFractionDigits: 2 });
  };

  const getOIChangeColor = (value: number): string => {
    if (value > 0) return 'text-green-400';
    if (value < 0) return 'text-red-400';
    return 'text-gray-400';
  };

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      {/* Backdrop */}
      <div 
        className="fixed inset-0 bg-black/70 backdrop-blur-sm transition-opacity"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="flex min-h-full items-center justify-center p-4">
        <div className="relative w-full max-w-6xl bg-slate-900 rounded-lg shadow-2xl border border-gray-700">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-gray-700">
            <h2 className="text-2xl font-bold text-gray-100">
              {title}
            </h2>
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
                          {formatNumber(row.FH_OPEN_INT)}
                        </td>
                        <td className={`px-4 py-3 text-sm text-right font-medium ${getOIChangeColor(row.FH_CHANGE_IN_OI)}`}>
                          <div className="flex items-center justify-end gap-1">
                            {row.FH_CHANGE_IN_OI > 0 && <TrendingUp className="w-4 h-4" />}
                            {row.FH_CHANGE_IN_OI < 0 && <TrendingDown className="w-4 h-4" />}
                            {row.FH_CHANGE_IN_OI > 0 ? '+' : ''}
                            {formatNumber(row.FH_CHANGE_IN_OI)}
                          </div>
                        </td>
                        <td className="px-4 py-3 text-sm text-right text-gray-100">
                          ₹{formatNumber(row.FH_SETTLE_PRICE)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>

                {/* Summary Stats */}
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
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-700">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-gray-100 rounded-lg transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}