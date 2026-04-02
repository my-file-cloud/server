use std::sync::Arc;
use axum::Router;
use axum::routing::post;
use tower_cookies::{Cookie, Cookies};
use tower_cookies::cookie::SameSite;
use tracing::info;
use crate::webserver::app_state::AppState;
use crate::webserver::jwt;
use crate::webserver::jwt::{AccessTokenClaims, RefreshTokenClaims};
use crate::webserver::router::auth::login::login;
use crate::webserver::router::auth::logout::logout;
use crate::webserver::router::auth::refresh::refresh;
use crate::webserver::router::auth::register::register;

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post    (register   ))
        .route("/auth/login",    post    (login      ))
        .route("/auth/logout",   post    (logout     ))
        .route("/auth/refresh",  post    (refresh    ))
}

pub mod authenticated_user;
pub mod register;
pub mod logout;
pub mod refresh;
pub mod login;

pub fn build_token_cookies(access_token: String, refresh_token: String) -> (Cookie<'static>, Cookie<'static>) {
    let access_token_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(false)
        .build();
    
    let refresh_token_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/auth")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(false)
        .build();

    (access_token_cookie, refresh_token_cookie)
}

#[derive(Debug)]
pub enum TokenCookieErrorKind {
    /// Token invalid
    ValidationError,
    /// Provided secret was invalid
    InvalidInputSecret,
    /// Token with invalid format
    InvalidToken,
}
impl From<&jsonwebtoken::errors::ErrorKind> for TokenCookieErrorKind {
    fn from(value: &jsonwebtoken::errors::ErrorKind) -> Self {
        match value {
            jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_)
            | jsonwebtoken::errors::ErrorKind::InvalidClaimFormat(_)
            | jsonwebtoken::errors::ErrorKind::ExpiredSignature
            | jsonwebtoken::errors::ErrorKind::InvalidIssuer
            | jsonwebtoken::errors::ErrorKind::InvalidAudience
            | jsonwebtoken::errors::ErrorKind::InvalidSubject
            | jsonwebtoken::errors::ErrorKind::ImmatureSignature
            | jsonwebtoken::errors::ErrorKind::InvalidAlgorithm
            | jsonwebtoken::errors::ErrorKind::MissingAlgorithm => Self::ValidationError,
            jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => Self::InvalidInputSecret,
            _ => Self::InvalidToken,
        }
    }
}

pub struct TokenCookieError {
    pub kind: TokenCookieErrorKind,
    pub message: String,
}
impl<T: ToString> From<(TokenCookieErrorKind, T)> for TokenCookieError {
    fn from((kind, msg): (TokenCookieErrorKind, T)) -> Self {
        Self {
            kind,
            message: msg.to_string(),
        }
    }
}

type AccessTokenResult<'a> = Result<Option<(AccessTokenClaims, Cookie<'a>)>, TokenCookieError>;
type RefreshTokenResult<'a> = Result<Option<(RefreshTokenClaims, Cookie<'a>)>, TokenCookieError>;

/// helper function to get and validate the tokens from a cookie list
/// removes the cookies when an error occurs
pub fn process_token_cookies<'a>(cookies: &'a Cookies, secret: &Vec<u8>) -> (AccessTokenResult<'a>, RefreshTokenResult<'a>) {
    let access_token_data = {
        let cookie = cookies.get("access_token");
        if let Some(mut cookie) = cookie {
            match jwt::decode_token(&cookie.value(), &secret) {
                Ok(token_data) => Ok(Some((token_data.claims, cookie))),
                Err(err) => {
                    cookie.make_removal();
                    
                    info!("error when decoding token: {err}");
                    
                    Err((TokenCookieErrorKind::from(err.kind()), err).into())
                }
            }
        } else {
            Ok(None)
        }
    };

    let refresh_token_data = {
        let cookie = cookies.get("refresh_token");
        if let Some(mut cookie) = cookie {
            match jwt::decode_token(&cookie.value(), &secret) {
                Ok(token_data) => Ok(Some((token_data.claims, cookie))),
                Err(err) => {
                    cookie.make_removal();

                    Err((TokenCookieErrorKind::from(err.kind()), err.to_string()).into())
                }
            }
        } else {
            Ok(None)
        }
    };
    
    (access_token_data, refresh_token_data)
}