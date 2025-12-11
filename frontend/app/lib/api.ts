import axios from 'axios';
import {
  ApiResponse,
  SecurityListResponse,
  ContractInfoResponse,
  SingleAnalysisResponse,
  BatchAnalysisResponse,
} from '@/app/types/api';

const API_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:3001'
  : '';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 120000, // 2 minutes timeout for batch operations
});

export const apiClient = {
  async getSecurities(): Promise<ApiResponse<SecurityListResponse>> {
    const response = await api.get('/api/securities');
    return response.data;
  },

  async getContractInfo(symbol: string): Promise<ApiResponse<ContractInfoResponse>> {
    const response = await api.get(`/api/contract-info?symbol=${encodeURIComponent(symbol)}`);
    return response.data;
  },

  async getSingleAnalysis(
    symbol: string,
    expiry: string
  ): Promise<ApiResponse<SingleAnalysisResponse>> {
    const response = await api.get(
      `/api/single-analysis?symbol=${encodeURIComponent(symbol)}&expiry=${encodeURIComponent(expiry)}`
    );
    return response.data;
  },

  async getBatchAnalysis(): Promise<ApiResponse<BatchAnalysisResponse>> {
    const response = await api.post('/api/batch-analysis');
    return response.data;
  },
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