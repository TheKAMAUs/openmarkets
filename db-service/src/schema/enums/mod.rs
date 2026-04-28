use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"market_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum MarketStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN = 1,
    #[serde(rename = "closed")]
    CLOSED = 2,
    #[serde(rename = "settled")]
    SETTLED = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"outcome\"")]
#[sqlx(rename_all = "lowercase")]
pub enum Outcome {
    #[serde(rename = "yes")]
    YES = 1,
    #[serde(rename = "no")]
    NO = 2,
    #[default]
    #[serde(rename = "unspecified")]
    UNSPECIFIED = 0,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"order_side\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderSide {
    #[default]
    #[serde(rename = "buy")]
    BUY = 1, // bids
    #[serde(rename = "sell")]
    SELL = 2, // asks
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN = 1,
    #[serde(rename = "filled")]
    FILLED = 2,
    #[serde(rename = "cancelled")]
    CANCELLED = 3,
    #[serde(rename = "expired")]
    EXPIRED = 4,
    #[serde(rename = "unspecified")]
    UNSPECIFIED = 5,
    #[sqlx(rename = "pending_update")]
    PendingUpdate = 6,
    #[sqlx(rename = "pending_cancel")]
    PendingCancel = 7,
    // NOT USED!!!! and DON'T USE IT
    #[sqlx(rename = "partial_fill")]
    PartialFill = 8,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionType {
    #[default]
    #[serde(rename = "deposit")]
    DEPOSIT = 1,
    #[serde(rename = "withdrawal")]
    WITHDRAWAL = 2,
    #[serde(rename = "trade")]
    TRADE = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionStatus {
    #[default]
    #[serde(rename = "pending")]
    PENDING = 1,
    #[serde(rename = "completed")]
    COMPLETED = 2,
    #[serde(rename = "failed")]
    FAILED = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderType {
    #[default]
    #[serde(rename = "limit")]
    LIMIT = 1,
    #[serde(rename = "market")]
    MARKET = 2,
    #[serde(rename = "stop_loss")]
    StopLoss = 3,
    #[serde(rename = "take_profit")]
    TakeProfit = 4,
}
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"verification_status\"")]
#[sqlx(rename_all = "snake_case")]  // Changed from lowercase to snake_case
pub enum VerificationStatus {
    #[default]
    #[serde(rename = "unverified")]
    Unverified = 1,
    #[serde(rename = "pending")]
    Pending = 2,
    #[serde(rename = "approved")]
    Approved = 3,
    #[serde(rename = "rejected")]
    Rejected = 4,
    #[serde(rename = "expired")]
    Expired = 5,
    #[serde(rename = "suspended")]
    Suspended = 6,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"verification_document_type\"")]
#[sqlx(rename_all = "snake_case")]  // Changed from lowercase to snake_case
pub enum VerificationDocumentType {
    #[default]
    #[serde(rename = "passport")]
    Passport = 1,
    #[serde(rename = "drivers_license")]
    DriversLicense = 2,
    #[serde(rename = "national_id")]
    NationalId = 3,
    #[serde(rename = "selfie")]
    Selfie = 4,
    #[serde(rename = "residence_permit")]
    ResidencePermit = 5,
    #[serde(rename = "proof_of_address")]
    ProofOfAddress = 6,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"verification_document_status\"")]
#[sqlx(rename_all = "snake_case")]  // Changed from lowercase to snake_case
pub enum VerificationDocumentStatus {
    #[default]
    #[serde(rename = "pending")]
    Pending = 1,
    #[serde(rename = "approved")]
    Approved = 2,
    #[serde(rename = "rejected")]
    Rejected = 3,
    #[serde(rename = "expired")]
    Expired = 4,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"verification_step\"")]  // Added quotes for consistency
#[sqlx(rename_all = "snake_case")]  // Already snake_case
pub enum VerificationStep {
    #[default]
    #[serde(rename = "identity_basic")]
    IdentityBasic,
    #[serde(rename = "document_upload")]
    DocumentUpload,
    #[serde(rename = "liveness_check")]
    LivenessCheck,
    #[serde(rename = "address_verification")]
    AddressVerification,
    #[serde(rename = "risk_assessment")]
    RiskAssessment,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"admin_verification_action\"")]
#[sqlx(rename_all = "snake_case")]  // Changed from lowercase to snake_case
pub enum AdminVerificationAction {
    #[default]
    #[serde(rename = "approved")]
    Approved = 1,
    #[serde(rename = "rejected")]
    Rejected = 2,
    #[serde(rename = "requested_revision")]  // Fixed: this was incorrectly named "Pending"
    RequestedRevision = 3,  // Fixed: variant name matches the rename
    #[serde(rename = "suspended")]
    Suspended = 4,
    #[serde(rename = "expired")]
    Expired = 5,
    #[serde(rename = "notes_added")]
    NotesAdded = 6,
}


#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"suggestion_status\"")]
#[sqlx(rename_all = "snake_case")]
pub enum SuggestionStatus {
    #[default]
    #[serde(rename = "pending")]
    Pending = 1,
    
    #[serde(rename = "approved")]
    Approved = 2,
    
    #[serde(rename = "rejected")]
    Rejected = 3,
    
    #[serde(rename = "in_review")]
    InReview = 4,
    
    #[serde(rename = "implemented")]
    Implemented = 5,
}
