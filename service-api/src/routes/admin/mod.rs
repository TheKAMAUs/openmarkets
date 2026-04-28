use axum::Router;

use crate::state::AppState;

pub mod markets;
pub mod verifications; 
pub mod messages;
pub fn router() -> Router<AppState> {
    Router::new().nest("/market", markets::market_router())

       .nest("/verifications", verifications::verification_router())
.nest("/messages", messages::messages_router())


}
