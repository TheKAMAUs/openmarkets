import axios, { AxiosError } from "axios";
import jsCookies from "js-cookie";

import { marketServiceClient, priceServiceClient } from "../grpc/clients";
import {
  GetUserHoldingsResponse,
  GetUserMetadataResponse,
  GetUserOrdersPaginatedResponse,
  GetUserResponse,
  GetUserTradesResponse,
} from "../types/api";
import { OrderCategory } from "../types";

import { Timeframe } from "@/generated/grpc_service_types/common";
import {
  GetMarketTradesResponse,
  MarketStatus,
} from "@/generated/grpc_service_types/markets";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class MarketGetters {
  static async getMarketData(
    page: number,        // Changed from number to bigint
    pageSize: number,
    marketStatus: MarketStatus,
  ) {
    console.log("Requesting market data with parameters:", {
      page,
      pageSize,
      marketStatus,
    });

    try {
      const data = await marketServiceClient.getMarketData({
        pageRequest: {
          page: BigInt(page),        // Convert here
          pageSize: BigInt(pageSize), // Convert here
        },
        marketStatus,
      });

      console.log("Full backend response:", data);

      if (!data.response || !data.response.markets) {
        console.warn(
          "Warning: response.markets is missing or undefined",
          data.response
        );
        return [];
      }

      console.log(
        `Fetched ${data.response.markets.length} markets from backend.`
      );
      data.response.markets.forEach((m, i) =>
        console.log(`Market ${i}:`, m)
      );

      return data.response.markets;
    } catch (error: any) {
      console.error("Error fetching market data:", error);
      if (error.response) {
        console.error("Backend error response:", error.response);
      }
      return [];
    }
  }



  static async getMarketById(marketId: string) {
    try {
      const { response } = await marketServiceClient.getMarketById({
        marketId,
      });
      return response;
    } catch (error: any) {
      console.log("Failed to get market due to ", error);
      return null;
    }
  }

  static async getOrderBook(marketId: string, depth: number = 10) {
    try {
      const { response } = await marketServiceClient.getMarketBook({
        depth,
        marketId,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get order book: ", error);
      return null;
    }
  }

  static async getTopTenHolders(marketId: string) {
    try {
      const { response } = await marketServiceClient.getTopHolders({
        marketId,
      });
      return response.topHolders;
    } catch (error: any) {
      console.error("Failed to get top ten holders: ", error);
      return [];
    }
  }

  static async getMarketTrades({
    marketId,
    page,
    pageSize,
  }: {
    marketId: string;
    page: number;
    pageSize: number;
  }): Promise<GetMarketTradesResponse> {
    try {
      const { response } = await marketServiceClient.getMarketTrades({
        marketId,
        pageRequest: {
          page: BigInt(page),
          pageSize: BigInt(pageSize),
        },
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get market trades: ", error);
      return {
        trades: [],
        marketId: "",
        pageInfo: {
          page: BigInt(0),
          pageSize: BigInt(0),
          totalPages: BigInt(0),
          totalItems: BigInt(0),
        },
      };
    }
  }

  static async getBTCMarkets() {
    try {
      const { data } = await axios.get<GetBTCMarketsResponse>(
        `${BASE_URL}/btcusdt`, // adjust to your actual route
        {
          headers: {
            "Content-Type": "application/json",
            ...(TOKEN && { Authorization: `Bearer ${TOKEN}` }),
          },
        }
      );

      return data;
    } catch (error: any) {
      console.error("Failed to fetch BTC markets:", error);
      return null;
    }
  }





}


export class UserGetters {
  static async getUserData() {
    try {
      const { data, status } = await axios.get<GetUserResponse>(
        `${BASE_URL}/user/profile`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      if (status !== 200) {
        throw new Error("Failed to fetch user data");
      }
      return data;
    } catch (e: any) {
      console.log("Error fetching user data:", e);
      return null;
    }
  }

  static async getUserMetadata() {
    try {
      const { data } = await axios.get<GetUserMetadataResponse>(
        `${BASE_URL}/user/metadata`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user metadata: ", error);
      return null;
    }
  }

  static async getUserTrades(
    page: number,
    pageSize: number,
  ): Promise<GetUserTradesResponse> {
    try {
      const { data } = await axios.get<GetUserTradesResponse>(
        `${BASE_URL}/user/trades?page=${page}&pageSize=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user trades: ", error);
      return {
        data: {
          trades: [],
          page_info: {
            page: 0,
            page_size: 0,
            total_items: 0,
            total_pages: 0,
          },
        },
      };
    }
  }

  static async getUserHoldings(
    page: number,
    pageSize: number,
  ): Promise<GetUserHoldingsResponse> {
    try {
      const { data } = await axios.get<GetUserHoldingsResponse>(
        `${BASE_URL}/user/holdings?page=${page}&pageSize=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user holdings: ", error);
      return {
        data: {
          holdings: [],
          page_info: {
            page: 0,
            page_size: 0,
            total_items: 0,
            total_pages: 0,
          },
        },
      };
    }
  }
}

export class OrderGetters {
  static async getUserOrdersPaginated(page: number, pageSize: number) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get?page=${page}&page_size=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return { orders: [], page: 0, page_size: 0 };
    }
  }

  static async getUserOrdersByMarket(
    marketId: string,
    page: number,
    pageSize: number,
    orderType: OrderCategory = "all",
  ) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get/${marketId}?page=${page}&page_size=${pageSize}&status=${orderType}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return {
        orders: [],
        page: 0,
        page_size: 0,
        holdings: { no: "0", yes: "0" },
        total_pages: 0,
      };
    }
  }
}

export class ChartGetters {
  static async getChartDataWithinTimeRange(
    marketId: string,
    timeframe: Timeframe,
  ) {
    try {
      const { response } = await priceServiceClient.getPriceDataWithinInterval({
        marketId,
        timeframe,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get chart data: ", error);
      return {
        marketId: "",
        priceData: [],
      };
    }
  }
}


// types/verification.types.ts
export interface PendingDocument {
  id: string;
  type: 'passport' | 'drivers_license' | 'national_id' | 'residence_permit' | 'proof_of_address';
  url: string;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  uploaded_at: string;
  file_name: string | null;
  file_size: number | null;
  mime_type: string | null;
  rejection_reason: string | null;
}

export interface PendingVerificationUser {
  id: string;
  name: string;
  email: string;
  avatar: string;
  verified: boolean;
  verification_applied_at: string | null;
  verification_notes: string | null;
  verification_step: 'identity_basic' | 'document_upload' | 'completed';
  user_since: string;
  total_documents: number;
  pending_documents: number;
  approved_documents: number;
  days_pending: number;
  documents: PendingDocument[];
}

export interface PaginationMetadata {
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

export interface PendingVerificationsResponse {
  users: PendingVerificationUser[];
  pagination: PaginationMetadata;
}

export interface AdminActionPayload {
  user_id: string;
  action: 'approve' | 'reject' | 'request_revision';
  notes?: string;
  rejected_document_types?: string[];
  document_id?: string;  // Reference to the specific document being acted upon
}

export class VerificationService {
  static async getPendingVerifications(
    limit: number = 20,
    offset: number = 0
  ): Promise<PendingVerificationsResponse> {
    const response = await fetch(
      `${BASE_URL}/admin/verifications/pending?limit=${limit}&offset=${offset}`,
      {
        headers: {

          'Content-Type': 'application/json',
          Authorization: `Bearer ${TOKEN}`,
        },
      }
    );

    if (!response.ok) {
      throw new Error('Failed to fetch pending verifications');
    }

    return response.json();
  }

  static async approveUser(userId: string, notes?: string, documentId?: string): Promise<void> {
    const response = await fetch(`${BASE_URL}/admin/verifications/${userId}/approve`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${TOKEN}`,
      },
      body: JSON.stringify({ notes, document_id: documentId }),
    });

    if (!response.ok) {
      throw new Error('Failed to approve user');
    }
  }

  static async rejectUser(userId: string, reason: string): Promise<void> {
    const response = await fetch(`${BASE_URL}/admin/verifications/${userId}/reject`, {
      method: 'POST',
      headers: {

        'Content-Type': 'application/json',
        Authorization: `Bearer ${TOKEN}`,
      },
      body: JSON.stringify({ reason }),
    });

    if (!response.ok) {
      throw new Error('Failed to reject user');
    }
  }

  static async requestRevision(
    userId: string,
    notes: string,
    rejectedDocumentTypes: string[]
  ): Promise<void> {
    const response = await fetch(`${BASE_URL}/admin/verifications/${userId}/request-revision`, {
      method: 'POST',
      headers: {

        'Content-Type': 'application/json',
        Authorization: `Bearer ${TOKEN}`,
      },
      body: JSON.stringify({
        notes,
        rejected_document_types: rejectedDocumentTypes
      }),
    });

    if (!response.ok) {
      throw new Error('Failed to request revision');
    }
  }
}



// ==================== DISCUSSION TYPES ====================
export interface DiscussionResponse {
  id: string;
  market_id: string;
  user: {
    id: string | null;
    name: string;
    avatar: string | null;
  };
  content: string;
  upvotes: number;
  reply_count: number;
  created_at: string;
}

// ==================== SUGGESTION TYPES ====================
export interface SuggestionResponse {
  id: string;
  user_name: string;
  user_avatar: string | null;
  title: string;
  description: string;
  category: string | null;
  upvotes: number;
  status: string;
  admin_notes: string | null;
  created_at: string;
  updated_at: string;
  user_voted: boolean;
}

// ==================== DISCUSSION GET METHODS ====================
export class DiscussionAPI {
  // GET: Get discussions for a market
  static async getMarketDiscussions(marketId: string): Promise<DiscussionResponse[]> {
    try {
      const { data } = await axios.get<{ success: boolean; discussions: DiscussionResponse[] }>(
        `${BASE_URL}/user/messages/discussions/market/${marketId}`,
        {
          headers: {
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.discussions;
    } catch (error: any) {
      console.error("Error fetching discussions:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch discussions"
        );
      }
      throw new Error("Failed to fetch discussions");
    }
  }
}

// ==================== SUGGESTION GET METHODS ====================
export class SuggestionAPI {
  // GET: Get all suggestions (authenticated - shows user votes)
  static async getSuggestions(): Promise<SuggestionResponse[]> {
    try {
      const { data } = await axios.get<{ success: boolean; suggestions: SuggestionResponse[] }>(
        `${BASE_URL}/user/messages/suggestions`,
        {
          headers: {
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.suggestions;
    } catch (error: any) {
      console.error("Error fetching suggestions:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch suggestions"
        );
      }
      throw new Error("Failed to fetch suggestions");
    }
  }

  // GET: Get public suggestions (no auth needed)
  static async getPublicSuggestions(): Promise<SuggestionResponse[]> {
    try {
      const { data } = await axios.get<{ success: boolean; suggestions: SuggestionResponse[] }>(
        `${BASE_URL}/suggestions/public`
      );

      // Add user_voted: false to each suggestion
      const suggestionsWithVote = data.suggestions.map(suggestion => ({
        ...suggestion,
        user_voted: false  // 👈 Public users haven't voted
      }));

      return suggestionsWithVote;
    } catch (error: any) {
      console.error("Error fetching public suggestions:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch public suggestions"
        );
      }
      throw new Error("Failed to fetch public suggestions");
    }
  }

}

// ==================== TYPES ====================
export interface UserProfitRanking {
  user_id: string;
  name: string;
  email: string;
  avatar: string | null;
  net_profit: number;
  total_wins: number;
  total_losses: number;
  winning_trades: number;
  losing_trades: number;
  total_trades: number;
  win_rate: number;
  rank: number;
}

export interface LeaderboardResponse {
  success: boolean;
  winners: UserProfitRanking[];
  losers: UserProfitRanking[];
}

// ==================== API METHODS ====================
export class ProfitLossAPI {
  /**
   * GET /winners
   * Get top winners and losers leaderboard
   */
  // In your dataGetter.ts or wherever ProfitLossAPI is defined
  static async getLeaderboard(): Promise<LeaderboardResponse> {
    try {
      console.log("🔍 Fetching leaderboard from:", `${BASE_URL}/user/trades/winners`);

      const { data } = await axios.get<LeaderboardResponse>(
        `${BASE_URL}/user/trades/winners`,
        {
          headers: {
            Authorization: `Bearer ${TOKEN}`,
          },
        }

      );
      console.log("✅ Leaderboard response:", data);
      return data;
    } catch (error: any) {
      console.error("Error fetching leaderboard:", error);
      console.error("Response data:", error.response?.data);
      console.error("Response status:", error.response?.status);

      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch leaderboard"
        );
      }
      throw new Error("Failed to fetch leaderboard");
    }
  }
}

export interface Market {
  id: string;
  name: string;
  description: string;
  logo: string[];
  status: string;
  liquidity_b: string;
  final_outcome: string;
  market_expiry: string;
  created_at: string;
  updated_at: string;

  parent_id?: string;
  is_event: boolean;
  child_market_ids?: string[];
  category?: string;
  resolution_criteria?: string;
  slug?: string;

  q_yes: string;
  q_no: string;
}

export interface GetBTCMarketsResponse {
  message: string;
  data: Market[];
}

