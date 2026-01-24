use thiserror::Error;
use std::path::PathBuf;
use std::fs;
use std::fs::DirEntry;
use crate::file_manager::File;
pub struct FsLightService;

impl FsLightService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load(&self, path: &PathBuf) -> Result<Vec<File> , LightServiceError> {
        let files : Vec<File> = fs::read_dir(&path)?.filter_map(|entry : Result<DirEntry, std::io::Error>| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        Ok(files)
    }
}

pub enum LightServiceAction {
    Load(PathBuf),
}

#[derive(Error, Debug)]
pub enum LightServiceError {
    #[error("Error loading files: {0}")]
    LoadFailed(String),
}

impl From<std::io::Error> for LightServiceError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => LightServiceError::LoadFailed("Path not found".to_string()),
            std::io::ErrorKind::PermissionDenied => LightServiceError::LoadFailed("Permission denied".to_string()),
            std::io::ErrorKind::NotADirectory => LightServiceError::LoadFailed("Path is not a directory".to_string()),
            _ => LightServiceError::LoadFailed(error.to_string()),
        }
    }
}