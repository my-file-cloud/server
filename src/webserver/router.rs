use std::sync::Arc;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::get;
use serde::Serialize;
use tower_cookies::CookieManagerLayer;
use crate::webserver::app_state::AppState;
use tower_http::cors;
use my_file_cloud_api::{ApiErrorBody, ApiErrorMessage};

mod auth;
mod dashboard;
mod browse;
mod upload;
mod create_directory;
mod download;

pub struct ApiResponse<T: Serialize> {
    status: StatusCode,
    body: T,
}
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
impl<T: Serialize> From<(StatusCode, T)> for ApiResponse<T> {
    fn from((status, body): (StatusCode, T)) -> Self {
        Self { status, body }
    }
}
impl<T: Serialize> From<T> for ApiResponse<T> {
    fn from(body: T) -> Self {
        (StatusCode::OK, body).into()
    }
}

pub struct ApiError<T: Into<StatusCode>> {
    pub kind: T,
    pub body: ApiErrorBody,
}
impl<T: Into<StatusCode>> From<(T, ApiErrorMessage)> for ApiError<T> {
    fn from((kind, msg): (T, ApiErrorMessage)) -> Self {
        ApiError {
            kind,
            body: ApiErrorBody::from(msg),
        }
    }
}
impl<T: Into<StatusCode>> From<(T, ApiErrorBody)> for ApiError<T> {
    fn from((kind, body): (T, ApiErrorBody)) -> Self {
        ApiError {
            kind,
            body,
        }
    }
}
impl<T: Into<StatusCode>> IntoResponse for ApiError<T> {
    fn into_response(self) -> Response {
        let status: StatusCode = self.kind.into();
        (status, Json(self.body)).into_response()
    }
}

pub fn setup_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(|| async {"Webserver is running"}))
        .merge(auth::setup_router())
        .merge(dashboard::setup_router())
        .merge(browse::setup_router())
        .merge(upload::setup_router())
        .merge(download::setup_router())
        .merge(create_directory::setup_router())
        
        .layer(CookieManagerLayer::new())
        .layer(setup_cors())
        .with_state(state)
}

fn setup_cors() -> cors::CorsLayer {
    cors::CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().expect("Failed to create HeaderValue"))
        .allow_methods(cors::AllowMethods::list([
            Method::GET,
            Method::PUT,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(cors::AllowHeaders::list([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::SET_COOKIE
        ]))
        .allow_credentials(true)
}