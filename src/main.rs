mod file_manager;
use file_manager::FileManager;

mod file;
use file::File;

mod app;
use app::App;

use std::fs;
use std::path::PathBuf;
use std::env;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use ratatui::style::{Style, Color};

fn main() -> std::io::Result<()> {
    let path = env::current_dir().unwrap_or(PathBuf::from("/"));
    let mut file_manager = FileManager::new(path.clone()).unwrap();
    let mut app = App::new(file_manager);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
