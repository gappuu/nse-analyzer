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
  BarChart3,
  RefreshCw,
  Database
} from 'lucide-react';
import { apiClient, handleApiError, getAlertBadgeClass, formatCurrency, getMoneyStatusColor } from '@/app/lib/api';
import { db } from '@/app/lib/db';
import { ContractInfoResponse, SingleAnalysisResponse, DataWithAge } from '@/app/types/api';

export default function SecurityPage() {
  const params = useParams();
  const symbol = params.symbol as string;
  
  const [contractData, setContractData] = useState<DataWithAge<ContractInfoResponse> | null>(null);
  const [selectedExpiry, setSelectedExpiry] = useState<string | null>(null);
  const [analysisData, setAnalysisData] = useState<DataWithAge<SingleAnalysisResponse> | null>(null);
  const [loading, setLoading] = useState(true);
  const [analysisLoading, setAnalysisLoading] = useState(false);
  const [fetching, setFetching] = useState(false);
  const [analysisFetching, setAnalysisFetching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentTime, setCurrentTime] = useState(Date.now());

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

  // Function to calculate real-time age from data timestamp string
  const getDataTimestampAge = (timestampString: string) => {
    try {
      const dataTime = new Date(timestampString).getTime();
      return db.getDataAge(dataTime, currentTime);
    } catch (error) {
      return 'unknown';
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
    const fetchContractInfo = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const response = await apiClient.getContractInfo(symbol);
        
        if (response.success && response.data) {
          const dataWithAge: DataWithAge<ContractInfoResponse> = {
            data: response.data,
            age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
            lastUpdated: response.lastUpdated || Date.now(),
            fromCache: response.fromCache || false
          };
          setContractData(dataWithAge);
          
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

  const fetchAnalysis = async (expiry: string, forceRefresh = false) => {
    try {
      if (forceRefresh) {
        setAnalysisFetching(true);
      } else {
        setAnalysisLoading(true);
      }
      
      const response = await apiClient.getSingleAnalysis(symbol, expiry, forceRefresh);
      
      if (response.success && response.data) {
        const dataWithAge: DataWithAge<SingleAnalysisResponse> = {
          data: response.data,
          age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
          lastUpdated: response.lastUpdated || Date.now(),
          fromCache: response.fromCache || false
        };
        setAnalysisData(dataWithAge);
      } else {
        setError(response.error || 'Failed to fetch analysis');
      }
    } catch (err) {
      setError(handleApiError(err));
    } finally {
      setAnalysisLoading(false);
      setAnalysisFetching(false);
    }
  };

  const handleExpirySelect = (expiry: string) => {
    setSelectedExpiry(expiry);
    setAnalysisData(null);
    fetchAnalysis(expiry);
  };

  const handleFetchContractInfo = () => {
    setFetching(true);
    apiClient.getContractInfo(symbol, true)
      .then(response => {
        if (response.success && response.data) {
          const dataWithAge: DataWithAge<ContractInfoResponse> = {
            data: response.data,
            age: response.lastUpdated ? db.getDataAge(response.lastUpdated) : 'just now',
            lastUpdated: response.lastUpdated || Date.now(),
            fromCache: response.fromCache || false
          };
          setContractData(dataWithAge);
        }
      })
      .catch(err => setError(handleApiError(err)))
      .finally(() => setFetching(false));
  };

  const handleFetchAnalysis = () => {
    if (selectedExpiry) {
      fetchAnalysis(selectedExpiry, true);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="w-12 h-12 animate-spin mx-auto text-nse-accent mb-4" />
          <p className="text-gray-400 text-lg">Loading contract info from database...</p>
        </div>
      </div>
    );
  }

  if (error && !contractData) {
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
            {contractData && (
              <span className="px-3 py-1 bg-nse-surface rounded-full text-sm text-gray-300">
                {contractData.data.expiry_dates.length} Expiries Available
              </span>
            )}
          </div>

          {/* Contract Info Age and Fetch Controls */}
          {contractData && (
            <div className="card-glow rounded-lg p-4 mb-6">
              <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                <div className="flex items-center gap-3 text-sm">
                  <div className="flex items-center gap-2">
                    {contractData.fromCache ? (
                      <Database className="w-4 h-4 text-blue-400" />
                    ) : (
                      <Clock className="w-4 h-4 text-green-400" />
                    )}
                    <span className="text-gray-400">
                      Contract info {contractData.fromCache ? 'from cache' : 'freshly fetched'}
                    </span>
                  </div>
                  <span className="text-gray-500">•</span>
                  <div className="flex items-center gap-2">
                    <Clock className="w-4 h-4 text-gray-500" />
                    <span className="text-gray-400">Updated {getRealTimeAge(contractData.lastUpdated)}</span>
                  </div>
                </div>
                
                <button
                  onClick={handleFetchContractInfo}
                  disabled={fetching}
                  className="btn-secondary inline-flex items-center text-sm"
                >
                  {fetching ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <RefreshCw className="w-4 h-4 mr-2" />
                  )}
                  {fetching ? 'Fetching...' : 'Fetch Expiry'}
                </button>
              </div>
            </div>
          )}
        </header>

        {/* Expiry Selection */}
        {contractData && (
          <section className="card-glow rounded-lg p-6 mb-8">
            <div className="flex items-center gap-4 mb-4">
              <Calendar className="w-5 h-5 text-nse-accent" />
              <h2 className="text-xl font-semibold text-gray-100">Select Expiry Date</h2>
            </div>
            
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3">
              {contractData.data.expiry_dates.map((expiry) => (
                <button
                  key={expiry}
                  onClick={() => handleExpirySelect(expiry)}
                  disabled={analysisLoading || analysisFetching}
                  className={`p-3 rounded-lg font-medium transition-all ${
                    selectedExpiry === expiry
                      ? 'bg-nse-accent text-white shadow-lg'
                      : 'bg-slate-700 text-gray-300 hover:bg-slate-600'
                  } ${(analysisLoading || analysisFetching) ? 'opacity-50 cursor-not-allowed' : ''}`}
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
                        Analysis {analysisData.fromCache ? 'from cache' : 'freshly fetched'} for {selectedExpiry}
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
                      {/* <DollarSign className="w-5 h-5 text-nse-accent" /> */}
                      <span className="text-sm text-gray-400">Underlying Value</span>
                    </div>
                    <p className="text-2xl font-bold text-gray-100">
                      {formatCurrency(analysisData.data.underlying_value)}
                    </p>
                    <p className="text-sm text-gray-500 mt-1">as of {analysisData.data.timestamp}</p>
                  </div>

                  <div className="card-glow rounded-lg p-6">
                    <div className="flex items-center gap-3 mb-2">
                      <BarChart3 className="w-5 h-5 text-nse-accent" />
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
                    {/* <p className="text-2xl font-bold text-gray-100">
                      {((analysisData.data.ce_oi + analysisData.data.pe_oi) ).toFixed(0)}
                    </p> */}
                    <p className="text-2xl font-bold text-gray-100">
                      CE : {Number(analysisData.data.ce_oi).toLocaleString("en-IN", {maximumFractionDigits: 0,})}
                    </p>
                    <p className="text-2xl font-bold text-gray-100">
                      PE : {Number(analysisData.data.pe_oi).toLocaleString("en-IN", {maximumFractionDigits: 0,})}
                    </p>
                  </div>
                </div>

                {/* Alerts */}
                {analysisData.data.alerts && analysisData.data.alerts.alerts.length > 0 && (
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
                                alert.option_type === 'CE' ? 'text-green-400' : 'text-red-400'
                              }`}>
                                {alert.option_type}
                              </span>
                              <span className="text-gray-400">
                                Strike: ₹{alert.strike_price} 
                              </span>
                              <span >
                                Expiry : {alert.expiry_date}
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

                {/* Summary Table - Separate from Options Chain */}
                <div className="card-glow rounded-t-lg overflow-hidden" style={{marginBottom:0}} >
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <tbody>
                        {/* Row 1: Symbol, Underlying Value, Timestamp, Fetch Timestamp */}
                        <tr className="border-b border-gray-700/50">
                          <td className="px-6 py-4 font-bold text-xl ">
                            <div className=" text-gradient">
                              {symbol} 
                            </div>
                              <div>
                                <sub> ₹ {analysisData.data.underlying_value.toLocaleString("en-IN")}</sub> 
                              </div>
                          </td>
                          <td className="px-6 py-4">
                            <div className="font-semibold text-lg text-gray-100">
                              {analysisData.data.timestamp}
                            </div>
                              <div className="text-sm text-gray-400">
                                Data Age : {getDataTimestampAge(analysisData.data.timestamp)}
                              </div>
                          </td>
                          <td className="px-6 py-4 text-center">
                            <div className="font-medium text-gray-100">
                              FUTURES
                            </div>
                              <div className="text-sm text-gray-400">
                                Short Covering
                              </div>
                          </td>
                          <td className="px-6 py-4 text-right">
                            <div className="font-medium text-gray-100">
                              Fetched {getRealTimeAge(analysisData.lastUpdated)}
                            </div>
                                <div className="text-sm text-gray-400">
                                 <button
                                  onClick={handleFetchAnalysis}
                                  disabled={analysisFetching}
                                  className="btn-outline-primary inline-flex items-center text-sm">
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
                            <div className="font-bold text-lg ">
                              Total CE : {analysisData.data.ce_oi.toLocaleString("en-IN")}
                            </div>
                            {/* <div className="text-sm text-gray-400">CE Total OI</div> */}
                          </td>
                          <td className="px-6 py-4 text-center" colSpan={2}>
                            <div className="font-bold text-lg text-nse-accent">
                              Expiry : {selectedExpiry} <sub>({analysisData.data.days_to_expiry} days left)</sub>
                            </div>
                            {/* <div className="text-sm text-gray-400">Selected Expiry</div> */}
                          </td>
                          <td className="px-6 py-4 text-center">
                            <div className="font-bold text-lg ">
                              Total PE : {analysisData.data.pe_oi.toLocaleString("en-IN")}
                            </div>
                            {/* <div className="text-sm text-gray-400">PE Total OI</div> */}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Options Chain Table - Separate table, no gap */}
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
                                    {Number(option.CE?.openInterest) || '-'}
                                    {ceOIRank.showRank && (
                                      <sup className="ml-1 text-xs ">
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
                                <td className={option.CE ? getMoneyStatusColor(option.CE.the_money) : 'text-gray-400'}>
                                  {option.CE?.the_money || '-'}
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
                                <td className={`font-bold text-center ${ option.CE ? getMoneyStatusColor(option.CE.the_money) : ""}`} style={{ backgroundColor: "rgba(51, 65, 85, 0.5)" }}> 
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
                                <td className={option.PE ? getMoneyStatusColor(option.PE.the_money) : 'text-gray-400'}>
                                  {option.PE?.the_money || '-'}
                                </td>
                                <td className={option.PE?.pchangeinOpenInterest ? 
                                  (option.PE.pchangeinOpenInterest > 0 ? 'pchange-positive' : 'pchange-negative') : 'pchange-neutral'}>
                                  {option.PE?.pchangeinOpenInterest ? 
                                    `${option.PE.pchangeinOpenInterest > 0 ? '+' : ''}${option.PE.pchangeinOpenInterest.toFixed(1)}%` : '0'}
                                </td>
                                <td className={`${option.PE?.openInterest ? peOIRank.className : 'text-gray-400'}`}>
                                  <div>
                                    {Number(option.PE?.openInterest) || '-'}
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
                {(!analysisData.data.alerts || analysisData.data.alerts.alerts.length === 0) && (
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