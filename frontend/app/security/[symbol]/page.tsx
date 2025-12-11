'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import {
  ArrowLeft,
  Calendar,
  TrendingUp,
  AlertCircle,
  Loader2,
  Clock,
  DollarSign,
  BarChart3,
} from 'lucide-react';
import { apiClient, handleApiError, getAlertBadgeClass, formatCurrency } from '@/app/lib/api';
import { ContractInfoResponse, SingleAnalysisResponse } from '@/app/types/api';

export default function SecurityPage() {
  const params = useParams();
  const symbol = params.symbol as string;
  
  const [contractInfo, setContractInfo] = useState<ContractInfoResponse | null>(null);
  const [selectedExpiry, setSelectedExpiry] = useState<string | null>(null);
  const [analysisData, setAnalysisData] = useState<SingleAnalysisResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [analysisLoading, setAnalysisLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchContractInfo = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const response = await apiClient.getContractInfo(symbol);
        
        if (response.success && response.data) {
          setContractInfo(response.data);
          // Auto-select first expiry
          if (response.data.expiry_dates.length > 0) {
            setSelectedExpiry(response.data.expiry_dates[0]);
          }
        } else {
          setError(response.error || 'Failed to fetch contract info');
        }
      } catch (err) {
        setError(handleApiError(err));
      } finally {
        setLoading(false);
      }
    };

    if (symbol) {
      fetchContractInfo();
    }
  }, [symbol]);

  const fetchAnalysis = async (expiry: string) => {
    try {
      setAnalysisLoading(true);
      
      const response = await apiClient.getSingleAnalysis(symbol, expiry);
      
      if (response.success && response.data) {
        setAnalysisData(response.data);
      } else {
        setError(response.error || 'Failed to fetch analysis');
      }
    } catch (err) {
      setError(handleApiError(err));
    } finally {
      setAnalysisLoading(false);
    }
  };

  const handleExpirySelect = (expiry: string) => {
    setSelectedExpiry(expiry);
    setAnalysisData(null);
    fetchAnalysis(expiry);
  };

  // Helper function to get money status color
  const getMoneyStatusColor = (theMoneyStatus: string): string => {
    if (theMoneyStatus === 'ATM') return 'text-yellow-400';
    if (theMoneyStatus.includes('ITM')) return 'text-green-400';
    if (theMoneyStatus.includes('OTM')) return 'text-red-400';
    return 'text-gray-400';
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
          <p className="text-gray-400 text-lg">Loading contract info...</p>
        </div>
      </div>
    );
  }

  if (error && !contractInfo) {
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
          <Link href="/" className="inline-flex items-center text-gray-400 hover:text-nse-accent transition-colors mb-4">
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Securities
          </Link>
          
          <div className="flex items-center gap-4 mb-6">
            <h1 className="text-4xl font-display font-bold text-gradient">
              {symbol}
            </h1>
            {contractInfo && (
              <span className="px-3 py-1 bg-nse-surface rounded-full text-sm text-gray-300">
                {contractInfo.expiry_dates.length} Expiries Available
              </span>
            )}
          </div>
        </header>

        {/* Expiry Selection */}
        {contractInfo && (
          <section className="card-glow rounded-lg p-6 mb-8">
            <div className="flex items-center gap-4 mb-4">
              <Calendar className="w-5 h-5 text-nse-accent" />
              <h2 className="text-xl font-semibold text-gray-100">Select Expiry Date</h2>
            </div>
            
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3">
              {contractInfo.expiry_dates.map((expiry) => (
                <button
                  key={expiry}
                  onClick={() => handleExpirySelect(expiry)}
                  disabled={analysisLoading}
                  className={`p-3 rounded-lg font-medium transition-all ${
                    selectedExpiry === expiry
                      ? 'bg-nse-accent text-white shadow-lg'
                      : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                  } ${analysisLoading ? 'opacity-50 cursor-not-allowed' : ''}`}
                >
                  {expiry}
                </button>
              ))}
            </div>
          </section>
        )}

        {/* Analysis Results */}
        {selectedExpiry && (
          <section className="space-y-6">
            {analysisLoading ? (
              <div className="card-glow rounded-lg p-12 text-center">
                <Loader2 className="w-8 h-8 animate-spin mx-auto text-nse-accent mb-4" />
                <p className="text-gray-400">Analyzing {symbol} for {selectedExpiry}...</p>
              </div>
            ) : analysisData ? (
              <>
                {/* Summary Cards */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-2">
                      <DollarSign className="w-5 h-5 text-nse-accent" />
                      <span className="text-sm text-gray-400">Underlying Value</span>
                    </div>
                    <p className="text-2xl font-bold text-gray-100">
                      {formatCurrency(analysisData.underlying_value)}
                    </p>
                    <p className="text-sm text-gray-500 mt-1">as of {analysisData.timestamp}</p>
                  </div>

                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-2">
                      <BarChart3 className="w-5 h-5 text-nse-accent" />
                      <span className="text-sm text-gray-400">Spread</span>
                    </div>
                    <p className="text-2xl font-bold text-gray-100">
                      ₹{analysisData.spread.toFixed(2)}
                    </p>
                    <p className="text-sm text-gray-500 mt-1">between strikes</p>
                  </div>

                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-2">
                      <Clock className="w-5 h-5 text-nse-accent" />
                      <span className="text-sm text-gray-400">Days to Expiry</span>
                    </div>
                    <p className="text-2xl font-bold text-gray-100">
                      {analysisData.days_to_expiry}
                    </p>
                    <p className="text-sm text-gray-500 mt-1">trading days left</p>
                  </div>

                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-2">
                      <TrendingUp className="w-5 h-5 text-nse-accent" />
                      <span className="text-sm text-gray-400">Total OI</span>
                    </div>
                    <p className="text-2xl font-bold text-gray-100">
                      {((analysisData.ce_oi + analysisData.pe_oi) / 10000000).toFixed(1)}Cr
                    </p>
                    <p className="text-sm text-gray-500 mt-1">
                      CE: {(analysisData.ce_oi / 10000000).toFixed(1)}Cr | PE: {(analysisData.pe_oi / 10000000).toFixed(1)}Cr
                    </p>
                  </div>
                </div>

                {/* Alerts */}
                {analysisData.alerts && analysisData.alerts.alerts.length > 0 && (
                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-4">
                      <AlertCircle className="w-5 h-5 text-nse-warning" />
                      <h2 className="text-xl font-semibold text-gray-100">
                        Active Alerts ({analysisData.alerts.alerts.length})
                      </h2>
                    </div>
                    
                    <div className="space-y-4">
                      {analysisData.alerts.alerts.map((alert, index) => (
                        <div key={index} className="bg-slate-800/50 rounded-lg p-4 border border-gray-700/50">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-3">
                              <span className={getAlertBadgeClass(alert.alert_type)}>
                                {alert.alert_type.replace('_', ' ')}
                              </span>
                              <span className={`font-medium ${
                                alert.option_type === 'CE' ? 'text-green-400' : 'text-red-400'
                              }`}>
                                {alert.option_type}
                              </span>
                              <span className="text-gray-400">
                                Strike: ₹{alert.strike_price}
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
                            {alert.values.last_price && (
                              <div>
                                <span className="text-gray-500">Last Price:</span>
                                <p className="text-gray-200 font-medium">₹{alert.values.last_price.toFixed(2)}</p>
                              </div>
                            )}
                            {alert.values.pchange_in_oi && (
                              <div>
                                <span className="text-gray-500">OI Change:</span>
                                <p className={`font-medium ${
                                  alert.values.pchange_in_oi > 0 ? 'text-green-400' : 'text-red-400'
                                }`}>
                                  {alert.values.pchange_in_oi.toFixed(2)}%
                                </p>
                              </div>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Options Chain Table */}
                <div className="card-glow rounded-lg overflow-hidden">
                  <div className="p-6 border-b border-gray-700/50">
                    <h2 className="text-xl font-semibold text-gray-100">Options Chain Data</h2>
                    <p className="text-gray-400 text-sm mt-1">
                      Showing filtered strikes around ATM with high OI outliers
                    </p>
                  </div>
                  
                  <div className="overflow-x-auto">
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Strike</th>
                          <th>CE LTP</th>
                          <th>CE OI</th>
                          <th>CE %OI</th>
                          <th>CE Time Val</th>
                          <th>CE Money</th>
                          <th>PE Money</th>
                          <th>PE Time Val</th>
                          <th>PE %OI</th>
                          <th>PE OI</th>
                          <th>PE LTP</th>
                        </tr>
                      </thead>
                      <tbody>
                        {analysisData.processed_data.map((option, index) => (
                          <tr key={index}>
                            <td className="font-bold text-center bg-slate-700/50">
                              ₹{option.strikePrice}
                            </td>
                            {/* CE Data */}
                            <td className="text-green-400">
                              {option.CE?.lastPrice ? `₹${option.CE.lastPrice.toFixed(2)}` : '-'}
                            </td>
                            <td>
                              {option.CE?.openInterest ? 
                                (option.CE.openInterest / 100000).toFixed(1) + 'L' : '-'}
                            </td>
                            <td className={option.CE?.pchangeinOpenInterest ? 
                              (option.CE.pchangeinOpenInterest > 0 ? 'text-green-400' : 'text-red-400') : ''}>
                              {option.CE?.pchangeinOpenInterest ? 
                                `${option.CE.pchangeinOpenInterest > 0 ? '+' : ''}${option.CE.pchangeinOpenInterest.toFixed(1)}%` : '-'}
                            </td>
                            <td>
                              {option.CE ? `₹${option.CE.time_val.toFixed(2)}` : '-'}
                            </td>
                            <td className={option.CE ? getMoneyStatusColor(option.CE.the_money) : ''}>
                              {option.CE?.the_money || '-'}
                            </td>
                            {/* PE Data */}
                            <td className={option.PE ? getMoneyStatusColor(option.PE.the_money) : ''}>
                              {option.PE?.the_money || '-'}
                            </td>
                            <td>
                              {option.PE ? `₹${option.PE.time_val.toFixed(2)}` : '-'}
                            </td>
                            <td className={option.PE?.pchangeinOpenInterest ? 
                              (option.PE.pchangeinOpenInterest > 0 ? 'text-green-400' : 'text-red-400') : ''}>
                              {option.PE?.pchangeinOpenInterest ? 
                                `${option.PE.pchangeinOpenInterest > 0 ? '+' : ''}${option.PE.pchangeinOpenInterest.toFixed(1)}%` : '-'}
                            </td>
                            <td>
                              {option.PE?.openInterest ? 
                                (option.PE.openInterest / 100000).toFixed(1) + 'L' : '-'}
                            </td>
                            <td className="text-red-400">
                              {option.PE?.lastPrice ? `₹${option.PE.lastPrice.toFixed(2)}` : '-'}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* No Alerts Message */}
                {(!analysisData.alerts || analysisData.alerts.alerts.length === 0) && (
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
          </section>
        )}
      </div>
    </div>
  );
}