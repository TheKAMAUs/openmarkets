use axum::{
    Router,
    routing::{get, post,patch, delete},
};


use crate::state::AppState;

pub mod suggestions;


pub fn messages_router() -> Router<AppState> {
    Router::new()
         .route("/suggestions", get(suggestions::get_suggestions_handler))
       .route("/suggestions/{suggestion_id}/upvote", post(suggestions::upvote_suggestion_handler))
    .route("/suggestions/{suggestion_id}/upvote", delete(suggestions::remove_suggestion_upvote_handler))
// ✅ Change :suggestion_id to {suggestion_id}
    .route("/suggestions/{suggestion_id}/status", patch(suggestions::update_suggestion_status_handler))

}





