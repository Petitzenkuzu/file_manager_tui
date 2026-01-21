use crate::file_manager::File;
use std::path::PathBuf;
use std::fs;
use std::fs::DirEntry;
use thiserror::Error;

pub struct FileStorage {
    files: Vec<File>,
}

// public utility methods
impl FileStorage {
    pub fn new(path: &PathBuf) -> std::io::Result<Self> {
        let files = fs::read_dir(&path)?.filter_map(|entry| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        Ok(Self { files })
    }
    pub fn files(&self) -> &Vec<File> {
        &self.files
    }
    pub fn dispatch(&mut self, action: StorageAction) -> Result<StorageResponse, StorageError> {
        match action {
            StorageAction::Load(path) => self.load(path),
        }
    }
}   

// private utility methods
impl FileStorage {
    fn load(&mut self, path: &PathBuf) -> Result<StorageResponse , StorageError> {
        let files : Vec<File> = fs::read_dir(&path)?.filter_map(|entry : Result<DirEntry, std::io::Error>| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        self.files = files;
        Ok(StorageResponse::Loaded)
    }
}
pub enum StorageAction<'a> {
    Load(&'a PathBuf),
}

pub enum StorageResponse {
    Loaded,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    LoadFailed(String),
}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => StorageError::LoadFailed("Path not found".to_string()),
            std::io::ErrorKind::PermissionDenied => StorageError::LoadFailed("Permission denied".to_string()),
            std::io::ErrorKind::NotADirectory => StorageError::LoadFailed("Path is not a directory".to_string()),
            _ => StorageError::LoadFailed("Something went wrong".to_string()),
        }
    }
}


