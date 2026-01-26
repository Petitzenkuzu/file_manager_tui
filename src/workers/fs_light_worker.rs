use std::sync::mpsc;
use std::path::PathBuf;
use crate::file::File;
use thiserror::Error;
use crate::workers::LightServiceError;
use crate::workers::FsLightService;
use std::sync::mpsc::SendError;

pub struct FsLightWorker {
    sync_id: usize,
    input_channel: mpsc::Receiver<LightWorkerMessage>,
    output_channel: mpsc::Sender<Result<LightWorkerResponse, LightWorkerError>>,
    service: FsLightService,
}

impl FsLightWorker {
    pub fn new(sync_id: usize, input_channel: mpsc::Receiver<LightWorkerMessage>, output_channel: mpsc::Sender<Result<LightWorkerResponse, LightWorkerError>>) -> Self {
        Self { sync_id, input_channel, output_channel, service: FsLightService::new() }
    }

    pub fn run(&mut self) -> Result<(), LightWorkerError> {
        loop {
            match self.input_channel.recv() {
                Ok(message) => {
                    match message {
                        LightWorkerMessage::Shutdown => break Ok(()),
                        LightWorkerMessage::WorkerAction{sync_id, action} => {
                            if sync_id != self.sync_id {
                                continue;
                            }
                            self.handle_action(action)?;
                            self.sync_id = (self.sync_id + 1) % usize::MAX;
                        }
                        
                    }
                },
                Err(_) => return Err(LightWorkerError::ReceiveMessageFailed),
            }
        }
    }

}

impl FsLightWorker {
    fn handle_action(&mut self, action: LightWorkerAction) -> Result<(), LightWorkerError> {
        match action {
            LightWorkerAction::Load(path) => {
                self.load(path)
            },
            LightWorkerAction::Read(path) => {
                self.read(path)
            },
        }
    }

    fn load(&mut self, path: PathBuf) -> Result<(), LightWorkerError> {
        match self.service.load(&path) {
            Ok(response) => {
                self.output_channel.send(Ok(LightWorkerResponse::Loaded(response, path)))?;
                Ok(())
            },
            Err(e) => Err(LightWorkerError::LoadFailed(e)),
        }
    }

    fn read(&mut self, path: PathBuf) -> Result<(), LightWorkerError> {
        match self.service.read(&path) {
            Ok(response) => {
                self.output_channel.send(Ok(LightWorkerResponse::Read(response, path)))?;
                Ok(())
            },
            Err(e) => {
                self.output_channel.send(Err(LightWorkerError::ReadFailed(e)))?;
                Ok(())
            }
        }
    }
}

pub enum LightWorkerMessage {
    WorkerAction{sync_id: usize, action: LightWorkerAction},
    Shutdown,
}

pub enum LightWorkerAction {
    Load(PathBuf),
    Read(PathBuf),
}

pub enum LightWorkerResponse {
    Loaded(Vec<File>, PathBuf),
    Read(String, PathBuf)
}

#[derive(Error, Debug)] 
pub enum LightWorkerError {
    #[error("Error loading files: {0}")]
    LoadFailed(LightServiceError),
    #[error("Error reading file: {0}")]
    ReadFailed(LightServiceError),
    #[error("Error receiving message: UI is dead")]
    ReceiveMessageFailed,
    #[error("Error sending response: UI is dead")]
    SendResponseFailed,
}

impl From<SendError<Result<LightWorkerResponse, LightWorkerError>>> for LightWorkerError {
    fn from(_: SendError<Result<LightWorkerResponse, LightWorkerError>>) -> Self {
        LightWorkerError::SendResponseFailed
    }
}