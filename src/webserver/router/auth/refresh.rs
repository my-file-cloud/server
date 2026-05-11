use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower_cookies::Cookies;
use tracing::error;
use api::route::auth::refresh::RefreshErrorKind;
use crate::util;
use crate::webserver::app_state::AppState;
use crate::webserver::jwt;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::{build_token_cookies, process_token_cookies, TokenCookieError, TokenCookieErrorKind};

/// Route to request a new token pair
/// - requires refresh_token cookie
/// - verifies refresh_token
/// - updates old saved hash of refresh_token to hash of new refresh_token 
pub async fn refresh(State(state): State<Arc<AppState>>, cookies: Cookies) -> Result<impl IntoResponse, ApiError<RefreshErrorKind>> {
    let (access_token_data, refresh_token_data) = process_token_cookies(&cookies, &state.jwt_secret);
    let refresh_token_data = match refresh_token_data {
        Err(TokenCookieError{ kind, message }) => {
            return match kind {
                TokenCookieErrorKind::InvalidInputSecret => {
                    error!("JWT Secret is of invalid format");
                    Err((RefreshErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
                },
                TokenCookieErrorKind::ValidationError => {
                    Err((RefreshErrorKind::Unauthorized, ApiErrorMessage::from(message)).into())
                },
                TokenCookieErrorKind::InvalidToken => {
                    Err((RefreshErrorKind::BadRequest, ApiErrorMessage::from(message)).into())
                }
            }
        },
        Ok(res) => res,
    };

    let (refresh_token_claims, mut refresh_token_cookie) = match refresh_token_data {
        Some(data) => data,
        None => return Err((RefreshErrorKind::BadRequest, ApiErrorMessage::from("No cookie 'refresh_token'")).into()),
    };

    let refresh_token: &str = refresh_token_cookie.value();

    let session = match state.database.get_session(&refresh_token_claims.session_id).await {
        Err(err) => {
            error!("Failed to get session: {err}");
            return Err((RefreshErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
        },
        Ok(res) => res,
    };

    // TODO: cookie.make_removal() somehow does not work?? so we need to use cookies.remove() instead
    match session {
        None => {
            refresh_token_cookie.make_removal();
            if let Ok(Some((_, mut cookie))) = access_token_data {
                cookie.make_removal();
            }
            Err((RefreshErrorKind::Unauthorized, ApiErrorMessage::from("Invalid session")).into())
        },
        Some(mut session) => {
            if let Err(_) = util::verify_hash(refresh_token, &session.refresh_token) {
                return Err((RefreshErrorKind::Unauthorized, ApiErrorMessage::from("Invalid refresh token")).into());
            }

            let (new_access_token, new_refresh_token) = jwt::default_token_pair(refresh_token_claims.user_id.clone(), refresh_token_claims.session_id, &state.jwt_secret)
                .map_err(|err| -> ApiError<RefreshErrorKind> {
                    error!("Failed to create token pair: {err}");
                    (RefreshErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
                })?;
            
            session.refresh_token = util::hash(&new_refresh_token)
                .map_err(|err| -> ApiError<RefreshErrorKind> {
                    error!("Failed to create refresh token hash: {err}");
                    (RefreshErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
                })?;
            
            state.database.update_session(session).await
                .map_err(|err| -> ApiError<RefreshErrorKind> {
                    error!("Could not update session: {err}");
                    (RefreshErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
                })?;
            
            let (access_token_cookie, refresh_token_cookie) = build_token_cookies(new_access_token, new_refresh_token);
            
            cookies.add(access_token_cookie);
            cookies.add(refresh_token_cookie);

            Ok((StatusCode::OK, ""))
        }
    }
}
