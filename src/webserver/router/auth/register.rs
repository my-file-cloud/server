use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use tracing::error;
use my_file_cloud_api::route::auth::register::{RegisterBody, RegisterErrorKind};
use crate::database::DatabaseError;
use crate::model::user::User;
use crate::util;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};

/// Route to register a new user
///
/// # Parameter
/// username has to be unique
pub async fn register(State(state): State<Arc<AppState>>, Json(body): Json<RegisterBody>) -> Result<impl IntoResponse, ApiError<RegisterErrorKind>> {
    let state = state.clone();

    if state.database.find_user_by_name(&body.username).await
        .map_err(|err| -> ApiError<RegisterErrorKind> {
            error!("Failed to find user by name: {err}");
            (RegisterErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
        })?.is_some() {
        return Err((RegisterErrorKind::Conflict, ApiErrorMessage::from("Username already exists")).into());
    }

    let user_password = match util::hash(&body.password) {
        Ok(password) => password,
        Err(err) => {
            error!("Failed to create password hash: {err}");
            return Err((RegisterErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
        }
    };
    
    let user = User::new(body.username.clone(), user_password);
    
    if let Err(err) = state.database.create_user(user.clone()).await {
        return Err(match err {
            DatabaseError::UniqueViolation(_) => (RegisterErrorKind::Conflict, ApiErrorMessage::from("Username already exists")).into(),
            _ => {
                error!("Failed to register user: {err}");
                (RegisterErrorKind::InternalServerError, ApiErrorMessage::from(err)).into()
            }
        })
    }

    match state.storage.create_user(user.id.clone()) {
        Ok(_) => (),
        Err(err) => return Err((RegisterErrorKind::BadRequest, ApiErrorMessage::from(err)).into()),
    };
    
    Ok((StatusCode::OK, ""))
}