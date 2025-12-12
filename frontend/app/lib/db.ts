// Database utility for LevelDB data storage with timestamps
interface StoredData<T> {
  data: T;
  timestamp: string;
  lastUpdated: number; // Unix timestamp for easy comparison
}

export class DataStorage {
  private dbName = 'nse-options-db';
  
  // Store data with timestamp
  async storeData<T>(key: string, data: T): Promise<void> {
    const storedData: StoredData<T> = {
      data,
      timestamp: new Date().toISOString(),
      lastUpdated: Date.now()
    };
    
    try {
      localStorage.setItem(`${this.dbName}:${key}`, JSON.stringify(storedData));
    } catch (error) {
      console.error(`Failed to store data for key ${key}:`, error);
      throw error;
    }
  }

  // Retrieve data with metadata
  async getData<T>(key: string): Promise<StoredData<T> | null> {
    try {
      const stored = localStorage.getItem(`${this.dbName}:${key}`);
      if (!stored) {
        return null;
      }
      return JSON.parse(stored) as StoredData<T>;
    } catch (error) {
      console.error(`Failed to retrieve data for key ${key}:`, error);
      return null;
    }
  }

  // Check if data exists
  async hasData(key: string): Promise<boolean> {
    const stored = await this.getData(key);
    return stored !== null;
  }

  // Get data age in human readable format
  getDataAge(lastUpdated: number): string {
    const now = Date.now();
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

  // Clear specific data
  async clearData(key: string): Promise<void> {
    try {
      localStorage.removeItem(`${this.dbName}:${key}`);
    } catch (error) {
      console.error(`Failed to clear data for key ${key}:`, error);
      throw error;
    }
  }

  // Clear all data
  async clearAllData(): Promise<void> {
    try {
      const keys = Object.keys(localStorage).filter(key => key.startsWith(`${this.dbName}:`));
      keys.forEach(key => localStorage.removeItem(key));
    } catch (error) {
      console.error('Failed to clear all data:', error);
      throw error;
    }
  }
}

// Database keys for different data types
export const DB_KEYS = {
  SECURITIES_LIST: 'securities_list',
  CONTRACT_INFO: (symbol: string) => `contract_info:${symbol}`,
  SINGLE_ANALYSIS: (symbol: string, expiry: string) => `single_analysis:${symbol}:${expiry}`,
  BATCH_ANALYSIS: 'batch_analysis'
} as const;

// Singleton instance
export const db = new DataStorage();