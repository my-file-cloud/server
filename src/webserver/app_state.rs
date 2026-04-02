use crate::database::{ RepositoryTrait};
use crate::storage::Storage;

pub struct AppState {
    pub jwt_secret: Vec<u8>,
    pub database:   Box<dyn RepositoryTrait>,
    pub storage:    Storage,
}