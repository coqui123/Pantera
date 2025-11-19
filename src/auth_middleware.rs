use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use crate::handlers::AppState;
use crate::auth_handler::verify_session_cookie;
use crate::auth::AdminAuth;

/// Middleware to check if Tezos auth is enabled and user is authenticated
pub async fn require_auth_middleware(
    State(app_state): State<AppState>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    // If auth is not enabled, allow access
    if !app_state.config.auth.enable_tezos_auth {
        return next.run(request).await;
    }

    // Check for valid session cookie
    if let Some(cookie) = jar.get("tezos_admin_session") {
        if let Some(session) = verify_session_cookie(
            cookie.value(),
            &app_state.config.auth.cookie_hmac_key,
        ) {
            // Verify the address is still in admin list
            if app_state.config.auth.admin_tezos_addresses.contains(&session.address) {
                // Valid session, allow access
                return next.run(request).await;
            }
        }
    }

    // Check if dev mode is enabled (bypass auth)
    if app_state.config.auth.dev_mode {
        return next.run(request).await;
    }

    // Not authenticated, redirect to login
    Redirect::to("/login").into_response()
}

/// Extract AdminAuth from request (for use in handlers)
pub fn extract_admin_auth(
    app_state: &AppState,
    jar: &CookieJar,
) -> AdminAuth {
    // If auth is not enabled, return public auth
    if !app_state.config.auth.enable_tezos_auth {
        return AdminAuth::public();
    }

    // Check for valid session cookie
    if let Some(cookie) = jar.get("tezos_admin_session") {
        if let Some(session) = verify_session_cookie(
            cookie.value(),
            &app_state.config.auth.cookie_hmac_key,
        ) {
            // Verify the address is still in admin list
            if app_state.config.auth.admin_tezos_addresses.contains(&session.address) {
                return AdminAuth {
                    is_dev_admin: false,
                    tezos_admin_address: Some(session.address),
                };
            }
        }
    }

    // Check if dev mode is enabled
    if app_state.config.auth.dev_mode {
        return AdminAuth {
            is_dev_admin: true,
            tezos_admin_address: None,
        };
    }

    // Not authenticated
    AdminAuth::public()
}

