use std::fmt;
use std::fmt::{Debug, Formatter};
use api::id::ID;
use crate::model::user::User;
use crate::model::session::Session;

mod mock_database;
mod database;

pub use database::MySQLDatabaseConfig;

pub enum DatabaseConfig {
    MySQL(MySQLDatabaseConfig),
    Mock,
}

pub async fn setup_database(config: &DatabaseConfig) -> Result<Box<dyn RepositoryTrait>, String> {
    Ok(match config { 
        DatabaseConfig::Mock => Box::new(mock_database::MockDatabase::new()),
        DatabaseConfig::MySQL(config) => Box::new(database::Database::new(config)
            .await.map_err(|err| err.to_string())?),
    })
}

#[derive(Debug)]
pub enum DatabaseError {
    UniqueViolation(String),
    ForeignKeyViolation(String),
    NotNullViolation(String),
    CheckViolation(String),
    Other(String),
}
impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UniqueViolation(msg)      => write!(f, "UniqueViolation: {msg}"),
            Self::ForeignKeyViolation(msg)  => write!(f, "ForeignKeyViolation: {msg}"),
            Self::NotNullViolation(msg)     => write!(f, "NotNullViolation: {msg}"),
            Self::CheckViolation(msg)       => write!(f, "CheckViolation: {msg}"),
            Self::Other(msg)                => write!(f, "{msg}"),
        }
    }
}
impl From<sqlx::Error> for DatabaseError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Database(error) => {
                let msg = error.message().to_string();
                match error.kind() {
                    sqlx::error::ErrorKind::UniqueViolation => DatabaseError::UniqueViolation(msg),
                    sqlx::error::ErrorKind::ForeignKeyViolation => DatabaseError::ForeignKeyViolation(msg),
                    sqlx::error::ErrorKind::NotNullViolation => DatabaseError::NotNullViolation(msg),
                    sqlx::error::ErrorKind::CheckViolation => DatabaseError::CheckViolation(msg),
                    sqlx::error::ErrorKind::Other | _ => DatabaseError::Other(msg),
                }
            },
            _ => DatabaseError::Other(value.to_string()),
        }
    }
}

#[async_trait::async_trait]
pub trait RepositoryTrait: Send + Sync {
    async fn get_user(&self, id: &ID<User>) -> Result<Option<User>, DatabaseError>;
    async fn find_user_by_name(&self, name: &str) -> Result<Option<User>, DatabaseError>;
    async fn create_user(&self, user: User) -> Result<(), DatabaseError>;

    async fn get_session(&self, id: &ID<Session>) -> Result<Option<Session>, DatabaseError>;
    async fn update_session(&self, session: Session) -> Result<(), DatabaseError>;
    async fn create_session(&self, session: Session) -> Result<(), DatabaseError>;
    async fn delete_session(&self, id: &ID<Session>) -> Result<(), DatabaseError>;
}
