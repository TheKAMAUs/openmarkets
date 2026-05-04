use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db_service::schema::verification::{
    VerificationService, };
use crate::{
    state::AppState,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_pending_verifications_handler(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result =  VerificationService::get_pending_verifications(
        &state.pg_pool,
        params.limit,
        params.offset,
    )
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok((StatusCode::OK, Json(result)))
}