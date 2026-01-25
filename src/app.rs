use crate::file_manager::{FileManager, FileManagerAction, FileType};
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
    popup: Popup,
    max_name_width: usize,
    light_receiver: mpsc::Receiver<Result<LightWorkerResponse, LightWorkerError>>,
}

enum FocusScreen {
    Files,
    Path,
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
            popup: Popup::None, 
            max_name_width: MIN_NAME_WIDTH 
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
        loop {

            // frame rendering
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                frame.render_widget(&mut self.popup, frame.area());
            })?;

            // input handling
            if crossterm::event::poll(Duration::from_millis(50))? {
                if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, modifiers, .. }) = crossterm::event::read()? {
                    if code == KeyCode::Char('q') || (modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c')) {
                        self.file_manager.shutdown();
                        break Ok(());
                    }
                    self.dispatch(code);
                }
            }

            // workers response handling
            while let Ok(response) = self.light_receiver.try_recv() {
                match response {
                    Ok(response) => {
                        match response {
                            LightWorkerResponse::Loaded(_, _) => {
                                self.file_manager.consume_response(response);
                                let min = self.min_selected();
                                self.list_state.select(min);
                                if let Some(min) = min {
                                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(min));
                                }
                            },
                            LightWorkerResponse::Read(..) => {
                                self.file_manager.consume_response(response);
                            },
                        }
                    },
                    Err(error) => {
                        eprintln!("Error: {}", error);
                    },
                }
            }
        }
    }

    fn dispatch(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => {
                if self.file_manager.files().is_empty() {
                    return;
                }
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => {
                        self.list_state.select(Some(0));
                        return;
                    },
                };
                if selected == 0 {
                    self.list_state.select(Some(self.file_manager.files().len() - 1));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.file_manager.files().len() - 1));
                }
                else {
                    self.list_state.scroll_up_by(1);
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(selected - 1));
                }
            },
            KeyCode::Down => {
                if self.file_manager.files().is_empty() {
                    return;
                }
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => {
                        self.list_state.select(Some(0));
                        return;
                    },
                };
                if selected == self.file_manager.files().len() - 1 {
                    self.list_state.select(Some(0));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(0));
                }
                else {
                    self.list_state.scroll_down_by(1);
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(selected + 1));
                }
            },
            KeyCode::Enter => {
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => return,
                };
                let res = self.file_manager.dispatch(FileManagerAction::Open(selected));
                if res.is_err() {
                    eprintln!("Error: {}", res.err().unwrap());
                }
                else {
                    self.list_state.select(self.min_selected());
                }
            },
            KeyCode::Backspace => {
                let res = self.file_manager.dispatch(FileManagerAction::GoToParent);
                if res.is_err() {
                    eprintln!("Error: {}", res.err().unwrap());
                }
                else {
                    self.list_state.select(self.min_selected());
                }
            },
            KeyCode::F(5) => {
                let res = self.file_manager.dispatch(FileManagerAction::Reload);
                if res.is_err() {
                    eprintln!("Error: {}", res.err().unwrap());
                }
                else {
                    self.list_state.select(self.min_selected());
                }
            },
            KeyCode::Char('c') => {
                self.popup = Popup::Create{file_type: FileType::Folder, name: String::new()};
            },
            _ => {}
        }
    }

    /**
    * @brief Returns the minimum selected index, if the list is empty, returns None
    * Returns the minimum selected index, if the list is empty, returns None
    */
    fn min_selected(&self) -> Option<usize> {
        match self.file_manager.files().len() {
            0 => None,
            _ => Some(0),
        }
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        // we shrink preview section to let space for files section if the terminal is too small
        let area_width = area.width;
        let files_section_width_constrain = if area_width / 2 > MIN_FILES_SECTION_WIDTH { Constraint::Percentage(50) } else { Constraint::Length(MIN_FILES_SECTION_WIDTH) };
        // split the screen into 2 parts horizontally left (Files) and right (File preview if available on selected file)
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                files_section_width_constrain,
                Constraint::Percentage(100),
            ])
            .split(area);

        let list_symbol = "->";
        // dynamically calculate the max name width based on the screen width and the modified time width
        self.max_name_width = std::cmp::max(crate::app::MIN_NAME_WIDTH, (main_layout[0].width as usize).saturating_sub(MODIFIED_TIME_WIDTH+list_symbol.len()));

        // split the left part into 2 parts vertically top (Path) and bottom (File list)
        let files_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Percentage(100),
            ])
            .split(main_layout[0]);

        // render the file list
        let list = List::new(self.file_manager.files().iter().map(|file| file.to_line(self.max_name_width-list_symbol.len(), MODIFIED_TIME_WIDTH)).collect::<Vec<Line>>()).block(Block::default()).highlight_symbol("->").repeat_highlight_symbol(true);
        StatefulWidget::render(list, files_layout[1], buf, &mut self.list_state);

        // render the path
        let _path_display = Paragraph::new(Text::from(self.file_manager.path().to_string_lossy().to_string())).block(Block::default().padding(Padding::new(1, 0, 1, 0))).left_aligned().render(files_layout[0], buf);

        let _preview_display = Paragraph::new(Text::from(self.file_manager.selected_file_preview_buffer())).block(Block::default().title(Line::from(" Preview ").centered())).render(main_layout[1], buf);
    
    }
}

