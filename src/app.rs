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
pub struct App {
    pub file_manager: FileManager,
    list_state: ListState,
}

impl App {
    pub fn new(file_manager: FileManager) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { file_manager, list_state: state }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area());
            })?;
            match crossterm::event::read()? {
                Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => {
                    if code == KeyCode::Char('q') {
                        break Ok(());
                    }
                    if code == KeyCode::Up {
                        self.list_state.scroll_up_by(1);
                    }
                    if code == KeyCode::Down {
                        self.list_state.scroll_down_by(1);
                    }
                },
                _ => {}
            }
        }
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints(vec![
                                        Constraint::Length(3),
                                        Constraint::Percentage(100),
                                    ])
                                    .split(area);
        let title_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Percentage(60),
                                            Constraint::Percentage(40),
                                        ])
                                        .split(layout[0]);
        let files_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Percentage(70),
                                            Constraint::Percentage(30),
                                        ])
                                        .split(layout[1]);
        
        let inside_file_layout = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints(vec![
                                            Constraint::Length(1),
                                            Constraint::Length(1),
                                            Constraint::Percentage(100),
                                        ])
                                        .split(files_layout[0]);

        // path Block
        let path = Paragraph::new(Text::from(self.file_manager.path().to_string_lossy().to_string()).centered()).block(Block::bordered().title(Line::from(" Path ").centered())).render(title_layout[0], buf);

        let header_row = Paragraph::new(Text::from(" Name | Size | Modified | Type ")).render(inside_file_layout[1], buf);

        let files = Block::bordered().title(Line::from(" Files ").centered()).render(files_layout[0], buf);

        let items_layout = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(vec![
                                            Constraint::Length(1),
                                            Constraint::Percentage(100),
                                        ])
                                        .split(inside_file_layout[2]);

        let items = self.file_manager.files().iter().map(|file| {
            Line::from(format!("{} {} ", file.name, file.file_type))
        }).collect::<Vec<Line>>();
        
        let list = List::new(items).highlight_symbol(">").repeat_highlight_symbol(true);
        StatefulWidget::render(list, items_layout[1], buf, &mut self.list_state);
        
        // file information Block on the right hand side
        let file_info = Block::bordered().title(Line::from(" File Info ").centered()).render(files_layout[1], buf);
    }
}

