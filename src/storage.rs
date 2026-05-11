use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use futures::{Stream, StreamExt};
use tokio::io::DuplexStream;
use tokio_util::io::ReaderStream;
use api::id::ID;
use zip::write::FileOptions;
use zip::ZipWriter;
use crate::model::user::User;
use crate::storage::init_storage_error::InitStorageError;
use crate::storage::storage_content::{BrowseStorageContent, StorageContent};

pub mod init_storage_error;

pub mod storage_content;

pub struct GlobalStorageState;
pub struct WithUserStorageState {
    user_id: ID<User>,   
}

pub struct Storage<State = GlobalStorageState> {
    pub root_directory_path: PathBuf,
    state: State
}
impl Storage {
    pub fn new(root_directory_path: PathBuf) -> Result<Self, InitStorageError> {
        if !root_directory_path.exists() {
            Self::setup_root_directory(&root_directory_path).map_err(InitStorageError::Io)?;
        }
        if !root_directory_path.is_dir() {
            return Err(InitStorageError::InvalidRootDirectory(String::from("root directory is not a directory")))
        }

        Ok(Self {
            root_directory_path,
            state: GlobalStorageState,
        })
    }
    
    fn setup_root_directory<P: AsRef<Path>>(path: &P) -> Result<(), io::Error> {
        fs::create_dir(path)
    }

    pub fn validate_path(path: &String) -> Result<(), String> {
        if path.contains("..") {
            return Err(String::from("Path can only be direct"))
        }

        Ok(())
    }
}

impl<T> Storage<T> {
    pub fn with_user(&self, user_id: ID<User>) -> Storage<WithUserStorageState> {
        Storage {
            state: WithUserStorageState { user_id },
            root_directory_path: self.root_directory_path.clone(),
        }
    }
    
    pub fn build_global_path(&self, path: &PathBuf) -> PathBuf {
        self.root_directory_path.join(path)
    }
    
    pub fn create_user(&self, user_id: ID<User>) -> Result<PathBuf, io::Error> {
        let user_path = self.build_global_path(&PathBuf::from(user_id.value()));

        fs::create_dir(&user_path)?;

        Ok(user_path)
    }
    
    pub fn global_browse_path(&self, path: &PathBuf) -> Result<BrowseStorageContent, io::Error> {
        let path =  self.build_global_path(&path);

        let metadata = path
            .metadata()?;

        if metadata.is_file() {
            Ok(BrowseStorageContent::File(
                self.global_browse_file(&path)?
            ))
        } else if metadata.is_dir() {
            Ok(BrowseStorageContent::Directory(
                self.global_browse_directory(&path)?
            ))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Entry not found"))
        }
    }

    pub fn global_delete_path(&self, path: &PathBuf) -> Result<(), io::Error> {
        let path = self.build_global_path(&path);

        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Entry not found"))
        }

        if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Entry not found"))
        }

        Ok(())
    }

    pub fn global_browse_file(&self, path: &PathBuf) -> Result<String, io::Error> {
        Ok(fs::read_to_string(
            self.build_global_path(path)
        )?)
    }

    pub fn global_browse_directory(&self, path: &PathBuf) -> Result<Vec<StorageContent>, io::Error> {
        let path = self.build_global_path(path);
        
        Ok(fs::read_dir(&path)?
            .map(|item| item.ok())
            .filter_map(|item| {
                match item {
                    None => None,
                    Some(item) => {
                        let file_name = item.file_name().to_string_lossy().to_string();

                        match Storage::validate_path(&file_name) {
                            Ok(()) => Some(item),
                            Err(_) => None,
                        }
                    },
                }
            })
            .filter_map(|item| StorageContent::try_from(item).ok())
            .collect())
    }

    pub fn global_path_metadata(&self, path: &PathBuf) -> io::Result<fs::Metadata> {
        self.build_global_path(path).metadata()
    }

    pub async fn global_stream_file(&self, path: &PathBuf) -> io::Result<ReaderStream<tokio::fs::File>> {
        let path = self.build_global_path(path);

        let file = tokio::fs::File::open(&path).await?;

        let stream = ReaderStream::new(file);

        Ok(stream)
    }
    
    pub fn global_create_directory(&self, path: &PathBuf) -> Result<(), io::Error> {
        fs::create_dir(&self.build_global_path(path))?;
        
        Ok(())
    }
}

impl Storage<WithUserStorageState> {
    pub fn build_user_path(&self, path: &PathBuf) -> PathBuf {
        PathBuf::from(self.state.user_id.value()).join(path)
    }
    
    pub fn browse_path(&self, path: &PathBuf) -> Result<BrowseStorageContent, io::Error> {
        self.global_browse_path(&self.build_user_path(path))
    }
    
    pub fn delete_path(&self, path: &PathBuf) -> Result<(), io::Error> {
        self.global_delete_path(&self.build_user_path(path))
    }

    pub fn browse_file(&self, path: &PathBuf) -> Result<String, io::Error> {
        self.global_browse_file(&self.build_user_path(path))
    }

    pub fn browse_directory(&self, path: &PathBuf) -> Result<Vec<StorageContent>, io::Error> {
        self.global_browse_directory(&self.build_user_path(path))
    }
    
    pub fn path_metadata(&self, path: &PathBuf) -> io::Result<fs::Metadata> {
        self.global_path_metadata(&self.build_user_path(path))
    }
    
    pub async fn stream_file(&self, path: &PathBuf) -> io::Result<ReaderStream<tokio::fs::File>> {
        self.global_stream_file(&self.build_user_path(path)).await
    }
    
    pub async fn stream_directory_zip(&self, path: &PathBuf)
       -> io::Result<impl Stream<Item = Result<bytes::Bytes, io::Error>> + use<>> 
    {
        let path = self.root_directory_path.join(self.build_user_path(path));
        if !path.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Path is not a directory"))
        }
        
        let (reader, writer) = tokio::io::duplex(64 * 1024);
        
        tokio::task::spawn_blocking(move || {
            write_zip(path, writer).expect("Failed to write zip")
        });
        
        Ok(ReaderStream::new(reader)
            .map(|chunk| chunk.map(bytes::Bytes::from)))
    }
    
    pub fn create_directory(&self, path: &PathBuf) -> Result<(), io::Error> {
        self.global_create_directory(&self.build_user_path(path))
    }
}

fn write_zip(dir: PathBuf, writer: DuplexStream) -> io::Result<()> {
    let writer = tokio_util::io::SyncIoBridge::new(writer);
    let mut zip = ZipWriter::new_stream(writer);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let base = dir.clone();

    for entry in walkdir::WalkDir::new(&dir) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(&base).map_err(|err| io::Error::new(io::ErrorKind::InvalidFilename, format!("Could not extract entry name from path: {err}")))?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
