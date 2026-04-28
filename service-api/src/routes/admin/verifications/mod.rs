
// routes/admin/verifications/mod.rs
use crate::state::AppState;
use crate::routes::admin::verifications::admin_actions::{

    approve_user_handler,
    reject_user_handler,
    request_revision_handler,
    suspend_user_handler,
    reinstate_user_handler,
};
use axum::{
    Router,
    routing::{get, post},
};
 use crate::routes::admin::verifications::pending::get_pending_verifications_handler;


pub mod pending;
pub mod admin_actions;

pub fn verification_router() -> Router<AppState> {
    Router::new()
        // GET routes
        .route("/pending", get(get_pending_verifications_handler))
        
       // POST routes for admin actions - use {capture} not :capture
        .route("/{user_id}/approve", post(approve_user_handler))
        .route("/{user_id}/reject", post(reject_user_handler))
        .route("/{user_id}/request-revision", post(request_revision_handler))
        .route("/{user_id}/suspend", post(suspend_user_handler))
        .route("/{user_id}/reinstate", post(reinstate_user_handler))
}