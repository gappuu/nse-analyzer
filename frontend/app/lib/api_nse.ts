import axios from 'axios';
import {
  ApiResponse,
  SecurityListResponse,
  ContractInfoResponse,
  SingleAnalysisResponse,
  BatchAnalysisResponse,
  FuturesDataResponse,
} from '@/app/types/api_nse_type';
import { db, DB_KEYS } from '@/app/lib/db';

// For static export, we need to use the full API URL
const API_NSE_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:3001'
  : process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

const api = axios.create({
  baseURL: API_NSE_BASE_URL,
  timeout: 120000,
});

// Enhanced API client with database integration
export const apiClient = {
  // Securities List
  async getSecurities(forceRefresh = false): Promise<ApiResponse<SecurityListResponse>> {
    try {
      if (!forceRefresh) {
        const cachedData = await db.getData<SecurityListResponse>(DB_KEYS.SECURITIES_LIST);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          } as ApiResponse<SecurityListResponse> & { fromCache: boolean; cachedAt: string; lastUpdated: number };
        }
      }

      const response = await api.get('/api/nse/securities');
      const apiResponse: ApiResponse<SecurityListResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.SECURITIES_LIST, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching securities:', error);
      throw error;
    }
  },

  // Contract Info
  async getContractInfo(symbol: string, forceRefresh = false): Promise<ApiResponse<ContractInfoResponse>> {
    try {
      const key = DB_KEYS.CONTRACT_INFO(symbol);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<ContractInfoResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          } as ApiResponse<ContractInfoResponse> & { fromCache: boolean; cachedAt: string; lastUpdated: number };
        }
      }

      const response = await api.get(`/api/nse/contract-info?symbol=${encodeURIComponent(symbol)}`);
      const apiResponse: ApiResponse<ContractInfoResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching contract info:', error);
      throw error;
    }
  },

  // Single Analysis
  async getSingleAnalysis(
    symbol: string,
    expiry: string,
    forceRefresh = false
  ): Promise<ApiResponse<SingleAnalysisResponse>> {
    try {
      const key = DB_KEYS.SINGLE_ANALYSIS(symbol, expiry);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<SingleAnalysisResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          } as ApiResponse<SingleAnalysisResponse> & { fromCache: boolean; cachedAt: string; lastUpdated: number };
        }
      }

      const response = await api.get(
        `/api/nse/single-analysis?symbol=${encodeURIComponent(symbol)}&expiry=${encodeURIComponent(expiry)}`
      );
      const apiResponse: ApiResponse<SingleAnalysisResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching single analysis:', error);
      throw error;
    }
  },

  // Futures Data
  async getFuturesData(
    symbol: string,
    expiry: string,
    forceRefresh = false
  ): Promise<ApiResponse<FuturesDataResponse>> {
    try {
      const key = DB_KEYS.FUTURES_DATA(symbol, expiry);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<FuturesDataResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          } as ApiResponse<FuturesDataResponse> & { fromCache: boolean; cachedAt: string; lastUpdated: number };
        }
      }

      const response = await api.get(
        `/api/nse/futures-data?symbol=${encodeURIComponent(symbol)}&expiry=${encodeURIComponent(expiry)}`
      );
      const apiResponse: ApiResponse<FuturesDataResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching futures data:', error);
      throw error;
    }
  },

  // Batch Analysis
  async getBatchAnalysis(forceRefresh = false): Promise<ApiResponse<BatchAnalysisResponse>> {
    try {
      if (!forceRefresh) {
        const cachedData = await db.getData<BatchAnalysisResponse>(DB_KEYS.BATCH_ANALYSIS);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          } as ApiResponse<BatchAnalysisResponse> & { fromCache: boolean; cachedAt: string; lastUpdated: number };
        }
      }

      const response = await api.post('/api/nse/batch-analysis');
      const apiResponse: ApiResponse<BatchAnalysisResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.BATCH_ANALYSIS, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching batch analysis:', error);
      throw error;
    }
  },

  // Derivatives Historical Data
  async getDerivativesHistorical(params: {
    symbol: string;
    instrument_type: 'FUTURES' | 'OPTIONS';
    expiry: string;
    year?: string;
    strike_price?: string;
    option_type?: 'CE' | 'PE';
    from_date: string;
    to_date: string;
  }): Promise<ApiResponse<any>> {
    try {
      const queryParams = new URLSearchParams({
        symbol: params.symbol,
        instrument_type: params.instrument_type,
        expiry: params.expiry,
        from_date: params.from_date,
        to_date: params.to_date,
      });

      if (params.year) queryParams.append('year', params.year);
      if (params.strike_price) queryParams.append('strike_price', params.strike_price);
      if (params.option_type) queryParams.append('option_type', params.option_type);

      const response = await api.get(`/api/nse/derivatives-historical?${queryParams.toString()}`);
      return response.data;
    } catch (error) {
      console.error('Error fetching derivatives historical data:', error);
      throw error;
    }
  },

  // Check if data exists in cache
  async hasSecurities(): Promise<boolean> {
    return await db.hasData(DB_KEYS.SECURITIES_LIST);
  },

  async hasContractInfo(symbol: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.CONTRACT_INFO(symbol));
  },

  async hasSingleAnalysis(symbol: string, expiry: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.SINGLE_ANALYSIS(symbol, expiry));
  },

  async hasFuturesData(symbol: string, expiry: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.FUTURES_DATA(symbol, expiry));
  },

  async hasBatchAnalysis(): Promise<boolean> {
    return await db.hasData(DB_KEYS.BATCH_ANALYSIS);
  }
};

// Error handler utility
export const handleApiError = (error: any): string => {
  if (error.response?.data?.error) {
    return error.response.data.error;
  }
  if (error.message) {
    return error.message;
  }
  return 'An unexpected error occurred';
};

// Format currency
export const formatCurrency = (value: number): string => {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    minimumFractionDigits: 2,
  }).format(value);
};

// Format percentage
export const formatPercentage = (value: number): string => {
  return `${value > 0 ? '+' : ''}${value.toFixed(2)}%`;
};

// Format large numbers
export const formatLargeNumber = (value: number): string => {
  if (value >= 10000000) {
    return `${(value / 10000000).toFixed(1)}Cr`;
  }
  if (value >= 100000) {
    return `${(value / 100000).toFixed(1)}L`;
  }
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}K`;
  }
  return value.toString();
};

// Get alert badge class
export const getAlertBadgeClass = (alertType: string): string => {
  switch (alertType) {
    case 'HUGE_OI_INCREASE':
      return 'alert-huge-oi-increase';
    case 'HUGE_OI_DECREASE':
      return 'alert-huge-oi-decrease';
    case 'LOW_PRICE':
      return 'alert-low-price';
    case 'NEGATIVE TIME VALUE':
      return 'alert-negative-time-value';
    default:
      return 'alert-badge bg-gray-900/30 text-gray-300 border border-gray-700/50';
  }
};

// Get option type color
export const getOptionTypeColor = (optionType: string): string => {
  return optionType === 'CE' ? 'text-green-400' : 'text-red-400';
};

// Get money status color class
export const getMoneyStatusColor = (theMoneyStatus: string): string => {
  if (theMoneyStatus === 'ATM') return 'money-atm';
  if (theMoneyStatus.includes('ITM')) return 'money-itm';
  if (theMoneyStatus.includes('OTM')) return 'money-otm';
  return 'text-gray-400';
};

// Get percentage change color class
export const getPChangeColorClass = (value: number | undefined): string => {
  if (value === undefined || value === null) return 'pchange-neutral';
  return value > 0 ? 'pchange-positive' : value < 0 ? 'pchange-negative' : 'pchange-neutral';
};

// Format date to DD-MMM-YYYY format (e.g., 06-Dec-2025)
export const formatDateForAPI = (date: Date): string => {
  const day = String(date.getDate()).padStart(2, '0');
  const month = String(date.getMonth() + 1).padStart(2, '0'); 
  const year = date.getFullYear();
  return `${day}-${month}-${year}`;
};

// Get date 20 days ago
export const get20DaysAgo = (): string => {
  const date = new Date();
  date.setDate(date.getDate() - 20);
  return formatDateForAPI(date);
};

// Get today's date
export const getToday = (): string => {
  return formatDateForAPI(new Date());
};