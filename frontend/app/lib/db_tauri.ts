// Tauri-specific database utility using Tauri Store plugin

interface StoredData<T> {
  data: T;
  timestamp: string;
  lastUpdated: number;
}

export class DataStorageTauri {
  private dbName = 'nse-options-db';
  private store: any = null;
  private initialized = false;

  async init(): Promise<void> {
    if (this.initialized) return;
    
    try {
      // Dynamic import to avoid type errors when not in Tauri environment
      const storeModule = await import('@tauri-apps/plugin-store');
      // Use the load method instead of constructor
      this.store = await storeModule.load('nse-options-store.bin');
      
      console.log('✅ Tauri store initialized');
      this.initialized = true;
    } catch (error) {
      console.error('Failed to initialize Tauri store:', error);
      throw error;
    }
  }

  async storeData<T>(key: string, data: T): Promise<void> {
    await this.init();
    
    const storedData: StoredData<T> = {
      data,
      timestamp: new Date().toISOString(),
      lastUpdated: Date.now()
    };
    
    const fullKey = `${this.dbName}:${key}`;
    
    try {
      if (this.store) {
        await this.store.set(fullKey, storedData);
        await this.store.save();
      } else {
        throw new Error('Tauri store not initialized');
      }
    } catch (error) {
      console.error(`Failed to store data for key ${key}:`, error);
      throw error;
    }
  }

  async getData<T>(key: string): Promise<StoredData<T> | null> {
    await this.init();
    
    const fullKey = `${this.dbName}:${key}`;
    
    try {
      if (this.store) {
        const stored = await this.store.get(fullKey);
        return stored as StoredData<T> | null;
      } else {
        throw new Error('Tauri store not initialized');
      }
    } catch (error) {
      console.error(`Failed to retrieve data for key ${key}:`, error);
      return null;
    }
  }

  async hasData(key: string): Promise<boolean> {
    try {
      const stored = await this.getData(key);
      return stored !== null;
    } catch (error) {
      return false;
    }
  }

  getDataAge(lastUpdated: number, currentTime?: number): string {
    const now = currentTime || Date.now();
    const diffMs = now - lastUpdated;
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMinutes < 1) {
      return 'just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes} minute${diffMinutes !== 1 ? 's' : ''} ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
    } else {
      return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    }
  }

  async clearData(key: string): Promise<void> {
    await this.init();
    
    const fullKey = `${this.dbName}:${key}`;
    
    try {
      if (this.store) {
        await this.store.delete(fullKey);
        await this.store.save();
        console.log(`🗑️ Cleared key: ${fullKey}`);
      } else {
        throw new Error('Tauri store not initialized');
      }
    } catch (error) {
      console.error(`Failed to clear data for key ${key}:`, error);
      throw error;
    }
  }

  async clearAllData(): Promise<void> {
    await this.init();
    
    try {
      if (this.store) {
        // Get all keys from store
        const allKeys = await this.store.keys();
        const keysToDelete = allKeys.filter((key: string) => 
          key.startsWith(`${this.dbName}:`)
        );
        
        console.log(`🗑️ Clearing ${keysToDelete.length} entries from Tauri store`);
        
        // Delete all keys with our prefix
        for (const key of keysToDelete) {
          await this.store.delete(key);
        }
        
        await this.store.save();
        console.log('✅ Cleared all Tauri store data');
      } else {
        throw new Error('Tauri store not initialized');
      }
    } catch (error) {
      console.error('Failed to clear all data:', error);
      throw error;
    }
  }

  async reset(): Promise<void> {
    await this.init();
    
    try {
      if (this.store) {
        await this.store.clear();
        await this.store.save();
        console.log('✅ Reset Tauri store completely');
      }
    } catch (error) {
      console.error('Failed to reset store:', error);
      throw error;
    }
  }
}

// Database keys (same as db.ts for consistency)
export const DB_KEYS = {
  // NSE Keys
  SECURITIES_LIST: 'securities_list',
  CONTRACT_INFO: (symbol: string) => `contract_info:${symbol}`,
  SINGLE_ANALYSIS: (symbol: string, expiry: string) => `single_analysis:${symbol}:${expiry}`,
  FUTURES_DATA: (symbol: string, expiry: string) => `futures_data:${symbol}:${expiry}`,
  BATCH_ANALYSIS: 'batch_analysis',
  
  // MCX Keys
  MCX_TICKERS: 'mcx_tickers',
  MCX_FUTURE_SYMBOLS: 'mcx_future_symbols',
  MCX_OPTION_CHAIN: (commodity: string, expiry: string) => `mcx_option_chain:${commodity}:${expiry}`,
  MCX_FUTURE_QUOTE: (commodity: string, expiry: string) => `mcx_future_quote:${commodity}:${expiry}`,
  MCX_OPTION_QUOTE: (commodity: string, expiry: string, optionType: string, strikePrice: string) => 
    `mcx_option_quote:${commodity}:${expiry}:${optionType}:${strikePrice}`,
  MCX_BATCH_ANALYSIS: 'mcx_batch_analysis',
  
  // Enhanced MCX Historical Data key with all required parameters
  MCX_HISTORICAL_DATA: (...params: string[]) => {
    const [symbol, expiry, fromDate, toDate, instrumentName, optionType = 'null', strikePrice = 'null'] = params;
    return `mcx_historical:${symbol}:${expiry}:${fromDate}:${toDate}:${instrumentName}:${optionType}:${strikePrice}`;
  }
} as const;