use crate::file_manager::FileManager;
use ratatui::{
    buffer::Buffer, layout::Rect, text::{Line, Text}, widgets::{Block, List, Padding, Paragraph, StatefulWidget, Widget}
};
use crossterm::event::{ Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;
use ratatui::layout::{Layout, Direction, Constraint};
use ratatui::widgets::ListState;
use crate::file_manager::FileManagerAction;
use crate::popup::Popup;
use crate::file_manager::FileType;

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
    pub file_manager: FileManager,
    list_state: ListState,
    error_message: Option<String>,
    focus: FocusScreen,
    popup: Popup,
    input_buffer: String,
    max_name_width: usize,
}

enum FocusScreen {
    Files,
    Path,
}


impl App {
    pub fn new(file_manager: FileManager) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { file_manager, list_state: state, error_message: None, focus: FocusScreen::Files, popup: Popup::None, input_buffer: String::new(), max_name_width: MIN_NAME_WIDTH }
    }

    pub fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                frame.render_widget(&mut self.popup, frame.area());
            })?;
            match crossterm::event::read()? {
                Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => {
                    if code == KeyCode::Char('q') {
                        break Ok(());
                    }
                    else {
                        self.dispatch(code)?;
                    }
                },
                _ => {}
            }
        }
    }

    fn dispatch(&mut self, code: KeyCode) -> io::Result<()> {
        match code {
            KeyCode::Up => {
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => 0,
                };
                if selected == 0 {
                    self.list_state.select(Some(self.file_manager.files().len() - 1));
                }
                else {
                    self.list_state.scroll_up_by(1);
                }
            },
            KeyCode::Down => {
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => 0,
                };
                if selected == self.file_manager.files().len() - 1 {
                    self.list_state.select(Some(0));
                }
                else {
                    self.list_state.scroll_down_by(1);
                }
            },
            KeyCode::Enter => {
                let selected = match self.list_state.selected() {
                    Some(selected) => selected,
                    None => return Ok(()),
                };
                let res = self.file_manager.dispatch(FileManagerAction::Open(self.list_state.selected().unwrap_or(0)));
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
        Ok(())
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

        let _preview_display = Paragraph::new(Text::from("")).block(Block::default().title(Line::from(" Preview ").centered())).render(main_layout[1], buf);
    
    }
}

