use std::path::PathBuf;
use std::fs;
use thiserror::Error;
use crate::file_manager::{File, FileType};
use std::sync::mpsc;
use crate::workers::LightWorkerMessage;
use crate::workers::LightWorkerAction;
use std::sync::mpsc::SendError;
use crate::workers::LightWorkerResponse;

pub struct FileManager {
    path: PathBuf,
    files: Vec<File>,
    light_sync_id: usize,
    light_worker_channel: mpsc::Sender<LightWorkerMessage>,
}
// public methods
impl FileManager {
    pub fn new(path :&PathBuf, light_sync_id: usize, light_worker_channel: mpsc::Sender<LightWorkerMessage>) -> Self {
        Self { path: path.clone(), files: Vec::new(), light_sync_id, light_worker_channel }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn files(&self) -> &Vec<File> {
        &self.files
    }

    pub fn light_sync_id(&self) -> usize {
        self.light_sync_id
    }

    pub fn increment_light_sync_id(&mut self) {
        self.light_sync_id = (self.light_sync_id + 1) % usize::MAX;
    }

    pub fn set_light_worker_channel(&mut self, channel: mpsc::Sender<LightWorkerMessage>) {
        self.light_worker_channel = channel;
    }

    pub fn shutdown(&self) {
        let _ = self.light_worker_channel.send(LightWorkerMessage::Shutdown);
    }

    pub fn dispatch(&mut self, action: FileManagerAction) -> Result<(), FileManagerError> {
        match action {
            FileManagerAction::Open(index) => self.open(index),
            FileManagerAction::GoToParent => self.go_to_parent(),
            FileManagerAction::Reload => self.reload_files(),
            FileManagerAction::CreateFolder(_relative_path) => {
                todo!();
            },
        }
    }

    pub fn consume_response(&mut self, response: LightWorkerResponse){
        match response {
            LightWorkerResponse::Loaded(files, path) => {
                self.path = path;
                self.files = files;
                self.increment_light_sync_id();
            },
        }
    }
}

// private action methods
impl FileManager {
    fn go_to_parent(&mut self) -> Result<(), FileManagerError> {
        let parent = match self.path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::from("/")
        };
        self.light_worker_channel.send(LightWorkerMessage::WorkerAction{sync_id: self.light_sync_id, action: LightWorkerAction::Load(parent)})?;
        Ok(())
    }

    fn open_folder(&self, path: PathBuf) -> Result<(), FileManagerError> {
        self.light_worker_channel.send(LightWorkerMessage::WorkerAction{sync_id: self.light_sync_id, action: LightWorkerAction::Load(path)})?;
        Ok(())
    }

    fn open_file(&self, path: PathBuf) -> Result<(), FileManagerError> {
        std::process::Command::new("cmd").arg("/c").arg("start").arg(path.to_string_lossy().to_string()).spawn().map_err(|_e| FileManagerError::OpenFileFailed("Failed to open file".to_string()))?;
        Ok(())
    }

    fn open_link(&self, file: &File) -> Result<(), FileManagerError> {
        match file.file_type() {
            FileType::Link { target, is_dead } => {
                if *is_dead {
                    return Err(FileManagerError::OpenFileFailed("Dead Symbolic Link".to_string()));
                }
                let path = fs::canonicalize(target).map_err(|_e| FileManagerError::OpenFileFailed("Failed to resolve the link".to_string()))?;

                let metadata = fs::metadata(&path).map_err(|_e| FileManagerError::OpenFileFailed("Failed to resolve targeted file".to_string()))?;

                if metadata.is_dir() {
                    self.open_folder(path)?;
                } else {
                    self.open_file(path)?;
                }
            },
            _ => return Err(FileManagerError::OpenUnknownFileType("Invalid file type".to_string())),
        }
        Ok(())
    }

    fn open(&self, index: usize) -> Result<(), FileManagerError> {
        match self.files()[index].file_type() {
            FileType::Folder => self.open_folder(self.path.join(self.files[index].name())),
            FileType::File => self.open_file(self.path.join(self.files[index].name())),
            FileType::Link { .. } => self.open_link(&self.files[index]),
            FileType::Unknown => return Err(FileManagerError::OpenUnknownFileType("Unknown file type".to_string())),
        }
    }

    fn reload_files(&self) -> Result<(), FileManagerError> {
        self.light_worker_channel.send(LightWorkerMessage::WorkerAction{sync_id: self.light_sync_id, action: LightWorkerAction::Load(self.path.clone())})?;
        Ok(())
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

#[derive(Error, Debug)]
pub enum FileManagerError {
    #[error("Error sending message: {0}")]
    SendMessageFailed(String),
    #[error("Error opening file: {0}")]
    OpenFileFailed(String),
    #[error("Error opening unknown file type: {0}")]
    OpenUnknownFileType(String),
}

impl From<SendError<LightWorkerMessage>> for FileManagerError {
    fn from(error: SendError<LightWorkerMessage>) -> Self {
        FileManagerError::SendMessageFailed(error.to_string())
    }
}
