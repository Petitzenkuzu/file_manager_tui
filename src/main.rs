mod file_manager;
use file_manager::FileManager;
mod file;
use file::File;
use std::fs;
use std::path::PathBuf;
use std::env;
fn main() -> std::io::Result<()> {
    let path = env::current_dir().unwrap_or(PathBuf::from("/"));
    let mut file_manager = FileManager::new(path.clone()).unwrap();
    let files = fs::read_dir(&path).unwrap();
    for file in files {
        let file = file.unwrap();
        if file.metadata().unwrap().file_type().is_symlink() && file.file_name() == "trois.txt" {
            file_manager.open_link(File::try_from(file).unwrap()).unwrap();
            println!("File manager: {:#?}", file_manager.path.display());
        }
    }
    Ok(())
}
