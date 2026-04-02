use std::sync::Arc;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use serde::Deserialize;
use tower_cookies::Cookies;
use tracing::error;
use my_file_cloud_api::id::ID;
use my_file_cloud_api::route::auth::AuthenticationError;
use crate::model::user::User;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::{process_token_cookies, TokenCookieErrorKind};

/// Contains the user ID and token claims of a user.
///
/// # Authentication Middleware
/// Implements FromRequestParts that handles the bearer token authentication.
#[derive(Clone, Debug, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: ID<User>,
}
impl AuthenticatedUser {
    fn new(user_id: ID<User>) -> Self {
        Self { user_id }
    }
}
impl<S> FromRequestParts<S> for AuthenticatedUser
where S: Send + Sync, Arc<AppState>: FromRef<S>
{
    type Rejection = ApiError<AuthenticationError>;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, &state)
            .await
            .map_err(|(_status_code, msg)| {
                let err: ApiError<AuthenticationError> = (AuthenticationError::BadRequest, ApiErrorMessage::from(msg)).into();
                err
            })?;

        let State(app_state): State<Arc<AppState>> = State::from_request_parts(parts, state)
            .await.map_err(|err| {
                error!("Failed to create State from request parts: {err}");
                let err: ApiError<AuthenticationError> = (AuthenticationError::InternalServerError, ApiErrorMessage::from(err)).into();
                err
            }
        )?;

        let (access_token_data, _) = process_token_cookies(&cookies, &app_state.jwt_secret);
        let access_token_data = match access_token_data {
            Err(err) => {
                return match err.kind {
                    TokenCookieErrorKind::InvalidInputSecret => {
                        error!("JWT Secret is of invalid format");
                        Err((AuthenticationError::InternalServerError, ApiErrorMessage::InternalServerError).into())
                    },
                    TokenCookieErrorKind::ValidationError => {
                        Err((AuthenticationError::Unauthorized, ApiErrorMessage::Custom(err.message)).into())
                    },
                    TokenCookieErrorKind::InvalidToken => {
                        Err((AuthenticationError::BadRequest, ApiErrorMessage::Custom(err.message)).into())
                    }
                }
            },
            Ok(res) => res,
        };
        
        let (access_token_claims, _) = match access_token_data {
            Some(token_data) => token_data,
            None => return Err((AuthenticationError::BadRequest, ApiErrorMessage::from("Missing access token")).into()),
        };

        Ok(AuthenticatedUser::new(access_token_claims.user_id))
    }
}