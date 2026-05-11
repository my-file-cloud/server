use api::route::dashboard::{DashboardErrorKind, DashboardResponse};
use std::sync::Arc;
use axum::extract::State;
use axum::Router;
use axum::routing::get;
use tracing::error;
use api::{ApiErrorMessage};
use crate::webserver::app_state::AppState;
use crate::webserver::router::{ApiError, ApiResponse};
use crate::webserver::router::auth::authenticated_user::AuthenticatedUser;

pub fn setup_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard", get(dashboard))
}

pub async fn dashboard(State(state): State<Arc<AppState>>, authenticated_user: AuthenticatedUser) -> Result<ApiResponse<DashboardResponse>, ApiError<DashboardErrorKind>> {
    let user = match state.database.get_user(&authenticated_user.user_id).await
        .map_err(|err| -> ApiError<DashboardErrorKind> {
            error!("Failed to get user: {err}");
            (DashboardErrorKind::InternalServerError, ApiErrorMessage::InternalServerError).into()
        })? {
        Some(user) => user,
        None => {
            error!("Was not able to get user specified in claims");
            return Err((DashboardErrorKind::NotFound, ApiErrorMessage::InternalServerError).into()) 
        },
    };
    
    Ok(DashboardResponse {
        name: user.name,
    }.into())
}
