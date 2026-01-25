use std::time::SystemTime;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::fs;
use std::fmt;
use chrono::{DateTime, Local};
use ratatui::text::Line;
use crate::utility::string::expand_or_truncate;

#[derive(Debug, Clone)]
pub struct File {
    name: String,
    file_type: FileType,
    modified_time: SystemTime,
    access_time: SystemTime,
    creation_time: SystemTime,
    size: Size,
}

impl File {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }
    pub fn modified_time(&self) -> &SystemTime {
        &self.modified_time
    }
    pub fn access_time(&self) -> &SystemTime {
        &self.access_time
    }
    pub fn creation_time(&self) -> &SystemTime {
        &self.creation_time
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }
    pub fn is_folder(&self) -> bool {
        matches!(self.file_type, FileType::Folder)
    }
    pub fn is_link(&self) -> bool {
        matches!(self.file_type, FileType::Link { .. })
    }

    pub fn modified_time_to_string(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.modified_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn access_time_to_string(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.access_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn creation_time_to_string(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.creation_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn size_to_string(&self) -> String {
        self.size.to_string()
    }

    pub fn to_line(&self, max_name_width: usize, max_modified_width: usize) -> Line<'_> {
        let name = expand_or_truncate(self.name.clone(), max_name_width);
        let modified = expand_or_truncate(self.modified_time_to_string(), max_modified_width);
        Line::from(format!("{}{}", name, modified))
    }
}

impl TryFrom<DirEntry> for File {
    type Error = std::io::Error;
    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let metadata = entry.metadata()?;

        Ok(Self {
            name: entry.file_name().to_string_lossy().to_string(),
            file_type: FileType::try_from(&entry)?,
            modified_time: metadata.modified()?,
            access_time: metadata.accessed()?,
            creation_time: metadata.created()?,
            size: metadata.len().into(),
        })
    }
}




#[derive(Debug, Copy, Clone)]
pub enum Size {
    Bytes(u16),
    KB(f32),
    MB(f32),
    GB(f32),
}

impl From<u64> for Size {
    fn from(size: u64) -> Self {
        if size < 1024 {
            Size::Bytes(size as u16)
        }
        else if size < 1024 * 1024 {
            Size::KB((size as f32) / 1024.0)
        }
        else if size < 1024 * 1024 * 1024 {
            Size::MB((size as f32) / 1024.0 / 1024.0)
        }
        else {
            Size::GB((size as f32) / 1024.0 / 1024.0 / 1024.0)
        }
    }
}

impl fmt::Display for Size{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Bytes(size) => write!(f, "{}B", size),
            Size::KB(size) => write!(f, "{:.2}KB", size),
            Size::MB(size) => write!(f, "{:.2}MB", size),
            Size::GB(size) => write!(f, "{:.2}GB", size),
        }
    }
}




#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Folder,
    Link { target: PathBuf, is_dead: bool },
    Unknown
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::File => write!(f, "File"),
            FileType::Folder => write!(f, "Folder"),
            FileType::Link { is_dead, .. } => {
                if *is_dead {
                    write!(f, "Link (Dead)")
                } else {
                    write!(f, "Link (Alive)")
                }
            },
            FileType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TryFrom<&DirEntry> for FileType {
    type Error = std::io::Error;
    fn try_from(entry: &DirEntry) -> Result<Self, Self::Error> {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            let metadata = fs::metadata(entry.path());
            Ok(FileType::Link { target: fs::read_link(entry.path())?, is_dead: metadata.is_err() })
        }
        else if file_type.is_file() {
            Ok(FileType::File)
        }
        else if file_type.is_dir() {
            Ok(FileType::Folder)
        }
        else {
            Ok(FileType::Unknown)
        }
    }
}