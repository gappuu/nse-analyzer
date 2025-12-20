import axios from 'axios';
import { db, DB_KEYS } from '@/app/lib/db';
import {
  McxApiResponse,
  McxTickersResponse,
  McxFutureSymbolsResponse,
  McxOptionChainResponse,
  McxFutureQuoteResponse,
  McxOptionQuoteResponse,
  McxBatchAnalysisResponse,
  McxHistoricalDataResponse,
  // McxDataWithAge
} from '@/app/types/api_mcx_type';

// For static export, we need to use the full API URL
const API_MCX_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:3002'
  : process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002';

const api = axios.create({
  baseURL: API_MCX_BASE_URL,
  timeout: 120000,
});

// Enhanced MCX API client with database integration
export const mcxApiClient = {
  // Check MCX API Health
  async checkHealth(): Promise<McxApiResponse<any>> {
    try {
      const response = await api.get('/mcx_health');
      return response.data;
    } catch (error) {
      console.error('Error checking MCX health:', error);
      throw error;
    }
  },

  // MCX Tickers List
  async getTickers(forceRefresh = false): Promise<McxApiResponse<McxTickersResponse>> {
    try {
      if (!forceRefresh) {
        const cachedData = await db.getData<McxTickersResponse>(DB_KEYS.MCX_TICKERS);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const response = await api.get('/api/mcx/tickers');
      const apiResponse: McxApiResponse<McxTickersResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.MCX_TICKERS, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX tickers:', error);
      throw error;
    }
  },

  // MCX Future Symbols
  async getFutureSymbols(forceRefresh = false): Promise<McxApiResponse<McxFutureSymbolsResponse>> {
    try {
      if (!forceRefresh) {
        const cachedData = await db.getData<McxFutureSymbolsResponse>(DB_KEYS.MCX_FUTURE_SYMBOLS);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const response = await api.get('/api/mcx/future-symbols');
      const apiResponse: McxApiResponse<McxFutureSymbolsResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.MCX_FUTURE_SYMBOLS, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX future symbols:', error);
      throw error;
    }
  },

  // MCX Option Chain
  async getOptionChain(
    commodity: string,
    expiry: string,
    forceRefresh = false
  ): Promise<McxApiResponse<McxOptionChainResponse>> {
    try {
      const key = DB_KEYS.MCX_OPTION_CHAIN(commodity, expiry);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<McxOptionChainResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const response = await api.get(
        `/api/mcx/option-chain?commodity=${encodeURIComponent(commodity)}&expiry=${encodeURIComponent(expiry)}`
      );
      const apiResponse: McxApiResponse<McxOptionChainResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX option chain:', error);
      throw error;
    }
  },

  // MCX Future Quote
  async getFutureQuote(
    commodity: string,
    expiry: string,
    forceRefresh = false
  ): Promise<McxApiResponse<McxFutureQuoteResponse>> {
    try {
      const key = DB_KEYS.MCX_FUTURE_QUOTE(commodity, expiry);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<McxFutureQuoteResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const response = await api.get(
        `/api/mcx/future-quote?commodity=${encodeURIComponent(commodity)}&expiry=${encodeURIComponent(expiry)}`
      );
      const apiResponse: McxApiResponse<McxFutureQuoteResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX future quote:', error);
      throw error;
    }
  },

  // MCX Option Quote
  async getOptionQuote(
    commodity: string,
    expiry: string,
    optionType: string,
    strikePrice: string,
    forceRefresh = false
  ): Promise<McxApiResponse<McxOptionQuoteResponse>> {
    try {
      const key = DB_KEYS.MCX_OPTION_QUOTE(commodity, expiry, optionType, strikePrice);
      
      if (!forceRefresh) {
        const cachedData = await db.getData<McxOptionQuoteResponse>(key);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const queryParams = new URLSearchParams({
        commodity,
        expiry,
        option_type: optionType,
        strike_price: strikePrice
      });

      const response = await api.get(`/api/mcx/option-quote?${queryParams.toString()}`);
      const apiResponse: McxApiResponse<McxOptionQuoteResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX option quote:', error);
      throw error;
    }
  },

  // MCX Batch Analysis
  async getBatchAnalysis(forceRefresh = false): Promise<McxApiResponse<McxBatchAnalysisResponse>> {
    try {
      if (!forceRefresh) {
        const cachedData = await db.getData<McxBatchAnalysisResponse>(DB_KEYS.MCX_BATCH_ANALYSIS);
        if (cachedData) {
          return {
            success: true,
            data: cachedData.data,
            fromCache: true,
            cachedAt: cachedData.timestamp,
            lastUpdated: cachedData.lastUpdated
          };
        }
      }

      const response = await api.post('/api/mcx/batch-analysis');
      const apiResponse: McxApiResponse<McxBatchAnalysisResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(DB_KEYS.MCX_BATCH_ANALYSIS, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX batch analysis:', error);
      throw error;
    }
  },

  // MCX Historical Data
  async getHistoricalData(params: {
    symbol: string;
    expiry: string;
    from_date: string;
    to_date: string;
  }): Promise<McxApiResponse<McxHistoricalDataResponse>> {
    try {
      const key = DB_KEYS.MCX_HISTORICAL_DATA(params.symbol, params.expiry, params.from_date, params.to_date);
      
      const cachedData = await db.getData<McxHistoricalDataResponse>(key);
      if (cachedData) {
        return {
          success: true,
          data: cachedData.data,
          fromCache: true,
          cachedAt: cachedData.timestamp,
          lastUpdated: cachedData.lastUpdated
        };
      }

      const queryParams = new URLSearchParams({
        symbol: params.symbol,
        expiry: params.expiry,
        from_date: params.from_date,
        to_date: params.to_date
      });

      const response = await api.get(`/api/mcx/historic-data?${queryParams.toString()}`);
      const apiResponse: McxApiResponse<McxHistoricalDataResponse> = response.data;

      if (apiResponse.success && apiResponse.data) {
        await db.storeData(key, apiResponse.data);
      }

      return apiResponse;
    } catch (error) {
      console.error('Error fetching MCX historical data:', error);
      throw error;
    }
  },

  // Check if data exists in cache
  async hasTickers(): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_TICKERS);
  },

  async hasFutureSymbols(): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_FUTURE_SYMBOLS);
  },

  async hasOptionChain(commodity: string, expiry: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_OPTION_CHAIN(commodity, expiry));
  },

  async hasFutureQuote(commodity: string, expiry: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_FUTURE_QUOTE(commodity, expiry));
  },

  async hasOptionQuote(commodity: string, expiry: string, optionType: string, strikePrice: string): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_OPTION_QUOTE(commodity, expiry, optionType, strikePrice));
  },

  async hasBatchAnalysis(): Promise<boolean> {
    return await db.hasData(DB_KEYS.MCX_BATCH_ANALYSIS);
  }
};

// MCX Error handler utility
export const handleMcxApiError = (error: any): string => {
  if (error.response?.data?.error) {
    return error.response.data.error;
  }
  if (error.message) {
    return error.message;
  }
  return 'An unexpected error occurred with MCX API';
};

// MCX specific utility functions
export const formatMcxCommodityName = (commodity: string): string => {
  return commodity.replace(/([A-Z])/g, ' $1').trim().toUpperCase();
};

export const getMcxCommodityIcon = (commodity: string): string => {
  const iconMap: Record<string, string> = {
    'CRUDEOIL': '🛢️',
    'GOLD': '🥇',
    'SILVER': '⚪',
    'COPPER': '🔶',
    'NATURALGAS': '🔥',
    'MENTHAOIL': '🌿',
    'ZINC': '🔩',
    'ALUMINIUM': '🔗',
    'NICKEL': '⚙️',
    'LEAD': '⚫'
  };
  
  return iconMap[commodity.toUpperCase()] || '📈';
};

// Get unique commodity letters for filtering
export const getMcxCommodityLetters = (commodities: Array<{name: string}>): string[] => {
  const letters = new Set<string>();
  commodities.forEach(commodity => {
    const firstLetter = commodity.name.charAt(0).toUpperCase();
    letters.add(firstLetter);
  });
  return Array.from(letters).sort();
};