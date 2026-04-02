use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use my_file_cloud_api::id::ID;
use crate::model::session::Session;
use crate::model::user::User;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RefreshTokenClaims {
    pub iat: i64,
    pub exp: i64,
    pub user_id: ID<User>,
    pub session_id: ID<Session>,
}
impl RefreshTokenClaims {
    pub fn create_token(
        expires_in: i64,
        user_id: &ID<User>,
        session_id: &ID<Session>,
        secret: &[u8],
    ) -> jsonwebtoken::errors::Result<String> {
        let claims = {
            let now = Utc::now();
            Self {
                iat: now.timestamp(),
                exp: (now + Duration::seconds(expires_in)).timestamp(),
                user_id: user_id.clone(),
                session_id: session_id.clone(),
            }   
        };
        
        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AccessTokenClaims {
    pub iat: i64,
    pub exp: i64,
    pub user_id: ID<User>,
}
impl AccessTokenClaims {
    pub fn create_token(
        expires_in: i64,
        user_id: &ID<User>,
        secret: &[u8],
    ) -> jsonwebtoken::errors::Result<String> {
        let claims = {
            let now = Utc::now();
            Self {
                iat: now.timestamp(),
                exp: (now + Duration::seconds(expires_in)).timestamp(),
                user_id: user_id.clone(),
            }
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
    }
}

pub fn decode_token<T: DeserializeOwned>(token: &str, secret: &[u8]) -> jsonwebtoken::errors::Result<TokenData<T>> {
    decode::<T>(token, &DecodingKey::from_secret(secret), &Validation::default())
}

pub fn default_token_pair(user_id: ID<User>, session_id: ID<Session>, secret: &[u8]) -> jsonwebtoken::errors::Result<(String, String)> {
    token_pair(user_id, (15 * 60, 7*24*60*60), session_id, secret)
}

pub fn token_pair(user_id: ID<User>, (access_exp, refresh_exp): (i64, i64), session_id: ID<Session>, secret: &[u8]) -> jsonwebtoken::errors::Result<(String, String)> {
    Ok((
        AccessTokenClaims::create_token(access_exp, &user_id, secret)?,
        RefreshTokenClaims::create_token(refresh_exp, &user_id, &session_id, secret)?
    ))
}
