use my_file_cloud_api::route::download::DownloadErrorKind;
use futures::StreamExt;
use std::path::PathBuf;
use crate::webserver::app_state::AppState;
use crate::webserver::router::auth::authenticated_user::AuthenticatedUser;
use axum::{
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue};
use tracing::error;
use crate::webserver::router::{ApiError, ApiErrorMessage};

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/download", get (download))
        .route("/download/{*path}", get (download))
}

async fn download(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser, Path(path): Path<String>) -> Result<impl IntoResponse, ApiError<DownloadErrorKind>> {
    let path = PathBuf::from(path);
    let out_filename = match path.file_name() {
        Some(val) => val.to_str().unwrap_or("download"),
        None => return Err((DownloadErrorKind::BadRequest, ApiErrorMessage::from("No file name provided")).into()),
    };
    
    let user_storage = state.storage.with_user(authenticated_user.user_id);
    
    let metadata = match user_storage.path_metadata(&path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return match err.kind() {
                std::io::ErrorKind::NotFound => Err((DownloadErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
                _ => {
                    error!("Failed to get metadata: {}", err);
                    Err((DownloadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())   
                },
            }
        }
    };
    
    if metadata.is_file() {
        let stream = match user_storage.stream_file(&path).await {
            Ok(stream) => stream.map(|a| a.map(|bytes| bytes)),
            Err(err) => {
                return match err.kind() {
                    std::io::ErrorKind::NotFound => Err((DownloadErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
                    _ => {
                        error!("Failed to stream file: {}", err);
                        Err((DownloadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
                    },
                }
            }
        };

        let body = Body::from_stream(stream);

        Ok((
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", out_filename).as_str()),
            ],
            body,
        ).into_response())
    } else if metadata.is_dir() {
        let stream = match user_storage.stream_directory_zip(&path).await {
            Ok(stream) => stream,
            Err(err) => {
                return match err.kind() {
                    std::io::ErrorKind::NotFound => Err((DownloadErrorKind::NotFound, ApiErrorMessage::FileNotFound).into()),
                    _ => {
                        error!("Failed to stream file: {}", err);
                        Err((DownloadErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into())
                    },
                }
            }
        };
        
        let body = Body::from_stream(stream);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/zip"),
        );
        
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", out_filename)).expect("Failed to create HeaderValue"),
        );
        
        Ok((headers, body).into_response())
    } else {
        Err((DownloadErrorKind::NotFound, ApiErrorMessage::FileNotFound).into())
    }
}