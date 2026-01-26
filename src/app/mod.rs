mod input;
mod ui;

use crate::file_manager::{FileManager, FileManagerAction};
use crate::file::FileType;
use crate::workers::{LightWorkerResponse, LightWorkerError, LightWorkerMessage, FsLightWorker};
use ratatui::{
    buffer::Buffer, layout::Rect, text::{Line, Text}, widgets::{Block, List, Padding, Paragraph, StatefulWidget, Widget}
};
use crossterm::event::{ Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use ratatui::layout::{Layout, Direction, Constraint};
use ratatui::widgets::ListState;
use crate::popup::Popup;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::file::File;

// Min char size width for the name column
pub static MIN_NAME_WIDTH: usize = 20;
// Max char size width for the size column
pub static _MAX_SIZE_WIDTH: usize = 10;
// Max char size width for the type column
pub static _MAX_TYPE_WIDTH: usize = 13;
// Max char size width for the modified column
pub static MODIFIED_TIME_WIDTH: usize = 20;
// Min files section width on the UI
pub static MIN_FILES_SECTION_WIDTH: u16 = 50;


pub struct App {
    file_manager: FileManager,
    list_state: ListState,
    focus: FocusScreen,
    filter_mode: bool,
    filter_buffer: String,
    filtered_files: Vec<usize>,
    popup: Option<Popup>,
    max_name_width: usize,
    light_receiver: mpsc::Receiver<Result<LightWorkerResponse, LightWorkerError>>,
    shutdown: bool,
}

enum FocusScreen {
    Files,
    Preview,
}


impl App {
    pub fn new(file_manager: FileManager, light_receiver: mpsc::Receiver<Result<LightWorkerResponse, LightWorkerError>>) -> Self {

        let mut state = ListState::default();
        state.select(None);
        Self { 
            file_manager, 
            light_receiver,
            list_state: state, 
            focus: FocusScreen::Files, 
            popup: None, 
            max_name_width: MIN_NAME_WIDTH,
            filter_mode: false,
            filter_buffer: String::new(),
            filtered_files: Vec::new(),
            shutdown: false,
        }
    }

    pub fn spawn_light_worker(&mut self, sender: Option<mpsc::Sender<Result<LightWorkerResponse, LightWorkerError>>>, receiver: Option<mpsc::Receiver<LightWorkerMessage>>) {
        if let (Some(sender), Some(receiver)) = (sender, receiver) {
            let mut light_worker = FsLightWorker::new(self.file_manager.light_sync_id(), receiver, sender);
            thread::spawn(move || {
                let _ = light_worker.run();
            });
        }
        else {
            let (file_manager_sender, light_worker_receiver) = mpsc::channel();
            let (light_worker_sender, app_receiver) = mpsc::channel();
            self.file_manager.set_light_worker_channel(file_manager_sender);
            self.light_receiver = app_receiver;
            self.spawn_light_worker(Some(light_worker_sender), Some(light_worker_receiver));
        }
    }

    pub fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        let _ = self.file_manager.dispatch(FileManagerAction::Reload);
        while !self.shutdown {

            // frame rendering
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                if let Some(popup) = &mut self.popup {
                    frame.render_widget(popup, frame.area());
                }
            })?;

            // input handling
            while let Ok(poll_result) = crossterm::event::poll(Duration::from_millis(50)) {
                if !poll_result {
                    break;
                }
                let event = crossterm::event::read()?;
                if let Some(popup) = &mut self.popup {
                    //popup.dispatch(event);
                }
                else {
                    if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, modifiers, .. }) = event {
                        if modifiers.contains(KeyModifiers::CONTROL) && (code == KeyCode::Char('q') || code == KeyCode::Char('c')) {
                            self.file_manager.shutdown();
                            self.shutdown = true;
                            break;
                        }
                    }
                    match self.focus {
                        FocusScreen::Files => {
                            if self.filter_mode {
                                self.handle_filter_input(event);
                            }
                            else {
                                self.handle_files_input(event);
                            }
                        }
                        FocusScreen::Preview => {
                            //self.handle_preview_input(event);
                        }
                    }
                }
            }

            // workers response handling
            while let Ok(response) = self.light_receiver.try_recv() {
                match response {
                    Ok(response) => {
                        match response {
                            LightWorkerResponse::Loaded(_, _) => {
                                // load the new files into the file manager to display them in the ui
                                self.file_manager.consume_response(response);
                                if self.filter_mode {
                                    // clear the filter buffer and update the filtered files vector then select the first file in the list and dispatch the read content action for it
                                    self.filter_buffer.clear();
                                    self.update_filtered_files();
                                    let min = self.min_filtered_selected();
                                    self.list_state.select(min);
                                    if let Some(min) = min {
                                        let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[min])));
                                    }
                                }
                                else {
                                    // select the first file in the list and dispatch the read content action for it
                                    let min = self.min_selected();
                                    self.list_state.select(min);
                                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(min));
                                }
                            },
                            LightWorkerResponse::Read(..) => {
                                // update the selected file preview buffer in the file manager to display it in the ui
                                self.file_manager.consume_response(response);
                            },
                        }
                    },
                    Err(_error) => {
                        self.file_manager.increment_light_sync_id();
                    },
                }
            }
        }
        Ok(())
    }

    /// Returns the minimum index that can be selected in the file list, if the list is empty, returns None
    fn min_selected(&self) -> Option<usize> {
        match self.file_manager.files().len() {
            0 => None,
            _ => Some(0),
        }
    }

    /// Returns the minimum index that can be selected in the filtered file list, if the list is empty, returns None
    fn min_filtered_selected(&self) -> Option<usize> {
        match self.filtered_files.len() {
            0 => None,
            _ => Some(0),
        }
    }

    /// update the filtered files vector with the current filter buffer
    fn update_filtered_files(&mut self) {
        let new_filtered_files : Vec<usize> = self.file_manager.files().iter().enumerate().filter(|(_ , file)| file.name().contains(&self.filter_buffer)).map(|(index, _)| index).collect();
        self.filtered_files = new_filtered_files;
    }
}