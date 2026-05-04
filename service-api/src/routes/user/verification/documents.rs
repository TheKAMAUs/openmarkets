// use axum::{
//     Extension, Json,
//     extract::State,
//     http::StatusCode,
//     response::{IntoResponse, Response},
// };
// use serde::{Deserialize, Serialize};
// use uuid::Uuid;

// use crate::{
//     state::AppState,
  
// };
// use auth_service::types::SessionTokenClaims;
// use db_service::schema::verification::{VerificationService, VerificationError};
// use utility_helpers::{log_error, log_info};

// #[derive(Debug, Serialize)]
// pub struct DocumentResponse {
//     pub id: Uuid,
//     pub document_type: String,
//     pub status: String,
//     pub uploaded_at: String,
//     pub document_url: String,
// }

// #[derive(Debug, Serialize)]
// pub struct DocumentStatus {
//     pub document_type: String,
//     pub status: String,
//     pub uploaded_at: Option<String>,
//     pub rejection_reason: Option<String>,
// }

// pub async fn get_user_documents(
//     State(state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     let user_id = claims.user_id;
    
//     log_info!(
//         "get_user_documents",
//         "User fetching documents",
//         &[("user_id", &user_id.to_string())]
//     );
    
//     match VerificationService::get_user_documents(
//         &state.pg_pool,
//         user_id,
//     )
//     .await
//     {
//         Ok(documents) => {
//             log_info!(
//                 "get_user_documents",
//                 &format!("Found {} documents", documents.len()),
//                 &[("user_id", &user_id.to_string())]
//             );
            
//             Ok((StatusCode::OK, Json(documents).into_response()))
//         }
//         Err(e) => {
//             log_error!(
//                 "get_user_documents",
//                 &format!("Failed to fetch documents: {}", e),
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