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
  latest_future_expiry?: string;
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

// MCX Futures Quote Response Structure (Raw API response)
export interface McxFutureQuoteApiResponse {
  success: boolean;
  data: {
    d: {
      Data: Array<{
        AbsoluteChange: number;
        ChangeInOpenInterest: number;
        ExpiryDate: string;
        OpenInterest: number;
        PercentChange: number;
        PreviousClose: number;
        TradingUnit: string;
        Productdesc: string;
        LifeTimeHigh: number;
        AveragePrice: number;
        LifeTimeLow: number;
        LTP: number;
        action?: string;
        pchangeinOpenInterest?: number;
      }>;
      Summary: {
        AsOn: string; // Format: "/Date(1766168956660)/"
      };
    };
  };
  error: string | null;
  processing_time_ms: number;
}

// MCX Futures Quote Response Structure (For API client)
export interface McxFutureQuoteResponse {
  d: {
    Data: Array<{
      AbsoluteChange: number;
      ChangeInOpenInterest: number;
      ExpiryDate: string;
      OpenInterest: number;
      PercentChange: number;
      PreviousClose: number;
      TradingUnit: string;
      Productdesc: string;
      LifeTimeHigh: number;
      AveragePrice: number;
      LifeTimeLow: number;
      Category: string;
      LTP: number;
      action?: string;
      pchangeinOpenInterest?: number;
    }>;
    Summary: {
      AsOn: string;
    };
  };
}

// MCX Future Analysis Helper Interface
export interface McxFutureAnalysis {
  action?: string;
  pchangeinOpenInterest?: number;
  underlyingValue: number;
  timestamp: string;
  lastPrice: number;
  openInterest: number;
  changeinOpenInterest: number;
  expiryDate?: string;
  percentChange?: number;
  absoluteChange?: number;
  previousClose?: number;
  asOnTimestamp?: string;
  Productdesc?: string;
  LifeTimeHigh?: number;
  AveragePrice?: number;
  LifeTimeLow?: number;
  Category?: string;
  TradingUnit?: string;
  LTP?: number;
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

// MCX Historical Data Request Parameters
export interface McxHistoricalDataParams {
  symbol: string;
  expiry: string;
  from_date: string;
  to_date: string;
  instrument_name: 'FUTCOM' | 'OPTFUT';
  option_type?: 'CE' | 'PE';
  strike_price?: string;
}

// MCX Historical Data Response Structure (Updated to match actual API response)
export interface McxHistoricalDataResponse {
  d: {
    Data: Array<{
      ChangeInOI:number;
      Close: number;
      Date: string;
      DateDisplay: string;
      ExpiryDate: string;
      High: number;
      InstrumentName: string;
      Low: number;
      Open: number;
      OpenInterest: number;
      OptionType: string;
      PreviousClose: number;
      StrikePrice: number;
      Symbol: string;
      Value: number;
      Volume: number;
      VolumeInThousands: string;
      __type: string;
    }>;
    Summary: {
      AsOn: string;
      Count: number;
      Status: string | null;
    };
  };
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