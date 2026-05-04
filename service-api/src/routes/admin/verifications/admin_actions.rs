// api/handlers/admin_verification_handlers.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};


use uuid::Uuid;
use auth_service::types::SessionTokenClaims;
use crate::state::AppState;

use db_service::schema::verification::{
    VerificationService, PendingVerificationUser};

use utility_helpers::{log_error, log_info};

// api/dtos/admin_verification_dtos.rs
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ApproveUserRequest {
    pub notes: Option<String>,
    pub document_id: Option<String>,  // Reference to the specific document being approved
}

#[derive(Debug, Deserialize)]
pub struct RejectUserRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestRevisionRequest {
    pub notes: String,
    pub rejected_document_types: Vec<String>,
}

// ✅ Add this AdminActionResponse struct
#[derive(Debug, Serialize)]
pub struct AdminActionResponse {
    pub message: String,
    pub user_id: Uuid,
    pub new_status: String,
    pub timestamp: String,
}

// You might also want these for other endpoints
#[derive(Debug, Serialize)]
pub struct GetPendingResponse {
    pub users: Vec<PendingVerificationUser>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Serialize)]
pub struct PaginationMetadata {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}





pub async fn approve_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ApproveUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
  match VerificationService::approve_user(
    &state.pg_pool,
    user_id,
    claims.user_id,
    req.notes,
    req.document_id,  // Added document_id parameter
)
.await
    {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "User approved successfully",
                "data": {
                    "user_id": user_id,
                    "new_status": "approved",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            })).into_response()
        )),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "user_id": user_id
            })).into_response()
        )),
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("pending state") => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Invalid user state",
                "details": msg.to_string(),
                "user_id": user_id
            })).into_response()
        )),
        Err(e) => {
            log_error!("Failed to approve user: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to approve user",
                    "message": "An internal server error occurred"
                })).into_response()
            ))
        }
    }
}

pub async fn reject_user_handler(
    State(state): State<AppState>,
     Extension(claims): Extension<SessionTokenClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<RejectUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    match  VerificationService::reject_user(
        &state.pg_pool,
        user_id,
        claims.user_id,
       req.reason.clone(),  // Clone here
    )
    .await
    {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "User rejected successfully",
                "data": {
                    "user_id": user_id,
                    "new_status": "rejected",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "reason": req.reason
                }
            })).into_response()
        )),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "user_id": user_id
            })).into_response()
        )),
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("pending state") => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Invalid user state",
                "details": msg.to_string(),
                "user_id": user_id
            })).into_response()
        )),
        Err(e) => {
            log_error!("Failed to reject user: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to reject user",
                    "message": "An internal server error occurred"
                })).into_response()
            ))
        }
    }
}

pub async fn request_revision_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<RequestRevisionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    // Log the incoming request
    log_info!(
        "Processing revision request for user: {}, admin: {}, document_types: {:?}",
        user_id,
        claims.user_id,
        req.rejected_document_types
    );
    
    match VerificationService::request_revision(
        &state.pg_pool,
        user_id,
        claims.user_id,
        req.notes.clone(),
        req.rejected_document_types.clone(),
    )
    .await
    {
        Ok(_) => {
            log_info!(
                "Successfully requested revision for user: {}, admin: {}, rejected_documents: {:?}",
                user_id,
                claims.user_id,
                req.rejected_document_types
            );
            
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Revision requested successfully",
                    "data": {
                        "user_id": user_id,
                        "new_status": "pending",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "rejected_documents": req.rejected_document_types,
                        "notes": req.notes
                    }
                })).into_response()
            ))
        },
        Err(sqlx::Error::RowNotFound) => {
            log_error!(
                "User not found for revision request - user_id: {}, admin_id: {}",
                user_id,
                claims.user_id
            );
            
            Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "User not found",
                    "user_id": user_id
                })).into_response()
            ))
        },
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("pending state") => {
            log_error!(
                "Invalid user state for revision request - user_id: {}, admin_id: {}, state_error: {}",
                user_id,
                claims.user_id,
                msg
            );
            
            Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Invalid user state",
                    "details": msg.to_string(),
                    "user_id": user_id
                })).into_response()
            ))
        },
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("valid document") => {
            log_error!(
                "Invalid document types in revision request - user_id: {}, admin_id: {}, invalid_types: {:?}, error: {}",
                user_id,
                claims.user_id,
                req.rejected_document_types,
                msg
            );
            
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid document types",
                    "details": msg.to_string(),
                    "user_id": user_id
                })).into_response()
            ))
        },
        Err(e) => {
            log_error!(
                "Failed to request revision - user_id: {}, admin_id: {}, error: {:?}",
                user_id,
                claims.user_id,
                e
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to request revision",
                    "message": "An internal server error occurred"
                })).into_response()
            ))
        }
    }
}



pub async fn suspend_user_handler(
    State(state): State<AppState>,
       Extension(claims): Extension<SessionTokenClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<RejectUserRequest>, // Reuse the same DTO for suspend reason
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    match  VerificationService::suspend_user(
        &state.pg_pool,
        user_id,
        claims.user_id,
        req.reason.clone(),
    )
    .await
    {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "User suspended successfully",
                "data": {
                    "user_id": user_id,
                    "new_status": "suspended",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "reason": req.reason
                }
            })).into_response()
        )),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "user_id": user_id
            })).into_response()
        )),
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("approved users") => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Invalid user state",
                "details": msg.to_string(),
                "user_id": user_id
            })).into_response()
        )),
        Err(e) => {
            log_error!("Failed to suspend user: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to suspend user",
                    "message": "An internal server error occurred"
                })).into_response()
            ))
        }
    }
}

pub async fn reinstate_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ApproveUserRequest>, // Reuse approve DTO for notes
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    match  VerificationService::reinstate_user(
        &state.pg_pool,
        user_id,
        claims.user_id,
        req.notes.clone(),
    )
    .await
    {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "User reinstated successfully",
                "data": {
                    "user_id": user_id,
                    "new_status": "approved",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "notes": req.notes
                }
            })).into_response()
        )),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "user_id": user_id
            })).into_response()
        )),
        Err(sqlx::Error::Protocol(msg)) if msg.to_string().contains("not suspended") => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Invalid user state",
                "details": msg.to_string(),
                "user_id": user_id
            })).into_response()
        )),
        Err(e) => {
            log_error!("Failed to reinstate user: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to reinstate user",
                    "message": "An internal server error occurred"
                })).into_response()
            ))
        }
    }
}