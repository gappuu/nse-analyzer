import axios from 'axios';
import {
  ApiResponse,
  SecurityListResponse,
  ContractInfoResponse,
  SingleAnalysisResponse,
  BatchAnalysisResponse,
} from '@/app/types/api';
import { db, DB_KEYS } from '@/app/lib/db';

const API_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:3001'
  : '';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 120000, // 2 minutes timeout for batch operations
});

// Enhanced API client with database integration
export const apiClient = {
  // Securities List
  async getSecurities(forceRefresh = false): Promise<ApiResponse<SecurityListResponse>> {
    try {
      // Check if we should use cached data
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

      // Fetch from API
      const response = await api.get('/api/securities');
      const apiResponse: ApiResponse<SecurityListResponse> = response.data;

      // Store in database if successful
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
      
      // Check if we should use cached data
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

      // Fetch from API
      const response = await api.get(`/api/contract-info?symbol=${encodeURIComponent(symbol)}`);
      const apiResponse: ApiResponse<ContractInfoResponse> = response.data;

      // Store in database if successful
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
      
      // Check if we should use cached data
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

      // Fetch from API
      const response = await api.get(
        `/api/single-analysis?symbol=${encodeURIComponent(symbol)}&expiry=${encodeURIComponent(expiry)}`
      );
      const apiResponse: ApiResponse<SingleAnalysisResponse> = response.data;

      // Store in database if successful
      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching single analysis:', error);
      throw error;
    }
  },

  // Batch Analysis
  async getBatchAnalysis(forceRefresh = false): Promise<ApiResponse<BatchAnalysisResponse>> {
    try {
      // Check if we should use cached data
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

      // Fetch from API
      const response = await api.post('/api/batch-analysis');
      const apiResponse: ApiResponse<BatchAnalysisResponse> = response.data;

      // Store in database if successful
      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.BATCH_ANALYSIS, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching batch analysis:', error);
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

// Get money status color
export const getMoneyStatusColor = (theMoneyStatus: string): string => {
  if (theMoneyStatus === 'ATM') return 'text-yellow-400';
  if (theMoneyStatus.includes('ITM')) return 'text-green-400';
  if (theMoneyStatus.includes('OTM')) return 'text-red-400';
  return 'text-gray-400';
};