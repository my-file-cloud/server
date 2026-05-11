use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower_cookies::Cookies;
use tracing::error;
use api::route::auth::logout::LogoutErrorKind;
use crate::util;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::{process_token_cookies, TokenCookieError, TokenCookieErrorKind};

/// Route for a user to log out
/// - requires refresh_token cookie
/// - verifies refresh_token
/// - removes refresh_token cookie
/// - removes access_token cookie
/// - removes refresh_token from stored tokens
pub async fn logout(State(state): State<Arc<AppState>>, cookies: Cookies) -> Result<impl IntoResponse, ApiError<LogoutErrorKind>> {
    let (access_token_data, refresh_token_data) = process_token_cookies(&cookies, &state.jwt_secret);
    let refresh_token_data = match refresh_token_data {
        Err(TokenCookieError { kind, message }) => {
            return match kind {
                TokenCookieErrorKind::InvalidInputSecret => {
                    error!("JWT Secret is of invalid format");
                    Err((LogoutErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
                },
                TokenCookieErrorKind::ValidationError => {
                    Err((LogoutErrorKind::Unauthorized, ApiErrorMessage::from(message)).into())
                },
                TokenCookieErrorKind::InvalidToken => {
                    Err((LogoutErrorKind::BadRequest, ApiErrorMessage::from(message)).into())
                }
            }
        },
        Ok(res) => res,
    };
    
    let (refresh_token_claims, mut refresh_token_cookie) = match refresh_token_data {
        Some(data) => data,
        None => return Err((LogoutErrorKind::BadRequest, ApiErrorMessage::from("Refresh token is required")).into()),
    };

    let session = match state.database.get_session(&refresh_token_claims.session_id).await {
        Ok(session) => session,
        Err(err) => {
            error!("Failed to get session: {err}");
            return Err((LogoutErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
        },
    };
    

    match session {
        None => {
            refresh_token_cookie.make_removal();
            if let Ok(Some((_, mut cookie))) = access_token_data {
                cookie.make_removal();
            }
            Err((LogoutErrorKind::Unauthorized, ApiErrorMessage::from("Invalid session")).into())
        },
        Some(session) => {
            if let Err(_) = util::verify_hash(refresh_token_cookie.value(), &session.refresh_token) {
                return Err((LogoutErrorKind::Unauthorized, ApiErrorMessage::from("Invalid refresh token")).into());
            }

            if let Err(err) = state.database.delete_session(&refresh_token_claims.session_id).await {
                error!("Failed to get session: {err}");
                return Err((LogoutErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
            };
            
            refresh_token_cookie.make_removal();
            if let Ok(Some((_, mut cookie))) = access_token_data {
                cookie.make_removal();
            }

            Ok((StatusCode::OK, ""))
        }
    }
}
