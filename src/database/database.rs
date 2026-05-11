use tracing::info;
use api::id::ID;
use crate::database::{DatabaseError, RepositoryTrait};
use crate::model::session::Session;
use crate::model::user::User;

pub struct MySQLDatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
}

pub struct Database {
    pool: sqlx::Pool<sqlx::MySql>
}
impl Database {
    pub async fn new(config: &MySQLDatabaseConfig) -> Result<Self, sqlx::Error> {
        info!("Initializing Database");
        
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url).await?;
        
        Ok(Self{
            pool,
        })
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    pub id: String,
    pub name: String,
    pub password: String,
}
impl Into<User> for UserRow {
    fn into(self) -> User {
        User {
            id: ID::from_value(self.id),
            name: self.name,
            password: self.password,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    pub id: String,
    pub refresh_token: String,
    pub user_id: String,
}
impl Into<Session> for SessionRow {
    fn into(self) -> Session {
        Session {
            id: ID::from_value(self.id),
            user_id: ID::from_value(self.user_id),
            refresh_token: self.refresh_token,
        }
    }
}


#[async_trait::async_trait]
impl RepositoryTrait for Database {
    async fn get_user(&self, id: &ID<User>) -> Result<Option<User>, DatabaseError> {
        let res: Option<UserRow> = sqlx::query_as("select * from bas_user where id = ?")
            .bind(id.value())
            .fetch_optional(&self.pool).await?;
        
        Ok(res.map(|row| row.into()))
    }

    async fn find_user_by_name(&self, name: &str) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query_as::<_, UserRow>("select id, name, password from bas_user where name = ?")
            .bind(name)
            .fetch_optional(&self.pool).await?;

        Ok(row.map(|row| row.into()))
    }

    async fn create_user(&self, user: User) -> Result<(), DatabaseError> {
        sqlx::query("insert into bas_user (id, name, password) values (?, ?, ?)")
            .bind(user.id.value())
            .bind(user.name)
            .bind(user.password)
            .execute(&self.pool).await?;
        
        Ok(())
    }
    
    async fn get_session(&self, id: &ID<Session>) -> Result<Option<Session>, DatabaseError> {
        let res: Option<SessionRow> = sqlx::query_as("select * from bas_session where id = ?")
            .bind(id.value())
            .fetch_optional(&self.pool).await?;
        
        Ok(res.map(|row|row.into()))
    }

    async fn update_session(&self, session: Session) -> Result<(), DatabaseError> {
        sqlx::query("update bas_session set refresh_token = ? where id = ?")
            .bind(session.refresh_token)
            .bind(session.id.value())
            .execute(&self.pool).await?;

        Ok(())
    }

    async fn create_session(&self, session: Session) -> Result<(), DatabaseError> {
        sqlx::query("insert into bas_session (id, refresh_token, user_id) values (?, ?, ?)")
            .bind(session.id.value())
            .bind(session.refresh_token)
            .bind(session.user_id.value())
            .execute(&self.pool).await?;
        
        Ok(())
    }

    async fn delete_session(&self, id: &ID<Session>) -> Result<(), DatabaseError> {
       sqlx::query("delete from bas_session where id = ?")
            .bind(id.value())
            .execute(&self.pool).await?;

        Ok(())
    }
}
