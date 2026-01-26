use thiserror::Error;
use std::path::PathBuf;
use std::fs;
use std::fs::DirEntry;
use crate::file::File;
use std::fs::OpenOptions;
use std::io::Read;
pub struct FsLightService;

impl FsLightService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load(&self, path: &PathBuf) -> Result<Vec<File> , LightServiceError> {
        let files : Vec<File> = fs::read_dir(&path)?.filter_map(|entry : Result<DirEntry, std::io::Error>| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        Ok(files)
    }

    pub fn read(&self, path: &PathBuf) -> Result<String, LightServiceError> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut content = Vec::with_capacity(1024);
        file.read_to_end(&mut content)?;
        match String::from_utf8(content) {
            Ok(content) => Ok(content),
            Err(error) => Err(LightServiceError::ReadFailed(format!("Invalid UTF-8 encoding ({})", error.to_string()))),
        }
    }
}

#[derive(Error, Debug)]
pub enum LightServiceError {
    #[error("{0}")]
    LoadFailed(String),
    #[error("{0}")]
    ReadFailed(String),
}

impl From<std::io::Error> for LightServiceError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => LightServiceError::LoadFailed(format!("Path not found ({})", error.to_string())),
            std::io::ErrorKind::PermissionDenied => LightServiceError::LoadFailed(format!("Permission denied ({})", error.to_string())),
            std::io::ErrorKind::NotADirectory => LightServiceError::LoadFailed(format!("Path is not a directory ({})", error.to_string())),
            _ => LightServiceError::LoadFailed(error.to_string()),
        }
    }
}