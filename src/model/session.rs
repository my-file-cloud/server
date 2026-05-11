use serde::Deserialize;
use api::id::ID;
use crate::model::user::User;

#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: ID<Session>,
    pub user_id: ID<User>,
    pub refresh_token: String,
}
impl Session {
    pub fn new_id() -> ID<Session> {
        ID::new()
    }
}
