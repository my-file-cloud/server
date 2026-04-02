use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use tower_cookies::Cookies;
use tracing::error;
use my_file_cloud_api::route::auth::login::{LoginBody, LoginErrorKind};
use crate::model::session::Session;
use crate::util;
use crate::webserver::app_state::AppState;
use crate::webserver::jwt;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::build_token_cookies;

/// Route for a user to log in
/// - looks up username - password connection
/// - when user exists, create access_ & refresh_token 
/// - store hashed refresh_token
/// - set client httponly cookie "refresh_token" to refresh_token
/// - set client httponly cookie "access_token" to access_token
pub async fn login(State(state): State<Arc<AppState>>, cookies: Cookies, Json(body): Json<LoginBody>) -> Result<impl IntoResponse, ApiError<LoginErrorKind>> {
    let user = match state.database.find_user_by_name(&body.username).await
        .map_err(|err| -> ApiError<LoginErrorKind> {
            error!("Failed to find user by name: {err}");
            (LoginErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into() 
        })? {
        Some(user) => user,
        None => return Err((LoginErrorKind::Unauthorized, ApiErrorMessage::from("Invalid username or password")).into()),
    };

    if let Err(_) = util::verify_hash(&body.password, &user.password) {
        return Err((LoginErrorKind::Unauthorized, ApiErrorMessage::from("Invalid username or password")).into());
    }

    let session_id = Session::new_id();
    let (access_token, refresh_token) = match jwt::default_token_pair(user.id.clone(), session_id.clone(), &state.jwt_secret.clone()) {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to create token pair: {err}");
            return Err((LoginErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
        }
    };

    let refresh_token_hash = match util::hash(&refresh_token) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to create refresh token hash: {err}");
            return Err((LoginErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
        },
    };
    
    match state.database.create_session(Session {
        id: session_id,
        user_id: user.id,
        refresh_token: refresh_token_hash
    }).await {
        Ok(_) => (),
        Err(err) => { 
            error!("Failed to create session: {err}");
            return Err((LoginErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
        },
    }

    let (access_token_cookie, refresh_token_cookie) = build_token_cookies(access_token, refresh_token);

    cookies.add(access_token_cookie);
    cookies.add(refresh_token_cookie);

    Ok((StatusCode::OK, ""))
}