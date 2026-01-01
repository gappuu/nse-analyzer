interface PlatformInfo {
  is_tauri: boolean;
  platform: string;
}

interface BackendConfig {
  exchange: string;
  port: string;
  health_path: string;
}

let platformCache: PlatformInfo | null = null;
let backendConfigsCache: BackendConfig[] | null = null;

export async function getPlatformInfo(): Promise<PlatformInfo> {
  if (platformCache) {
    return platformCache;
  }

  try {
    // Dynamic import to avoid errors in web mode
    const { invoke } = await import('@tauri-apps/api/core');
    platformCache = await invoke<PlatformInfo>('get_platform_info');
    return platformCache;
  } catch (error) {
    // Not running in Tauri
    platformCache = {
      is_tauri: false,
      platform: 'web'
    };
    return platformCache;
  }
}

export async function getBackendConfigs(): Promise<BackendConfig[]> {
  if (backendConfigsCache) {
    return backendConfigsCache;
  }

  try {
    // Dynamic import to avoid errors in web mode
    const { invoke } = await import('@tauri-apps/api/core');
    
    // Retry logic for getting backend configs
    let retries = 10;
    while (retries > 0) {
      try {
        const configs = await invoke<BackendConfig[]>('get_backend_configs');
        if (configs && configs.length > 0) {
          backendConfigsCache = configs;
          console.log('✅ Got backend configs:', configs);
          return configs;
        }
      } catch (err) {
        console.log(`Waiting for backends to start... (${retries} retries left)`);
      }
      
      // Wait 1 second before retrying
      await new Promise(resolve => setTimeout(resolve, 1000));
      retries--;
    }
    
    throw new Error('Failed to get backend configs after retries');
  } catch (error) {
    console.log('Not running in Tauri or backends not ready, using defaults');
    // Return default configs for web mode
    backendConfigsCache = [
      { exchange: 'nse', port: '3001', health_path: '/nse_health' },
      { exchange: 'mcx', port: '3002', health_path: '/mcx_health' }
    ];
    return backendConfigsCache;
  }
}

export async function isTauriPlatform(): Promise<boolean> {
  const info = await getPlatformInfo();
  return info.is_tauri;
}

// Synchronous version (use after initial load)
let isTauriSync: boolean | null = null;

export function isTauri(): boolean {
  return isTauriSync ?? false;
}

// Initialize at app start
export async function initPlatform(): Promise<void> {
  const info = await getPlatformInfo();
  isTauriSync = info.is_tauri;
  
  // Preload backend configs if in Tauri
  if (info.is_tauri) {
    await getBackendConfigs();
  }
}

// Get API base URL for a specific exchange
export async function getApiBaseUrl(exchange: 'nse' | 'mcx'): Promise<string> {
  const info = await getPlatformInfo();
  
  if (info.is_tauri) {
    const configs = await getBackendConfigs();
    const config = configs.find(c => c.exchange === exchange);
    
    if (config) {
      console.log(`📡 Using ${exchange.toUpperCase()} API at port ${config.port}`);
      return `http://localhost:${config.port}`;
    }
  }
  
  // Fallback to environment variables or defaults
  const defaultPorts = {
    nse: process.env.NEXT_PUBLIC_NSE_API_PORT || '3001',
    mcx: process.env.NEXT_PUBLIC_MCX_API_PORT || '3002'
  };
  
  console.log(`📡 Using ${exchange.toUpperCase()} API at default port ${defaultPorts[exchange]}`);
  return `http://localhost:${defaultPorts[exchange]}`;
}

// Get the appropriate database instance
export async function getDatabase() {
  const info = await getPlatformInfo();
  
  if (info.is_tauri) {
    const { DataStorageTauri, DB_KEYS } = await import('./db_tauri');
    const dbInstance = new DataStorageTauri();
    return { db: dbInstance, DB_KEYS };
  } else {
    const { db, DB_KEYS } = await import('./db');
    return { db, DB_KEYS };
  }
}