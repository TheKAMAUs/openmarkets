import { Order, PageInfoServiceAPi } from ".";

export interface BaseResponse {
  message: string;
  success: boolean;
}
export interface ErrorResponse {
  error: string;
}

export interface LoginResponse extends BaseResponse {
  userId: string;
  sessionToken: string;
}

export interface GetUserResponse {
  avatar: string;
  balance: number;
  email: string;
  name: string;
  public_key: string;
}

export interface GetUserOrdersPaginatedResponse {
  orders: Order[];
  page: number;
  page_size: number;
  total_pages: number;
  holdings: {
    no: string;
    yes: string;
  };
}

// types/user.ts

export interface GetUserMetadataResponse {
  profile_insight: {
    // Existing fields
    avatar: string;
    avg_fill_ratio: string;
    avg_trade_price: string;
    balance: string;
    created_at: string;
    email: string;
    first_trade_at: string;
    id: string;
    last_deposit: null;
    last_login: string;
    last_trade_at: string;
    last_withdraw: null;
    markets_traded: number;
    max_trade_qty: string;
    name: string;
    open_orders: number;
    partial_orders: number;
    public_key: string;
    total_deposit: null;
    total_orders: number;
    total_trades: number;
    total_volume: string;
    total_withdraw: null;
    verified: boolean;

    // New verification fields
    verification_status: 'unverified' | 'pending' | 'approved' | 'rejected' | 'expired' | 'suspended';
    verification_step: 'identity_basic' | 'document_upload' | 'completed';
    verification_applied_at: string | null;
    verification_reviewed_at: string | null;
    verified_at: string | null;
    verification_expires_at: string | null;
    verification_notes: string | null;
  };
  user_id: string;
}

export interface Trade {
  market_final_outcome: string;
  market_logo: string;
  market_name: string;
  market_status: string;
  trade_outcome: string;
  trade_price: string;
  trade_quantity: string;
  trade_type: string;
}

export interface GetUserTradesResponse {
  data: {
    page_info: PageInfoServiceAPi;
    trades: Trade[];
  };
}

// Holdings interface based on your data structure
interface Holding {
  final_outcome: string;
  market_created_at: string;
  market_description: string;
  market_expiry: string;
  market_id: string;
  market_logo: string;
  market_name: string;
  market_status: string;
  market_updated_at: string;
  outcome: string;
  shares: string;
}

export interface GetUserHoldingsResponse {
  data: {
    holdings: Holding[];
    page_info: PageInfoServiceAPi;
  };
}

export interface Market {
  id: string;
  name: string;
  description: string;
  logo: string[];
  status: MarketStatus;
  liquidity_b: number;
  final_outcome: Outcome | null;
  created_at: string;
  updated_at: string;
  market_expiry: string;
  yes_price: number;
  no_price: number;

  // New fields
  parent_id: string;
  is_event: boolean;
  child_markets: Market[];
  category: string;
  resolution_criteria: string;
  slug: string;
}

export enum MarketStatus {
  OPEN = 0,
  CLOSED = 1,
  SETTLED = 2,
}

export enum Outcome {
  YES = 0,
  NO = 1,
  UNRESOLVED = 2,
}




