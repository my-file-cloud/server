use std::fs::{DirEntry, FileType};
use std::{fmt, io};
use std::fmt::{Formatter};
use serde::{Deserialize, Serialize};
use api::route::browse::{BrowseResponseDTO, StorageContentDTO, StorageContentTypeDTO};

#[derive(Serialize, Deserialize)]
pub enum BrowseStorageContent {
    File(String),
    Directory(Vec<StorageContent>),
}
impl Into<BrowseResponseDTO> for BrowseStorageContent {
    fn into(self) -> BrowseResponseDTO {
        match self {
            Self::File(val) => BrowseResponseDTO::File(val),
            Self::Directory(val) => BrowseResponseDTO::Directory(val.into_iter().map(|item| item.into()).collect())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum StorageContentType {
    File,
    Folder,
}
impl Into<StorageContentTypeDTO> for StorageContentType {
    fn into(self) -> StorageContentTypeDTO {
        match self { 
            Self::File => StorageContentTypeDTO::File,
            Self::Folder => StorageContentTypeDTO::Folder,
        }
    }
}
impl TryFrom<FileType> for StorageContentType {
    type Error = String;
    
    fn try_from(value: FileType) -> Result<Self, Self::Error> {
        if value.is_dir() {
            Ok(Self::Folder)
        } else if value.is_file() {
            Ok(Self::File)
        } else {
            Err(String::from("Unimplemented FileType"))
        }
    }
}

#[derive(Debug)]
pub enum StorageContentFromDirEntryError {
    Io(io::Error),
    StorageContentTypeFromFileTypeFailure(String),
}
impl fmt::Display for StorageContentFromDirEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "error: {msg}"),
            Self::StorageContentTypeFromFileTypeFailure(msg) => write!(f, "failed to convert from FileType to StorageContentType: {msg}"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StorageContent {
    pub name: String,
    pub size: u64,
    pub storage_content_type: StorageContentType,
}
impl Into<StorageContentDTO> for StorageContent {
    fn into(self) -> StorageContentDTO {
        StorageContentDTO {
            name: self.name,
            size: self.size,
            storage_content_type: self.storage_content_type.into()
        }
    }
}
impl TryFrom<DirEntry> for StorageContent {
    type Error = StorageContentFromDirEntryError;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let meta = value.metadata().map_err(StorageContentFromDirEntryError::Io)?;
        Ok(Self {
                name: value
                    .file_name()
                    .to_string_lossy().to_string(),
                size: meta.len(),
                storage_content_type: value
                    .file_type().map_err(StorageContentFromDirEntryError::Io)?
                    .try_into().map_err(StorageContentFromDirEntryError::StorageContentTypeFromFileTypeFailure)?,
        })
    }
}
