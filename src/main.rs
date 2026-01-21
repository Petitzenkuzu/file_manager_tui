mod file_manager;
use file_manager::FileManager;

mod app;
use app::App;

mod utility;

mod popup;

use std::path::PathBuf;
use std::env;


fn main() -> std::io::Result<()> {
    
    let path = env::current_dir().unwrap_or(PathBuf::from("/"));
    let file_manager = FileManager::new(&path).unwrap();
    let app = App::new(file_manager);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
