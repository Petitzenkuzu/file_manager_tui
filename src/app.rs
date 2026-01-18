use crate::file_manager::FileManager;
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, List, Padding, Paragraph, StatefulWidget, Widget}
};
use ratatui::style::{Style, Color};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;
use std::time::Duration;
use ratatui::layout::{Layout, Direction, Constraint};
use std::path::PathBuf;
use ratatui::widgets::ListState;
use crate::file;
use crate::utility::string::{expand_or_truncate, center};
use crate::file_manager::Action;
use std::cmp::max;

// Min char size width for the name column
pub static MIN_NAME_WIDTH: usize = 12;
// Max char size width for the size column
pub static MAX_SIZE_WIDTH: usize = 10;
// Max char size width for the type column
pub static MAX_TYPE_WIDTH: usize = 13;
// Max char size width for the modified column
pub static MAX_MODIFIED_WIDTH: usize = 20;


pub struct App {
    pub file_manager: FileManager,
    list_state: ListState,
    max_name_width: usize,
}

impl App {
    pub fn new(file_manager: FileManager) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { file_manager, list_state: state, max_name_width: MIN_NAME_WIDTH }
    }

    pub fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
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

    pub fn dispatch(&mut self, code: KeyCode) -> io::Result<()> {
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
                let res = self.file_manager.dispatch(Action::Open(self.list_state.selected().unwrap_or(0)));
                if res.is_err() {
                    eprintln!("Error: {}", res.err().unwrap());
                }
                else {
                    self.list_state.select(Some(0));
                }
            },
            KeyCode::Backspace => {
                let res = self.file_manager.dispatch(Action::GoToParent);
                if res.is_err() {
                    eprintln!("Error: {}", res.err().unwrap());
                }
                else {
                    self.list_state.select(Some(0));
                }
            },
            _ => {}
        }
        Ok(())
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // split the screen into 2 parts vertically top (Title) and bottom (Files/informations)
        let layout = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints(vec![
                                        Constraint::Length(3),
                                        Constraint::Percentage(100),
                                    ])
                                    .split(area);

        // split the top part into 2 parts horizontally left (Path) and right (nothing for the moment)
        let title_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Percentage(60),
                                            Constraint::Percentage(40),
                                        ])
                                        .split(layout[0]);
        // split the bottom part into 2 parts horizontally left (Files) and right (File informations)
        let files_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Percentage(70),
                                            Constraint::Percentage(30),
                                        ])
                                        .split(layout[1]);
        
        // split the left part into 3 parts vertically top (padding), middle (categories) and bottom (Files)
        let inside_file_layout = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints(vec![
                                            Constraint::Length(1),
                                            Constraint::Percentage(100),
                                        ])
                                        .split(files_layout[0]);

        // path Block on the top left hand side
        let _path = Paragraph::new(Text::from(self.file_manager.path().to_string_lossy().to_string()).centered()).block(Block::bordered().title(Line::from(" Path ").centered())).render(title_layout[0], buf);

        self.max_name_width = max(MIN_NAME_WIDTH, (files_layout[0].width as usize).saturating_sub(5+MAX_TYPE_WIDTH+MAX_MODIFIED_WIDTH+MAX_SIZE_WIDTH));

        // header row in the file section under the padding
        let name_category = expand_or_truncate(center("Name".to_string(), self.max_name_width), self.max_name_width);
        let size_category = expand_or_truncate(center("Size".to_string(), MAX_SIZE_WIDTH), MAX_SIZE_WIDTH);
        let type_category = expand_or_truncate(center("Type".to_string(), MAX_TYPE_WIDTH), MAX_TYPE_WIDTH);
        let modified_category = expand_or_truncate(center("Modified".to_string(), MAX_MODIFIED_WIDTH), MAX_MODIFIED_WIDTH);

        let _header_row = Line::from(format!("    {}{}{}{}", name_category, size_category, type_category, modified_category)).render(inside_file_layout[0], buf);

        // files Block on the left hand side
        let files = Block::bordered();

        let items_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Length(1),
                                            Constraint::Percentage(100),
                                        ])
                                        .split(inside_file_layout[1]);

        let items = self.file_manager.files().iter().map(|file| {
            file.to_line(self.max_name_width, MAX_SIZE_WIDTH, MAX_TYPE_WIDTH, MAX_MODIFIED_WIDTH)
        }).collect::<Vec<Line>>();
        
        let list = List::new(items).highlight_symbol("->").repeat_highlight_symbol(true).block(files);
        StatefulWidget::render(list, items_layout[1], buf, &mut self.list_state);
        
        // file information Block on the right hand side
        let _file_info = Block::bordered().title(Line::from(" File Info ").centered()).render(files_layout[1], buf);
    }
}

