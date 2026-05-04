use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use auth_service::types::SessionTokenClaims;
use db_service::schema::verification::{
    VerificationService, 
    SubmitVerificationPayload,  // Use directly from db_service
    DocumentSubmission,          // Use directly from db_service
};

use crate::{
    state::AppState,
};
use utility_helpers::{log_error, log_info};

// Remove the local SubmitVerificationPayload and DocumentSubmission definitions
// They're now imported from db_service

#[derive(Debug, Serialize)]
pub struct SubmitVerificationResponse {
    pub message: String,
    pub status: String,
    pub submitted_at: String,
}

pub async fn submit_verification(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<SubmitVerificationPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    // FIX: Use structured logging syntax
    log_info!(
        user_id = %user_id,
        address = %payload.address,
        city = %payload.city,
        document_count = payload.documents.len(),
        "User submitting verification"
    );
    
    match VerificationService::submit_verification(
        &state.pg_pool,
        user_id,
        payload,
    )
    .await
    {
        Ok(_) => {
            log_info!(
                user_id = %user_id,
                "Verification submitted successfully"
            );
            
            let response = Json(SubmitVerificationResponse {
                message: "Verification submitted successfully".to_string(),
                status: "pending".to_string(),
                submitted_at: chrono::Utc::now().to_rfc3339(),
            });
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                user_id = %user_id,
                error = %e,
                "Failed to submit verification"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to submit verification",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}