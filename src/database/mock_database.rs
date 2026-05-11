use std::collections::HashMap;
use api::id::ID;
use crate::database::{DatabaseError, RepositoryTrait};
use crate::model::session::Session;
use crate::model::user::User;
use std::collections::hash_map::Entry;
use std::sync::Mutex;
use tracing::info;

pub struct MockDatabase {
    users: Mutex<HashMap<ID<User>, User>>,
    sessions: Mutex<HashMap<ID<Session>, Session>>,
}
impl MockDatabase {
    pub fn new() -> Self {
        info!("Initializing Mock Database");

        Self{
            users: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RepositoryTrait for MockDatabase {
    async fn get_user(&self, id: &ID<User>) -> Result<Option<User>, DatabaseError> {
        let users = self.users.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;
        Ok(users.get(&id).cloned())
    }
    
    async fn find_user_by_name(&self, name: &str) -> Result<Option<User>, DatabaseError> {
        let users = self.users.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;

        Ok(users.iter().find(|(_, user)| user.name.eq(name)).map(|(_, user)| user).cloned())
    }
    
    async fn create_user(&self, user: User) -> Result<(), DatabaseError> {
        let mut users = self.users.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;
        
        match users.entry(user.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(user);
                Ok(())
            }
            Entry::Occupied(entry) => Err(DatabaseError::UniqueViolation(format!("Entry is already occupied for: {}", entry.key().value()))),
        }
    }

    async fn get_session(&self, id: &ID<Session>) -> Result<Option<Session>, DatabaseError> {
        let sessions = self.sessions.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;

        Ok(sessions.get(id).cloned())
    }

    async fn update_session(&self, session: Session) -> Result<(), DatabaseError> {
        let mut sessions = self.sessions.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;

        match sessions.entry(session.id.clone()) {
            Entry::Occupied(mut entry) => {
                entry.insert(session);
                Ok(())
            },
            Entry::Vacant(entry) => Err(DatabaseError::Other(format!("Entry could not be found for: {}", entry.key().value())))
        }
    }

    async fn create_session(&self, session: Session) -> Result<(), DatabaseError> {
        let mut sessions = self.sessions.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;

        match sessions.entry(session.id.clone()) {
            Entry::Occupied(entry) => Err(DatabaseError::UniqueViolation(format!("Entry is already occupied for: {}", entry.key().value()))),
            Entry::Vacant(entry) => {
                entry.insert(session);
                Ok(())
            }
        }
    }

    async fn delete_session(&self, id: &ID<Session>) -> Result<(), DatabaseError> {
        let mut sessions = self.sessions.lock()
            .map_err(|err| DatabaseError::Other(format!("Failed to lock DB Mutex: {err}")))?;

        sessions.remove(id);
        
        Ok(())
    }
}
