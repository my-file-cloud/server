use std::io;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub mod router;
pub mod app_state;
pub mod jwt;

use crate::webserver::app_state::AppState;

pub async fn start(port: u16, app_state: AppState, client_origin: String) -> Result<(), io::Error> {
    info!("Starting webserver on http://localhost:{}", port);
    
    let state = Arc::new(app_state);
    let router = router::setup_router(state, client_origin);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    
    axum::serve(listener, router).await
}
