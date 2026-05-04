import axios, { AxiosError } from "axios";
import jsCookies from "js-cookie";

import { LoginResponse } from "../types/api";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class UserAuthActions {
  static async handleSignInWithGoogle({
    id_token,
    referralCode,
  }: {
    id_token: string;
    referralCode?: string;
  }) {
    const { data, status } = await axios.post(`${BASE_URL}/login`, {
      id_token,
      ...(referralCode ? { referralCode } : {}), // only include if exists
    });

    if (status != 200) throw new Error(data.error);
    return data as LoginResponse;
  }
}


// types/market.ts or in your dataPosters.ts
export interface CreateChildMarketRequest {
  name?: string;
  question?: string;
  logo?: string[];  // Changed to array
  liquidity?: number;
  market_expiry?: string;
  slug?: string;
  category?: string;
  resolution_criteria?: string;
}

export interface CreateMarketRequest {
  name?: string;
  description?: string;
  logo?: string[];  // Changed to array
  liquidity_b?: number;
  market_expiry?: string;
  slug?: string;
  is_event?: boolean;
  child_markets?: CreateChildMarketRequest[];
  category?: string;
  resolution_criteria?: string;
}



interface ValidateTradeRequest {
  market_id: string;
  amount: number;
  outcome: boolean;  // true = yes, false = no
}

interface ValidateTradeResponse {
  market_id: string;
  amount_staked: number;
  outcome: string;
  shares_acquired: number;
  potential_profit: number;
  max_allowed_profit: number;
  is_valid: boolean;
  message: string;
}

export class MarketActions {
  static async createLimitOrder(reqPayload: {
    market_id: string;
    price: number;
    quantity: number;
    side: "buy" | "sell";
    outcome_side: "yes" | "no";
  }) {
    try {
      await axios.post(`${BASE_URL}/user/orders/create/limit`, reqPayload, {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${TOKEN}`,
        },
      });
    } catch (error: any) {
      console.error("Error creating limit order:", error);
      if (error instanceof AxiosError) {
        console.log("Axios error details:", error.response?.data);
        throw new Error(
          error.response?.data?.error || "Failed to create limit order",
        );
      }
      throw new Error("Failed to create limit order");
    }
  }

  static async updateOrder(payload: {
    order_id: string;
    new_quantity: number;
    new_price: number;
  }) {
    try {
      const { data } = await axios.patch(
        `${BASE_URL}/user/orders/update`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Error updating order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to update order",
        );
      }
      throw new Error("Failed to update order");
    }
  }

  static async cancelOrder(orderId: string) {
    try {
      const { data } = await axios.delete(
        `${BASE_URL}/user/orders/cancel/${orderId}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Error canceling order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to cancel order",
        );
      }
      throw new Error("Failed to cancel order");
    }
  }

  static async createMarketOrder(reqPayload: {
    market_id: string;
    outcome: "yes" | "no";
    side: "buy" | "sell";
    amount_spent: number;
    price_at_execution: number;
    shares_to_sell?: number;        // optional — only required when side is "sell"
  }) {
    try {
      await axios.post(`${BASE_URL}/user/orders/create/market`, reqPayload, {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${TOKEN}`,
        },
      });
    } catch (error: any) {
      console.error("Error creating market order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create market order",
        );
      }
      throw new Error("Failed to create market order");
    }
  }


  // In your MarketActions class/file
  static async validateTrade(reqPayload: {
    market_id: string;
    outcome: "yes" | "no";
    amount: number;
  }) {
    try {
      const payload = {
        market_id: reqPayload.market_id,
        amount: reqPayload.amount,
        outcome: reqPayload.outcome === "yes",
      };

      const response = await axios.post(`${BASE_URL}/quote`, payload, {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${TOKEN}`,
        },
      });

      return response.data;
    } catch (error: any) {
      console.error("Error validating trade:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to validate trade"
        );
      }
      throw new Error("Failed to validate trade");
    }
  }
  /** Creates a new market (admin only) */


  // In your MarketActions class
  static async createMarket(reqPayload: CreateMarketRequest) {

    try {

      const { data } = await axios.post(
        `${BASE_URL}/admin/market/create`,
        reqPayload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data; // <-- return created market details
    } catch (error: any) {
      console.error("Error creating market:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create market"
        );
      }
      throw new Error("Failed to create market");
    }
  }


  static async initializeMarket(payload: {
    market_id: string;
    depth: number;
    quantity: number;
  }) {
    try {

      const { data } = await axios.post(
        `${BASE_URL}/admin/market/initialize`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error initializing market:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to initialize market"
        );
      }
      throw new Error("Failed to initialize market");
    }
  }

  static async finalizeMarket(payload: {
    market_id: string;
    final_outcome: "yes" | "no";
  }) {
    console.log("🟦 FINALIZE MARKET REQUEST INITIATED");
    console.log("➡️ URL:", `${BASE_URL}/admin/market/finalize`);
    console.log("➡️ Payload:", payload);
    console.log("➡️ Headers:", {
      "Content-Type": "application/json",
      Authorization: `Bearer ${TOKEN}`,
    });

    try {
      const { data } = await axios.post(
        `${BASE_URL}/admin/market/finalize`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );

      console.log("🟩 FINALIZE MARKET RESPONSE SUCCESS:", data);
      return data;
    } catch (error: any) {
      console.log("🟥 FINALIZE MARKET REQUEST FAILED");

      // Axios error details
      if (error.response) {
        console.log("🔻 Axios Response Error:");
        console.log("   Status:", error.response.status);
        console.log("   Data:", error.response.data);
        console.log("   Headers:", error.response.headers);

        throw new Error(
          error.response.data?.error || error.response.data?.message || "Failed to finalize market"
        );
      }

      // Request was made but no response
      if (error.request) {
        console.log("🔻 Axios Request Error: No response received");
        console.log("   Request:", error.request);
        throw new Error("No response received from server");
      }

      // Unknown error
      console.log("🔻 Unknown Error:", error.message);
      throw new Error("Failed to finalize market");
    }
  }

}



interface DepositPayload {
  amount: number;
  phone_number: string;
  account_reference: string;
}

interface WithdrawPayload {
  amount: number;
  phoneNumber: string;
}

export default class UserService {
  static async depositFunds(payload: DepositPayload) {
    try {
      const { data } = await axios.post(
        `${BASE_URL}/user/deposit`, // your deposit endpoint
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`, // same auth as updateOrder
          },
        }
      );

      return data;
    } catch (error: any) {
      console.error("Error depositing funds:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to deposit funds"
        );
      }
      throw new Error("Failed to deposit funds");
    }
  }


  // ⭐ NEW: Withdrawal Method
  static async withdrawFunds(payload: WithdrawPayload) {


    try {
      const { data } = await axios.post(
        `${BASE_URL}/user/withdrawal`, // your Rust B2C route
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );

      return data;
    } catch (error: any) {
      console.error("Error withdrawing funds:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to withdraw funds"
        );
      }
      throw new Error("Failed to withdraw funds");
    }
  }

}



// Types - Use snake_case to match backend
export interface ApplyVerificationPayload {
  full_name: string;           // Changed from fullName
  date_of_birth: string;       // Changed from dateOfBirth
  country_of_residence: string; // Changed from countryOfResidence
}


// In your VerificationService
export interface SubmitVerificationPayload {
  // Address
  address: string;
  city: string;
  postal_code: string;  // snake_case to match backend

  // Risk assessment
  trading_experience: string;  // snake_case
  annual_income: string;       // snake_case
  source_of_funds: string;     // snake_case

  // Documents
  documents: Array<{
    document_type: string;   // snake_case
    document_url: string;    // snake_case
    file_name?: string;      // snake_case
    file_size?: number;      // snake_case
    mime_type?: string;      // snake_case
  }>;

  agreed_to_terms: boolean;    // snake_case
}


export interface DocumentResponse {
  id: string;
  documentType: string;
  status: string;
  uploadedAt: string;
  documentUrl: string;
}

export interface VerificationProgressResponse {
  currentStep: string;
  stepsCompleted: string[];
  documentsStatus: Array<{
    documentType: string;
    status: string;
    uploadedAt: string | null;
    rejectionReason: string | null;
  }>;
  missingRequirements: string[];
}

export class VerificationService {
  // Apply for verification
  static async applyForVerification(payload: ApplyVerificationPayload) {
    try {
      const { data } = await axios.post(
        `${BASE_URL}/user/verification/apply`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error applying for verification:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to start verification"
        );
      }
      throw new Error("Failed to start verification");
    }
  }



  // Get user documents
  static async getUserDocuments() {
    try {
      const { data } = await axios.get(
        `${BASE_URL}/user/verification/documents`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error fetching documents:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch documents"
        );
      }
      throw new Error("Failed to fetch documents");
    }
  }

  // Delete document
  static async deleteDocument(documentId: string) {
    try {
      const { data } = await axios.delete(
        `${BASE_URL}/user/verification/documents/${documentId}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error deleting document:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to delete document"
        );
      }
      throw new Error("Failed to delete document");
    }
  }

  // Submit complete verification
  static async submitVerification(payload: SubmitVerificationPayload) {
    try {
      const { data } = await axios.post(
        `${BASE_URL}/user/verification/submit`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error submitting verification:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to submit verification"
        );
      }
      throw new Error("Failed to submit verification");
    }
  }

  // Get verification progress
  static async getVerificationProgress() {
    try {
      const { data } = await axios.get<VerificationProgressResponse>(
        `${BASE_URL}/user/verification/progress`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data;
    } catch (error: any) {
      console.error("Error fetching progress:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to fetch progress"
        );
      }
      throw new Error("Failed to fetch progress");
    }
  }
}




// ==================== DISCUSSION TYPES ====================
// ==================== DISCUSSION TYPES ====================
export interface CreateDiscussionRequest {
  market_id: string;
  content: string;
  parent_id?: string | null;
}

export interface Discussion {
  id: string;
  market_id: string;
  user_id: string;
  parent_id: string | null;
  content: string;
  upvotes: number;
  created_at: string;
  updated_at: string;
}

export interface DiscussionResponse {
  id: string;
  market_id: string;
  user_name: string;
  user_avatar: string | null;
  content: string;
  upvotes: number;
  reply_count: number;
  created_at: string;
}

// ==================== SUGGESTION TYPES ====================
export interface CreateSuggestionRequest {
  title: string;
  description: string;
  category?: string | null;
}

export interface Suggestion {
  id: string;
  user_id: string;
  title: string;
  description: string;
  category: string | null;
  upvotes: number;
  status: string;
  admin_notes: string | null;
  created_at: string;
  updated_at: string;
}

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

export interface UpdateStatusRequest {
  status: 'pending' | 'approved' | 'rejected' | 'in_review' | 'implemented';
  admin_notes?: string;
}


// ==================== DISCUSSION API METHODS ====================
export class DiscussionAPII {
  // POST: Create a new discussion
  static async createDiscussion(req: CreateDiscussionRequest): Promise<Discussion> {
    try {
      const { data } = await axios.post<{ success: boolean; discussion: Discussion }>(
        `${BASE_URL}/user/messages/discussions`,
        req,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.discussion;
    } catch (error: any) {
      console.error("Error creating discussion:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create discussion"
        );
      }
      throw new Error("Failed to create discussion");
    }
  }

  // GET: Get discussions for a market
  static async getMarketDiscussions(marketId: string): Promise<DiscussionResponse[]> {
    try {
      const { data } = await axios.get<{ success: boolean; discussions: DiscussionResponse[] }>(
        `${BASE_URL}/user/messages/discussions/market/${marketId}`,
        {
          headers: {
            "Content-Type": "application/json",
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

  // POST: Upvote a discussion
  static async upvoteDiscussion(discussionId: string): Promise<void> {
    try {
      await axios.post(
        `${BASE_URL}/user/messages/discussions/${discussionId}/upvote`,
        {},
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
    } catch (error: any) {
      console.error("Error upvoting discussion:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to upvote discussion"
        );
      }
      throw new Error("Failed to upvote discussion");
    }
  }

  // DELETE: Remove upvote from discussion
  static async removeDiscussionUpvote(discussionId: string): Promise<void> {
    try {
      await axios.delete(
        `${BASE_URL}/user/messages/discussions/${discussionId}/upvote`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
    } catch (error: any) {
      console.error("Error removing upvote:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to remove upvote"
        );
      }
      throw new Error("Failed to remove upvote");
    }
  }
}

// ==================== SUGGESTION API METHODS ====================
export class SuggestionAPII {
  // POST: Create a new suggestion
  static async createSuggestion(req: CreateSuggestionRequest): Promise<Suggestion> {
    try {
      const { data } = await axios.post<{ success: boolean; suggestion: Suggestion }>(
        `${BASE_URL}/user/messages/suggestions`,
        req,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.suggestion;
    } catch (error: any) {
      console.error("Error creating suggestion:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create suggestion"
        );
      }
      throw new Error("Failed to create suggestion");
    }
  }



  // POST: Upvote a suggestion
  static async upvoteSuggestion(suggestionId: string): Promise<void> {
    try {
      await axios.post(
        `${BASE_URL}/user/messages/suggestions/${suggestionId}/upvote`,
        {},
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
    } catch (error: any) {
      console.error("Error upvoting suggestion:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to upvote suggestion"
        );
      }
      throw new Error("Failed to upvote suggestion");
    }
  }

  // DELETE: Remove upvote from suggestion
  static async removeSuggestionUpvote(suggestionId: string): Promise<void> {
    try {
      await axios.delete(
        `${BASE_URL}/user/messages/suggestions/${suggestionId}/upvote`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
    } catch (error: any) {
      console.error("Error removing upvote:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to remove upvote"
        );
      }
      throw new Error("Failed to remove upvote");
    }
  }

  // PATCH: Update suggestion status (admin only)
  static async updateSuggestionStatus(
    suggestionId: string,
    req: UpdateStatusRequest
  ): Promise<Suggestion> {
    try {
      const { data } = await axios.patch<{ success: boolean; suggestion: Suggestion }>(
        `${BASE_URL}/admin/messages/suggestions/${suggestionId}/status`,
        req,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.suggestion;
    } catch (error: any) {
      console.error("Error updating suggestion status:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to update suggestion status"
        );
      }
      throw new Error("Failed to update suggestion status");
    }
  }
}


/// ==================== LIQUIDITY TYPES ====================
export interface AddLiquidityRequest {
  market_id: string;
  amount: number;  // Amount in KES to add as liquidity
}

export interface LiquidityPosition {
  lp_position_id: number;
  user_id: string;
  market_id: string;
  amount_deposited: number;
  shares_of_pool: number;
  fees_earned: number;
  withdrawn_amount: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface AddLiquidityResponse {
  message: string;
  data: LiquidityPosition;
}

export interface RemoveLiquidityRequest {
  lp_position_id: number;
  withdraw_amount: number;
}

export interface FeeEarningsResponse {
  message: string;
  data: {
    total_fees_earned: number;
    positions: LiquidityPosition[];
  };
}

// ==================== LIQUIDITY API METHODS ====================
export class LiquidityService {
  // POST: Add liquidity to a market
  static async addLiquidity(req: AddLiquidityRequest): Promise<LiquidityPosition> {
    try {

      const { data } = await axios.post<AddLiquidityResponse>(
        `${BASE_URL}/user/trades/liquidity/add`,
        req,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data;
    } catch (error: any) {
      console.error("Error adding liquidity:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to add liquidity"
        );
      }
      throw new Error("Failed to add liquidity");
    }
  }

  // POST: Remove liquidity from a market
  static async removeLiquidity(req: RemoveLiquidityRequest): Promise<LiquidityPosition> {
    try {

      const { data } = await axios.post<AddLiquidityResponse>(
        `${BASE_URL}/user/trades/liquidity/remove`,
        req,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data;
    } catch (error: any) {
      console.error("Error removing liquidity:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to remove liquidity"
        );
      }
      throw new Error("Failed to remove liquidity");
    }
  }

  // GET: Get user's fee earnings
  static async getFeeEarnings(market_id?: string): Promise<FeeEarningsResponse['data']> {
    try {

      const url = market_id
        ? `${BASE_URL}/user/trades/liquidity/fees?market_id=${market_id}`
        : `${BASE_URL}/user/trades/liquidity/fees`;

      const { data } = await axios.get<FeeEarningsResponse>(
        url,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data;
    } catch (error: any) {
      console.error("Error getting fee earnings:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to get fee earnings"
        );
      }
      throw new Error("Failed to get fee earnings");
    }
  }

  // GET: Get user's total fee earnings (simple version)
  static async getTotalFeeEarnings(): Promise<number> {
    try {

      const { data } = await axios.get<{ data: { total_fees_earned: number } }>(
        `${BASE_URL}/liquidity/fees/total`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data.total_fees_earned;
    } catch (error: any) {
      console.error("Error getting total fee earnings:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to get total fee earnings"
        );
      }
      throw new Error("Failed to get total fee earnings");
    }
  }

  // GET: Get user's liquidity positions
  static async getLiquidityPositions(): Promise<LiquidityPosition[]> {
    try {

      const { data } = await axios.get<{ data: LiquidityPosition[] }>(
        `${BASE_URL}/user/trades/liquidity/fees`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data;
    } catch (error: any) {
      console.error("Error getting liquidity positions:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to get liquidity positions"
        );
      }
      throw new Error("Failed to get liquidity positions");
    }
  }

  // GET: Get pool stats for a market
  static async getPoolStats(market_id: string): Promise<any> {
    try {

      const { data } = await axios.get(
        `${BASE_URL}/liquidity/pool/${market_id}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        }
      );
      return data.data;
    } catch (error: any) {
      console.error("Error getting pool stats:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.message || "Failed to get pool stats"
        );
      }
      throw new Error("Failed to get pool stats");
    }
  }
}