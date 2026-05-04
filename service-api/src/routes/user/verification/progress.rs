// use axum::{
//     Extension, Json,
//     extract::State,
//     http::StatusCode,
//     response::{IntoResponse, Response},
// };
// use serde::Serialize;

// use crate::{
//     state::AppState,

// };
// use auth_service::types::SessionTokenClaims;
// use db_service::schema::verification::{VerificationService, VerificationError};
// use utility_helpers::{log_error, log_info};

// #[derive(Debug, Serialize)]
// pub struct VerificationProgressResponse {
//     pub current_step: String,
//     pub steps_completed: Vec<String>,
//     pub documents_status: Vec<DocumentStatus>,
//     pub missing_requirements: Vec<String>,
// }

// #[derive(Debug, Serialize)]
// pub struct DocumentStatus {
//     pub document_type: String,
//     pub status: String,
//     pub uploaded_at: Option<String>,
//     pub rejection_reason: Option<String>,
// }

// pub async fn get_verification_progress(
//     State(state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     let user_id = claims.user_id;
    
//     log_info!(
//         "get_verification_progress",
//         "User fetching verification progress",
//         &[("user_id", &user_id.to_string())]
//     );
    
//     match VerificationService::get_verification_progress(
//         &state.pg_pool,
//         user_id,
//     )
//     .await
//     {
//         Ok(progress) => {
//             log_info!(
//                 "get_verification_progress",
//                 "Progress fetched successfully",
//                 &[("user_id", &user_id.to_string())]
//             );
            
//             Ok((StatusCode::OK, Json(progress).into_response()))
//         }
//         Err(e) => {
//             log_error!(
//                 "get_verification_progress",
//                 &format!("Failed to fetch progress: {}", e),
//                 &[("user_id", &user_id.to_string())]
//             );
            
//             let (status_code, error_message) = match e {
//                 VerificationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
//                 VerificationError::Database(db_err) => {
//                     (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", db_err))
//                 }
//                 _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
//             };
            
//             let error_response = serde_json::json!({
//                 "error": error_message
//             }).to_string();
            
//             Err((status_code, error_response.into_response()))
//         }
//     }
// }