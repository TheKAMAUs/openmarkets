
use axum::{
    Extension, Json,
    extract::{Query,Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use uuid::Uuid;
use serde_json::json;

use serde::{Deserialize, Serialize};

use crate::{
    state::AppState,
};

use utility_helpers::{log_error, log_info, log_warn};

use auth_service::types::SessionTokenClaims;
use db_service::schema::{
    enums::{SuggestionStatus},

};

use db_service::schema::suggestions::{SuggestionActions, CreateSuggestionRequest};



/// Get public suggestions (no auth required)
pub async fn get_public_suggestions_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    log_info!("Fetching public suggestions");
    
    match SuggestionActions::get_all(&state.pg_pool, None).await {
        Ok(suggestions) => {
            log_info!(
                count = suggestions.len(),
                "Public suggestions fetched successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "suggestions": suggestions
            }));
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                error = %e,
                "Failed to fetch public suggestions"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to fetch suggestions",
                  "details": format!("{}", e)  // Explicit formatting
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}