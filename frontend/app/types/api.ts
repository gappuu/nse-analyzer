export interface SecurityInfo {
  symbol: string;
  security_type: string;
}

export interface SecurityListResponse {
  indices: SecurityInfo[];
  equities: Record<string, SecurityInfo[]>;
}

export interface ContractInfoResponse {
  symbol: string;
  expiry_dates: string[];
  strike_prices: string[];
}

export interface ProcessedOptionDetail {
  strikePrice?: number;
  underlyingValue?: number;
  openInterest?: number;
  changeinOpenInterest?: number;
  lastPrice?: number;
  change?: number;
  pchange?: number;
  pchangeinOpenInterest?: number;
  the_money: string;
  tambu?: string;
  time_val: number;
  days_to_expiry: number;
  oiRank?: number;
}

export interface ProcessedOptionData {
  expiryDates?: string;
  strikePrice?: number;
  CE?: ProcessedOptionDetail;
  PE?: ProcessedOptionDetail;
  days_to_expiry: number;
}

export interface AlertValues {
  pchange_in_oi?: number;
  last_price?: number;
  open_interest?: number;
  the_money?: string;
  time_val: number;
  days_to_expiry: number;
}

export interface Alert {
  symbol: string;
  strike_price: number;
  expiry_date: string;
  option_type: string;
  alert_type: string;
  description: string;
  spread: number;
  values: AlertValues;
}

export interface RulesOutput {
  symbol: string;
  timestamp: string;
  underlying_value: number;
  alerts: Alert[];
}

export interface SingleAnalysisResponse {
  symbol: string;
  timestamp: string;
  underlying_value: number;
  spread: number;
  days_to_expiry: number;
  ce_oi: number;
  pe_oi: number;
  processed_data: ProcessedOptionData[];
  alerts?: RulesOutput;
}

export interface BatchSummary {
  total_securities: number;
  successful: number;
  failed: number;
  securities_with_alerts: number;
  total_alerts: number;
  processing_time_ms: number;
}

export interface BatchAnalysisResponse {
  summary: BatchSummary;
  rules_output: RulesOutput[];
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  processing_time_ms?: number;
  // Cache-related fields
  fromCache?: boolean;
  cachedAt?: string;
  lastUpdated?: number;
}

// UI State interfaces for cache management
export interface DataWithAge<T> {
  data: T;
  age: string;
  lastUpdated: number;
  fromCache: boolean;
}

export interface CacheInfo {
  hasCache: boolean;
  age?: string;
  lastUpdated?: number;
}