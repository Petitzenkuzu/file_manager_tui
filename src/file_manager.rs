use std::path::PathBuf;
use std::fs;
use crate::file::FileType;
use crate::file::File;
pub struct FileManager {
    pub path: PathBuf,
    pub files: Vec<File>,
}
// public methods
impl FileManager {
    pub fn new(path :PathBuf) -> std::io::Result<Self> {
        let files = fs::read_dir(&path)?.filter_map(|entry| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        Ok(Self { path, files })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn files(&self) -> &Vec<File> {
        &self.files
    }

    pub fn dispatch(&mut self, action: Action) -> std::io::Result<()> {
        match action {
            Action::Open(index) => self.open(index),
            Action::GoToParent => self.go_to_parent(),
            Action::Reload => self.reload_files(),
        }
    }
}

// private action methods
impl FileManager {
    fn go_to_parent(&mut self) -> std::io::Result<()> {
        self.path = match self.path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => self.path.clone()
        };
        self.reload_files()?;
        Ok(())
    }

    fn open_folder(&mut self, file: File) -> std::io::Result<()> {
        self.path.push(&file.name);
        self.reload_files()?;
        Ok(())
    }

    fn open_file(&mut self, file: File) -> std::io::Result<()> {
        std::process::Command::new("cmd").arg("/c").arg("start").arg(self.path().join(&file.name).to_string_lossy().to_string()).spawn()?;
        Ok(())
    }

    fn open_link(&mut self, file: File) -> std::io::Result<()> {
        match &file.file_type {
            FileType::Link { target, is_dead } => {
                if *is_dead {
                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Link is dead"));
                }
                let mut path = fs::canonicalize(target)?;
                let metadata = fs::metadata(&path)?;
                if metadata.is_dir() {
                    self.path = path;
                    self.reload_files()?;
                } else {
                    // in a first time, we go to the parent folder of the pointed file, later i'll fix this to open the file with the correct thing
                    path.pop();
                    self.path = path;
                    self.reload_files()?;

                }
            },
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid file type")),
        }
        Ok(())
    }

    fn open(&mut self, index: usize) -> std::io::Result<()> {
        match self.files[index].file_type {
            FileType::Folder => self.open_folder(self.files[index].clone())?,
            FileType::File => self.open_file(self.files[index].clone())?,
            FileType::Link { .. } => self.open_link(self.files[index].clone())?,
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid file type")),
        }
        Ok(())
    }
}

// private utility methods
impl FileManager {
    fn reload_files(&mut self) -> std::io::Result<()> {
        self.files = fs::read_dir(&self.path)?.filter_map(|entry| File::try_from(entry.ok()?).ok()).collect::<Vec<File>>();
        Ok(())
    }
}
pub enum Action {
    Open(usize),
    GoToParent,
    Reload,
}