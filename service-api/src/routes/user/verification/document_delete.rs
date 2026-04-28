// use axum::{
//     Extension, Json,
//     extract::{State, Path},
//     http::StatusCode,
//     response::{IntoResponse, Response},
// };
// use uuid::Uuid;

// use crate::{
//     state::AppState,

// };
// use auth_service::types::SessionTokenClaims;
// use db_service::schema::verification::VerificationService;
// use utility_helpers::{log_error, log_info};

// pub async fn delete_document(
//     State(state): State<AppState>,
//     Extension(claims): Extension<SessionTokenClaims>,
//     Path(document_id): Path<Uuid>,
// ) -> Result<impl IntoResponse, (StatusCode, Response)> {
//     let user_id = claims.user_id;
    
//     log_info!(
//         "delete_document",
//         "User deleting document",
//         &[
//             ("user_id", &user_id.to_string()),
//             ("document_id", &document_id.to_string())
//         ]
//     );
    
//     match VerificationService::delete_document(
//         &state.pg_pool,
//         user_id,
//         document_id,
//     )
//     .await
//     {
//         Ok(_) => {
//             log_info!(
//                 "delete_document",
//                 "Document deleted successfully",
//                 &[("user_id", &user_id.to_string())]
//             );
            
//             Ok((StatusCode::NO_CONTENT, ().into_response()))
//         }
//         Err(e) => {
//             log_error!(
//                 "delete_document",
//                 &format!("Failed to delete document: {}", e),
//                 &[("user_id", &user_id.to_string())]
//             );
            
//             let (status_code, error_message) = match e {
//                 VerificationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
//                 VerificationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
//                 VerificationError::Database(db_err) => {
//                     (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", db_err))
//                 }
//             };
            
//             let error_response = serde_json::json!({
//                 "error": error_message
//             }).to_string();
            
//             Err((status_code, error_response.into_response()))
//         }
//     }
// }