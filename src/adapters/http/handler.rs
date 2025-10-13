use crate::application::errors::AppError;
use crate::application::types::{AuthCode, PKCEVerifier, SessionId, SessionUser};
use crate::AppState;
use axum::extract::{Multipart, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use axum_extra::extract::PrivateCookieJar;
use std::time::Duration;

use crate::adapters::http::cookie_util;
use crate::adapters::http::request_types::AuthRequest;
use cookie::time::Duration as CookieDuration;

// TODO split handlers

pub(crate) async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
pub(crate) async fn images_handler(State(state): State<AppState>, profile: SessionUser) -> Result<impl IntoResponse, AppError> {
    let result = state.image_service.find_all_images(&profile).await?;
    Ok((StatusCode::OK, Json(serde_json::json!(result))).into_response())
}

pub(crate) async fn upload_handler(
    State(state): State<AppState>,
    profile: SessionUser,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.to_lowercase().starts_with("multipart/"))
        .unwrap_or(false)
    {
        return Err(AppError::InvalidContentType);
    }

    match state.receipt_scanner.try_process_receipt(multipart, &profile.user_id).await? {
        Some(json) => Ok((StatusCode::OK, Json(serde_json::json!({"result": json}))).into_response()),
        None => Ok((StatusCode::NO_CONTENT, ()).into_response()),
    }
}

pub(crate) async fn login_handler(State(state): State<AppState>, jar: PrivateCookieJar) -> impl IntoResponse {
    let (pkce_verifier, csrf_token, auth_url) = state.authentication.login().await;
    let cookie_ttl = CookieDuration::seconds(600);
    let pkce_cookie = cookie_util::make_cookie(state.config.clone(), "pkce_v", pkce_verifier.as_ref(), cookie_ttl, true);
    let state_cookie = cookie_util::make_cookie(state.config.clone(), "oauth_state", csrf_token.as_ref(), cookie_ttl, true);

    (jar.add(pkce_cookie).add(state_cookie), Redirect::temporary(auth_url.as_ref()))
}

pub(crate) async fn protected(profile: SessionUser) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!(profile)))
}

pub(crate) async fn me(State(state): State<AppState>, profile: SessionUser) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.find_user(&profile.user_id).await?;

    match user {
        Some(user) => Ok((StatusCode::OK, Json(user))),
        None => Err(AppError::Unauthorized),
    }
}

pub(crate) async fn logout_handler(State(state): State<AppState>, jar: PrivateCookieJar, _profile: SessionUser) -> impl IntoResponse {
    let sid = jar.get("sid").map(|c| c.value().to_string());
    if let Some(sid) = sid {
        state.authentication.logout(&SessionId(sid)).await;
    }
    let expired_pkce = cookie_util::make_cookie(state.config.clone(), "pkce_v", "", CookieDuration::seconds(0), true);
    let expired_state = cookie_util::make_cookie(state.config.clone(), "oauth_state", "", CookieDuration::seconds(0), true);
    let expired_sid = cookie_util::make_cookie(state.config.clone(), "sid", "", CookieDuration::seconds(0), true);

    (
        jar.add(expired_sid).add(expired_pkce).add(expired_state),
        Redirect::to(&state.config.login_redirect_url),
    )
}

pub(crate) async fn google_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let cookie_state = jar.get("oauth_state").map(|c| c.value().to_string()).ok_or(AppError::Unauthorized)?;
    if cookie_state != query.state {
        return Err(AppError::Unauthorized);
    }
    let pkce_verifier = jar.get("pkce_v").map(|c| c.value().to_string()).ok_or(AppError::Unauthorized)?;
    let ttl = Duration::from_secs(3600);
    let session_id = state
        .authentication
        .update_session(PKCEVerifier(pkce_verifier), AuthCode(query.code), ttl)
        .await?;
    let ttl_cookie = CookieDuration::seconds(ttl.as_secs() as i64);
    let cookie = cookie_util::make_cookie(state.config.clone(), "sid", session_id.as_ref(), ttl_cookie, true);

    let expired_pkce = cookie_util::make_cookie(state.config.clone(), "pkce_v", "", CookieDuration::seconds(0), true);
    let expired_state = cookie_util::make_cookie(state.config.clone(), "oauth_state", "", CookieDuration::seconds(0), true);
    Ok((
        jar.add(cookie).add(expired_pkce).add(expired_state),
        Redirect::to(&state.config.login_redirect_url),
    ))
}
