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


/// Create a new market suggestion
pub async fn create_suggestion_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(req): Json<CreateSuggestionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        user_id = %user_id,
        title = %req.title,
        "Creating new market suggestion"
    );
    
    match SuggestionActions::create(&state.pg_pool, req, user_id).await {
        Ok(suggestion) => {
            log_info!(
                suggestion_id = %suggestion.id,
                user_id = %user_id,
                "Suggestion created successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "suggestion": suggestion
            }));
            
            Ok((StatusCode::CREATED, response.into_response()))
        }
        Err(e) => {
            log_error!(
                user_id = %user_id,
                error = %e,
                "Failed to create suggestion"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to create suggestion",
                  "details": format!("{}", e)  // Explicit formatting
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}

/// Get all suggestions (with optional user vote status)
pub async fn get_suggestions_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        user_id = %user_id,
        "Fetching all suggestions"
    );
    
    match SuggestionActions::get_all(&state.pg_pool, Some(user_id)).await {
        Ok(suggestions) => {
            log_info!(
                count = suggestions.len(),
                "Suggestions fetched successfully"
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
                "Failed to fetch suggestions"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to fetch suggestions",
                 "details": format!("{}", e)  // Explicit formatting
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}

///moved to sugg.rs

// /// Get public suggestions (no auth required)
// pub async fn get_public_suggestions_handler(
//     State(state): State<AppState>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     log_info!("Fetching public suggestions");
    
//     match SuggestionActions::get_all(&state.pg_pool, None).await {
//         Ok(suggestions) => {
//             log_info!(
//                 count = suggestions.len(),
//                 "Public suggestions fetched successfully"
//             );
            
//             let response = Json(json!({
//                 "success": true,
//                 "suggestions": suggestions
//             }));
            
//             Ok((StatusCode::OK, response.into_response()))
//         }
//         Err(e) => {
//             log_error!(
//                 error = %e,
//                 "Failed to fetch public suggestions"
//             );
            
//             let error_response = serde_json::json!({
//                 "error": "Failed to fetch suggestions",
//                   "details": format!("{}", e)  // Explicit formatting
//             }).to_string();
            
//             Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
//         }
//     }
// }



/// Upvote a suggestion
pub async fn upvote_suggestion_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(suggestion_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        suggestion_id = %suggestion_id,
        user_id = %user_id,
        "Upvoting suggestion"
    );
    
    match SuggestionActions::upvote(&state.pg_pool, suggestion_id, user_id).await {
        Ok(_) => {
            log_info!(
                suggestion_id = %suggestion_id,
                user_id = %user_id,
                "Suggestion upvoted successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "message": "Suggestion upvoted successfully"
            }));
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                suggestion_id = %suggestion_id,
                user_id = %user_id,
                error = %e,
                "Failed to upvote suggestion"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to upvote suggestion",
                  "details": format!("{}", e)  // Explicit formatting
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}

/// Remove upvote from a suggestion
pub async fn remove_suggestion_upvote_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(suggestion_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        suggestion_id = %suggestion_id,
        user_id = %user_id,
        "Removing upvote from suggestion"
    );
    
    match SuggestionActions::remove_upvote(&state.pg_pool, suggestion_id, user_id).await {
        Ok(_) => {
            log_info!(
                suggestion_id = %suggestion_id,
                user_id = %user_id,
                "Upvote removed successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "message": "Upvote removed successfully"
            }));
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                suggestion_id = %suggestion_id,
                user_id = %user_id,
                error = %e,
                "Failed to remove upvote"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to remove upvote",
                 "details": format!("{}", e)  // Explicit formatting
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}



// moved to admin messages

// /// Update suggestion status (admin only)
// pub async fn update_suggestion_status_handler(
//     State(state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
//     Path(suggestion_id): Path<Uuid>,
//     Json(req): Json<UpdateStatusRequest>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     let admin_id = claims.user_id;
    
//     // Check if user is admin (you need to implement this)
//     if !claims.is_admin {
//         log_warn!(
//             user_id = %admin_id,
//             "Non-admin attempted to update suggestion status"
//         );
        
//         let error_response = serde_json::json!({
//             "error": "Unauthorized: Admin access required"
//         }).to_string();
        
//         return Err((StatusCode::FORBIDDEN, error_response.into_response()));
//     }
    
//     log_info!(
//         suggestion_id = %suggestion_id,
//         admin_id = %admin_id,
//         new_status = ?req.status,
//         "Updating suggestion status"
//     );
    
//     match SuggestionActions::update_status(
//         &state.pg_pool, 
//         suggestion_id, 
//         admin_id, 
//         req.status,
//         req.admin_notes
//     ).await {
//         Ok(suggestion) => {
//             log_info!(
//                 suggestion_id = %suggestion_id,
//                 status = ?suggestion.status,
//                 "Suggestion status updated successfully"
//             );
            
//             let response = Json(json!({
//                 "success": true,
//                 "suggestion": suggestion
//             }));
            
//             Ok((StatusCode::OK, response.into_response()))
//         }
//         Err(e) => {
//             log_error!(
//                 suggestion_id = %suggestion_id,
//                 admin_id = %admin_id,
//                 error = %e,
//                 "Failed to update suggestion status"
//             );
            
//             let error_response = serde_json::json!({
//                 "error": "Failed to update suggestion status",
//                    "details": format!("{}", e)  // Explicit formatting
//             }).to_string();
            
//             Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
//         }
//     }
// }

// // Request struct for status update
// #[derive(Debug, Deserialize)]
// pub struct UpdateStatusRequest {
//     pub status: SuggestionStatus,
//     pub admin_notes: Option<String>,
// }