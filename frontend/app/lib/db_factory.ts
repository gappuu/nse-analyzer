// Factory to get the correct database instance based on platform
import { isTauriPlatform } from './platform';

// Define a common interface that both storage classes implement
export interface IDataStorage {
  storeData<T>(key: string, data: T): Promise<void>;
  getData<T>(key: string): Promise<{ data: T; timestamp: string; lastUpdated: number } | null>;
  hasData(key: string): Promise<boolean>;
  getDataAge(lastUpdated: number, currentTime?: number): string;
  clearData(key: string): Promise<void>;
  clearAllData(): Promise<void>;
}

let dbInstance: IDataStorage | null = null;
let DB_KEYS_EXPORT: any = null;

export async function getDb(): Promise<{ db: IDataStorage; DB_KEYS: typeof DB_KEYS_EXPORT }> {
  if (dbInstance && DB_KEYS_EXPORT) {
    return { db: dbInstance, DB_KEYS: DB_KEYS_EXPORT };
  }

  const isTauri = await isTauriPlatform();
  
  if (isTauri) {
    const { DataStorageTauri, DB_KEYS } = await import('./db_tauri');
    dbInstance = new DataStorageTauri();
    DB_KEYS_EXPORT = DB_KEYS;
  } else {
    const { db, DB_KEYS } = await import('./db');
    dbInstance = db;
    DB_KEYS_EXPORT = DB_KEYS;
  }

  return { db: dbInstance, DB_KEYS: DB_KEYS_EXPORT };
}

// For convenience
export async function getDatabase(): Promise<IDataStorage> {
  const { db } = await getDb();
  return db;
}

export async function getDbKeys() {
  const { DB_KEYS } = await getDb();
  return DB_KEYS;
}