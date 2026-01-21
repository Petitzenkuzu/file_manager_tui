use std::path::PathBuf;
use std::fs;
use crate::file_manager::FileStorage;
use crate::file_manager::{StorageAction, StorageResponse, StorageError};
use thiserror::Error;
use crate::file_manager::{File, FileType};

pub struct FileManager {
    path: PathBuf,
    file_storage: FileStorage,
}
// public methods
impl FileManager {
    pub fn new(path :&PathBuf) -> std::io::Result<Self> {
        let file_storage = FileStorage::new(path)?;
        Ok(Self { path: path.clone(), file_storage })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn files(&self) -> &Vec<File> {
        self.file_storage.files()
    }

    pub fn dispatch(&mut self, action: FileManagerAction) -> Result<FileManagerResponse, FileManagerError> {
        match action {
            FileManagerAction::Open(index) => self.open(index),
            FileManagerAction::GoToParent => self.go_to_parent(),
            FileManagerAction::Reload => self.reload_files(),
            FileManagerAction::CreateFolder(_relative_path) => {
                todo!();
            },
        }
    }
}

// private action methods
impl FileManager {
    fn go_to_parent(&mut self) -> Result<FileManagerResponse, FileManagerError> {
        let parent = match self.path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::from("/")
        };
        match self.file_storage.dispatch(StorageAction::Load(&parent)) {
            Ok(response) => {
                match response {
                    StorageResponse::Loaded => {
                        self.path = parent;
                        Ok(FileManagerResponse::WentToParent)
                    },
                    //_ => Err(FileManagerError::GoToParentFailed("Failed to load parent directory".to_string())),
                }
            },
            Err(error) => Err(FileManagerError::GoToParentFailed(error)),
        }
    }

    fn open_folder(&mut self, file: File) -> Result<FileManagerResponse, FileManagerError> {
        let path = self.path.join(file.name());
        match self.file_storage.dispatch(StorageAction::Load(&path)) {
            Ok(response) => {
                match response {
                    StorageResponse::Loaded => {
                        self.path = path;
                        Ok(FileManagerResponse::Opened)
                    },
                }
            },
            Err(error) => Err(FileManagerError::OpenFailed(error.to_string())),
        }
    }

    fn open_file(&mut self, file: File) -> Result<FileManagerResponse, FileManagerError> {
        std::process::Command::new("cmd").arg("/c").arg("start").arg(self.path().join(file.name()).to_string_lossy().to_string()).spawn().map_err(|_e| FileManagerError::OpenFailed("Failed to open file".to_string()))?;
        Ok(FileManagerResponse::Opened)
    }

    fn open_link(&mut self, file: File) -> Result<FileManagerResponse, FileManagerError> {
        match file.file_type() {
            FileType::Link { target, is_dead } => {
                if *is_dead {
                    return Err(FileManagerError::OpenFailed("Dead Symbolic Link".to_string()));
                }
                let mut path = fs::canonicalize(target).map_err(|e| FileManagerError::OpenFailed("Failed to resolve the link".to_string()))?;

                let metadata = fs::metadata(&path).map_err(|_e| FileManagerError::OpenFailed("Failed to resolve targeted file".to_string()))?;
                if metadata.is_dir() {
                    self.file_storage.dispatch(StorageAction::Load(&path))?;
                    self.path = path;
                } else {
                    // in a first time, we go to the parent folder of the pointed file, later i'll fix this to open the file with the correct thing
                    path.pop();
                    self.file_storage.dispatch(StorageAction::Load(&path))?;
                    self.path = path;

                }
            },
            _ => return Err(FileManagerError::OpenFailed("Invalid file type".to_string())),
        }
        Ok(FileManagerResponse::Opened)
    }

    fn open(&mut self, index: usize) -> Result<FileManagerResponse, FileManagerError> {
        match self.files()[index].file_type() {
            FileType::Folder => self.open_folder(self.files()[index].clone()),
            FileType::File => self.open_file(self.files()[index].clone()),
            FileType::Link { .. } => self.open_link(self.files()[index].clone()),
            FileType::Unknown => return Err(FileManagerError::OpenFailed("Unknown file type".to_string())),
        }
    }

    fn reload_files(&mut self) -> Result<FileManagerResponse, FileManagerError> {
        match self.file_storage.dispatch(StorageAction::Load(&self.path)) {
            Ok(response) => {
                match response {
                    StorageResponse::Loaded => Ok(FileManagerResponse::Reloaded),
                }
            },
            Err(error) => Err(FileManagerError::ReloadFailed(error.to_string())),
        }
    }

    fn _create_folder(&mut self, path: PathBuf) -> std::io::Result<()> {
        fs::create_dir_all(&path)?;
        Ok(())
    }
}

pub enum FileManagerAction {
    Open(usize),
    GoToParent,
    Reload,
    CreateFolder(String),
}

pub enum FileManagerResponse {
    Opened,
    WentToParent,
    FolderCreated,
    Reloaded,
}

#[derive(Error, Debug)]
pub enum FileManagerError {
    #[error("Error opening file: {0}")]
    OpenFailed(String),
    #[error("Error going to parent directory: {0}")]
    GoToParentFailed(StorageError),
    #[error("Error creating folder: {0}")]
    CreateFolderFailed(String),
    #[error("Error reloading files: {0}")]
    ReloadFailed(String),
}

impl From<crate::file_manager::StorageError> for FileManagerError {
    fn from(error: crate::file_manager::StorageError) -> Self {
        match error {
            crate::file_manager::StorageError::LoadFailed(message) => FileManagerError::OpenFailed(message),
        }
    }
}
