use axum::{
    Router,
    routing::{get, post,patch, delete},
};

use crate::state::AppState;

mod getandcreateDisc;
mod suggestions;


pub fn router() -> Router<AppState> {
 Router::new()
    // Discussions routes
    .route("/discussions", post(getandcreateDisc::create_discussion_handler))
    // ✅ Change :market_id to {market_id}
    .route("/discussions/market/{market_id}", get(getandcreateDisc::get_market_discussions_handler))
    
    // ✅ Change :discussion_id to {discussion_id}
    .route("/discussions/{discussion_id}/upvote", post(getandcreateDisc::upvote_discussion_handler))
    .route("/discussions/{discussion_id}/upvote", delete(getandcreateDisc::remove_upvote_discussion_handler))
    
    // Suggestions routes
    .route("/suggestions", post(suggestions::create_suggestion_handler))
    .route("/suggestions", get(suggestions::get_suggestions_handler))
    // .route("/suggestions/public", get(suggestions::get_public_suggestions_handler))
    
    // ✅ Change :suggestion_id to {suggestion_id}
    .route("/suggestions/{suggestion_id}/upvote", post(suggestions::upvote_suggestion_handler))
    .route("/suggestions/{suggestion_id}/upvote", delete(suggestions::remove_suggestion_upvote_handler))
    
    // ✅ Change :suggestion_id to {suggestion_id}
    // .route("/admin/suggestions/{suggestion_id}/status", patch(suggestions::update_suggestion_status_handler))
}