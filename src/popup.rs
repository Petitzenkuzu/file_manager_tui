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
use crate::file::FileType;

pub enum Popup {
    Create{file_type: FileType, name: String},
}

impl Widget for &mut Popup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Popup::Create{file_type, name} => {
                let display_square = area.centered(Constraint::Length(20), Constraint::Length(20));
                let display_square = Block::bordered().border_style(Style::default().fg(Color::White)).render(display_square, buf);
            },
        }
    }
}