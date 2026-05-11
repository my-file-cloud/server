use std::sync::Arc;

use axum::{extract::FromRequestParts, http::{Method, Request, Uri}};
use api::id::ID;
use tower_cookies::{Cookie, Cookies};

use crate::{
    model::user::User,
    storage::Storage,
    webserver::{
        app_state::AppState,
        jwt::AccessTokenClaims,
        router::auth::authenticated_user::AuthenticatedUser,
    },
};

#[tokio::test]
async fn from_request_parts() {
    let jwt_secret = b"secret".to_vec();

    let user_id: ID<User> = ID::new();
    let access_token = AccessTokenClaims::create_token(15 * 60, &user_id, &jwt_secret)
        .expect("Failed to create access token");

    let app_state = Arc::new(AppState {
        jwt_secret,
        database: crate::database::setup_database(&crate::database::DatabaseConfig::Mock)
            .await.expect("Failed to initialize Database"),
        storage: Storage::new(std::env::temp_dir())
            .expect("Failed to initialize storage"),
    });

    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/"))
        .body(())
        .expect("Failed to build request");

    let cookies = Cookies::default();
    cookies.add(Cookie::new("access_token", access_token));
    request.extensions_mut().insert(cookies);

    let (mut parts, _body) = request.into_parts();

    let result = AuthenticatedUser::from_request_parts(&mut parts, &app_state).await;

    assert!(result.is_ok(), "expected Ok but got Err");
    assert_eq!(result.ok().unwrap().user_id, user_id);
}
