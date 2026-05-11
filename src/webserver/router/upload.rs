use std::sync::Arc;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Router};
use axum::routing::{post};
use chrono::Utc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::error;
use api::route::upload::UploadErrorKind;
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiErrorMessage};
use crate::webserver::router::auth::authenticated_user::AuthenticatedUser;

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload", post  (upload))
        .route("/upload/{*path}", post  (upload))
        .layer(DefaultBodyLimit::disable())
}

async fn upload(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser, path: Option<Path<String>>, mut multipart: Multipart) -> Result<impl IntoResponse, ApiError<UploadErrorKind>> {
    let path = match path {
        Some(path) => path.0,
        None => String::new(),
    };
    
    let directory_path = state.storage.root_directory_path
        .join(authenticated_user.user_id.value())
        .join(&path);
    
    if !directory_path.exists() {
        std::fs::create_dir_all(&directory_path)
        .map_err(|err| -> ApiError<UploadErrorKind> {
            error!("Failed to create directory at {}: {err}", directory_path.to_string_lossy());
            (UploadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
        })?;
    }
    while let Some(mut field) = multipart.next_field().await
        .map_err(|err| -> ApiError<UploadErrorKind> {
            (UploadErrorKind::BadRequest, ApiErrorMessage::from(format!("Failed to read multipart field: {}", err))).into()
        })? 
    {
        let now = Utc::now().to_string();
        
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => format!("upload-{now}"),
        };
        
        let file_path = directory_path.join(file_name);

        let mut file = File::create(&file_path).await
            .map_err(|err| -> ApiError<UploadErrorKind> { (UploadErrorKind::InternalServerError, ApiErrorMessage::from(err)).into() })?;

        loop {
            match field.chunk().await {
                Ok(None) => break,
                Ok(Some(bytes)) => {
                    if let Err(err) = file.write_all(&bytes).await {
                        error!("Failed to write file chunk: {err}");
                        return Err((UploadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
                    }
                }
                Err(err) => {
                    error!("Failed to write file chunk: {err}");
                    return Err((UploadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into());
                }
            }
        }
    }

    Ok((StatusCode::OK, ""))
}
