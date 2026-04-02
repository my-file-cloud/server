use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum InitStorageError {
    InvalidRootDirectory(String),
    Io(std::io::Error)
}
impl fmt::Display for InitStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRootDirectory(msg) => write!(f, "invalid root directory: {msg}"),
            Self::Io(msg) => write!(f, "error: {msg}")
        }
    }
}
