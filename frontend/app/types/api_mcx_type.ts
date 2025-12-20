// MCX API Response Types
export interface McxApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  fromCache?: boolean;
  cachedAt?: string;
  lastUpdated?: number;
}

export interface McxTickersResponse {
  InstrumentName: string;
  Symbols: Array<{
    ExpiryDates: string[];
    SymbolValue: string;
  }>;
}

export interface McxFutureSymbolsResponse {
  InstrumentName: string;
  Products: Array<{
    ExpiryDates: string[];
    Product: string;
  }>;
}

export interface McxOptionChainResponse {
  symbol: string;
  timestamp: string;
  underlyingValue: number; // camelCase as per API
  spread: number;
  days_to_expiry: number;
  ce_oi: number;
  pe_oi: number;
  processed_data: ProcessedOptionData[];
  alerts?: RulesOutput;
}

export interface ProcessedOptionData {
  expiryDates?: string;
  strikePrice?: number;
  CE?: ProcessedOptionDetail;
  PE?: ProcessedOptionDetail;
  days_to_expiry: number;
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

export interface RulesOutput {
  symbol: string;
  timestamp: string;
  underlyingValue: number; // camelCase as per API
  alerts: Alert[];
}

export interface Alert {
  symbol: string;
  strikePrice: number;
  expiryDates: string;
  option_type: string;
  alert_type: string;
  description: string;
  spread: number;
  values: AlertValues;
}

export interface AlertValues {
  pchangeinOpenInterest?: number;
  lastPrice?: number;
  openInterest?: number;
  the_money?: string;
  time_val: number;
  days_to_expiry: number;
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

// MCX Futures Quote Response Structure
export interface McxFutureQuoteResponse {
  symbol: string;
  timestamp: string;
  data: Array<{
    symbol: string;
    underlyingValue: number;
    lastPrice: number;
    change: number;
    pchange: number;
    openInterest: number;
    changeinOpenInterest: number;
    pchangeinOpenInterest: number;
  }>;
}

// MCX Future Analysis Helper Interface
export interface McxFutureAnalysis {
  action: string;
  color: string;
  underlyingValue: number;
  timestamp: string;
  lastPrice: number;
  openInterest: number;
  changeinOpenInterest: number;
}

// MCX Option Quote Response Structure  
export interface McxOptionQuoteResponse {
  symbol: string;
  timestamp: string;
  data: Array<{
    symbol: string;
    strikePrice: number;
    optionType: string;
    underlyingValue: number;
    lastPrice: number;
    change: number;
    pchange: number;
    openInterest: number;
    changeinOpenInterest: number;
    pchangeinOpenInterest: number;
  }>;
}

// MCX Batch Analysis Response Structure
export interface McxBatchAnalysisResponse {
  summary: BatchSummary;
  rules_output: RulesOutput[];
}

// MCX Historical Data Response Structure
export interface McxHistoricalDataResponse {
  symbol: string;
  instrumentType: string;
  expiry: string;
  data: Array<{
    FH_TIMESTAMP: string;
    FH_UNDERLYING_VALUE: number;
    FH_OPEN_INT: number;
    FH_CHANGE_IN_OI: number;
    FH_SETTLE_PRICE: number;
    FH_STRIKE_PRICE?: number;
    FH_OPTION_TYPE?: string;
  }>;
}

// Data with age interface for MCX
export interface McxDataWithAge<T> {
  data: T;
  age: string;
  lastUpdated: number;
  fromCache: boolean;
}

// Combined commodity data for UI
export interface CombinedCommodityData {
  optionExpiries: string[];
  futureExpiries: string[];
}