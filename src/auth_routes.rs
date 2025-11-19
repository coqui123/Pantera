use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::AppState;
use crate::auth_handler;

pub fn create_auth_router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/auth/tezos/challenge", get(auth_handler::get_tezos_challenge))
        .route("/auth/tezos/login", post(auth_handler::tezos_login))
        .route("/auth/logout", post(auth_handler::logout))
        .route("/auth/status", get(auth_handler::auth_status));
    
    // Only add debug endpoint in debug builds
    #[cfg(debug_assertions)]
    {
        router = router.route("/chumfunkular/auth/debug", get(auth_handler::debug_auth_status));
    }
    
    router
} 