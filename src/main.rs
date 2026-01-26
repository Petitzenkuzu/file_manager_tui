mod file_manager;
use file_manager::FileManager;

mod file;

mod app;
use app::App;

mod utility;

mod popup;

mod workers;

use std::path::PathBuf;
use std::env;
use std::sync::mpsc;

fn main() -> std::io::Result<()> {

    let path = env::current_dir().unwrap_or(PathBuf::from("/"));

    // channels for communication between file manager -> light worker -> app
    let (file_manager_sender, light_worker_receiver) = mpsc::channel();
    let (light_worker_sender, app_receiver) = mpsc::channel();

    let file_manager = FileManager::new(&path, 0, file_manager_sender);
    let mut app = App::new(file_manager, app_receiver);

    app.spawn_light_worker(Some(light_worker_sender), Some(light_worker_receiver));

    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())

}
