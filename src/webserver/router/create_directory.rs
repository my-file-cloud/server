use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use api::route::create_directory::CreateDirectoryErrorKind;
use crate::storage::Storage;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::authenticated_user::AuthenticatedUser;

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create-directory/{*path}", post(create_directory))
}

async fn create_directory(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser, Path(path): Path<String>) -> Result<impl IntoResponse, ApiError<CreateDirectoryErrorKind>> {
    Storage::validate_path(&path)
        .map_err(|reason| -> ApiError<CreateDirectoryErrorKind> {
            (CreateDirectoryErrorKind::BadRequest, ApiErrorMessage::InvalidStoragePath(reason)).into()
        })?;
    
    match state.storage.with_user(authenticated_user.user_id).create_directory(&PathBuf::from(path)) {
        Ok(()) => Ok((StatusCode::OK, "")),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Err((CreateDirectoryErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
            _ => Err((CreateDirectoryErrorKind::BadRequest, ApiErrorMessage::from(err)).into())
        },
    }
}
