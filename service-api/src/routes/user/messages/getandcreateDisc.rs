// In your service-api route handler

use axum::{
    Extension, Json,
    extract::{Query,Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};   
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::json;

use crate::{
    state::AppState,
};
use utility_helpers::{log_error, log_info};

use auth_service::types::SessionTokenClaims;
use db_service::schema::users::{
   CreateDiscussionRequest, User };

pub async fn create_discussion_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(req): Json<CreateDiscussionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        user_id = %user_id,
        market_id = %req.market_id,
        "Creating new discussion"
    );
    
    match User::create(&state.pg_pool, req, user_id).await {
        Ok(discussion) => {
            log_info!(
                discussion_id = %discussion.id,
                user_id = %user_id,
                "Discussion created successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "discussion": discussion
            }));
             
            Ok((StatusCode::CREATED, response.into_response()))
        }
        Err(e) => {
            log_error!(
                user_id = %user_id,
                error = %e,
                "Failed to create discussion"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to create discussion",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}

pub async fn get_market_discussions_handler(
    State(state): State<AppState>,
    Path(market_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    log_info!(
        market_id = %market_id,
        "Fetching discussions for market"
    );
    
    match User::get_all_for_market(&state.pg_pool, market_id).await {
        Ok(discussions) => {
            log_info!(
                market_id = %market_id,
                count = discussions.len(),
                "Discussions fetched successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "discussions": discussions
            }));
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                market_id = %market_id,
                error = %e,
                "Failed to fetch discussions"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to fetch discussions",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}


pub async fn upvote_discussion_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(discussion_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        discussion_id = %discussion_id,
        user_id = %user_id,
        "Upvoting discussion"
    );
    
    match User::upvote(&state.pg_pool, discussion_id, user_id).await {
        Ok(_) => {
            log_info!(
                discussion_id = %discussion_id,
                user_id = %user_id,
                "Discussion upvoted successfully"
            );
            
            let response = Json(json!({
                "success": true,
                "message": "Discussion upvoted successfully"
            }));
            
            Ok((StatusCode::OK, response.into_response()))
        }
        Err(e) => {
            log_error!(
                discussion_id = %discussion_id,
                user_id = %user_id,
                error = %e,
                "Failed to upvote discussion"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to upvote discussion",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}



pub async fn remove_upvote_discussion_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(discussion_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    
    log_info!(
        discussion_id = %discussion_id,
        user_id = %user_id,
        "Removing upvote from discussion"
    );
    
    match User::remove_upvote(&state.pg_pool, discussion_id, user_id).await {
        Ok(_) => {
            log_info!(
                discussion_id = %discussion_id,
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
                discussion_id = %discussion_id,
                user_id = %user_id,
                error = %e,
                "Failed to remove upvote"
            );
            
            let error_response = serde_json::json!({
                "error": "Failed to remove upvote",
                "details": e.to_string()
            }).to_string();
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response.into_response()))
        }
    }
}


