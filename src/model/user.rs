use serde::Deserialize;
use api::id::ID;

#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: ID<User>,
    pub name: String,
    pub password: String,
}
impl User {
    pub fn new(name: String, password: String) -> Self {
        Self {
            id: ID::new(),
            name, 
            password,
        }
    }
}
