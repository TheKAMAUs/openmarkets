use axum::{
    Router,
    routing::{get, post, delete},
};

use crate::state::AppState;

mod apply;
mod documents;
mod document_delete;
mod submit;
mod progress;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/apply", post(apply::apply_for_verification))
        // .route("/documents", get(documents::get_user_documents))
        // .route("/documents", post(documents::upload_document))
        // .route("/documents/{document_id}", delete(document_delete::delete_document))
        .route("/submit", post(submit::submit_verification))
        // .route("/progress", get(progress::get_verification_progress))
}