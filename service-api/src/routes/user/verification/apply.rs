use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use auth_service::types::SessionTokenClaims;
use db_service::schema::verification::{VerificationService, ApplyVerificationPayload};  // Use directly
use crate::{
    state::AppState,
};
use utility_helpers::{log_error, log_info};

// Remove the local ApplyVerificationPayload definition - use db_service's one

#[derive(Debug, Serialize)]
pub struct ApplyVerificationResponse {
    pub message: String,
    pub status: String,
    pub applied_at: String,
}

pub async fn apply_for_verification(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<ApplyVerificationPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    // FIX: Use structured logging syntax
    log_info!(
        user_id = %user_id,
        full_name = %payload.full_name,
        country = %payload.country_of_residence,
        "User applying for verification"
    );
    
    match VerificationService::apply_for_verification(
        &state.pg_pool,
        user_id,
        payload,
    )
    .await
    {
        Ok(_) => {
            log_info!(
                user_id = %user_id,
                "Verification application submitted successfully"
            );
            
            let response = Json(ApplyVerificationResponse {
                message: "Verification application submitted successfully".to_string(),
                status: "pending".to_string(),
                applied_at: chrono::Utc::now().to_rfc3339(),
            });
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                user_id = %user_id,
                error = %e,
                "Failed to submit verification application"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to submit verification application",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}