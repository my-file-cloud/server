use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get};
use my_file_cloud_api::route::browse::BrowseErrorKind;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::authenticated_user::AuthenticatedUser;

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/browse", get(browse))
        .route("/browse/{*path}", get(browse))
        .route("/browse/{*path}", delete(delete_path))
}

pub async fn browse(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser, path: Option<Path<String>>) -> Result<impl IntoResponse, ApiError<BrowseErrorKind>> {
    let path = match path {
        Some(Path(path)) => PathBuf::from(path),
        None => PathBuf::new(),
    };
    
    match state.storage.with_user(authenticated_user.user_id).browse_path(&path) {
        Ok(res) => Ok((StatusCode::OK, Json(res))),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Err((BrowseErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
            _ => Err((BrowseErrorKind::BadRequest, ApiErrorMessage::from(err)).into()),
        },
    }
}

pub enum DeletePathErrorKind {
    NotFound,
    BadRequest,
}
impl Into<StatusCode> for DeletePathErrorKind {
    fn into(self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
        }
    }
}

pub async fn delete_path(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser, path: Path<String>) -> Result<impl IntoResponse, ApiError<DeletePathErrorKind>> {
    let path = PathBuf::from(path.0);
    
    match state.storage.with_user(authenticated_user.user_id).delete_path(&path) {
        Ok(res) => Ok((StatusCode::OK, Json(res))),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Err((DeletePathErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
            _ => Err((DeletePathErrorKind::BadRequest, ApiErrorMessage::from(err)).into()),
        },
    }
}
