use std::time::SystemTime;
use std::fs::DirEntry;
use std::fs::FileType as StdFileType;
use std::fs::Metadata as StdMetadata;
use std::path::PathBuf;
use std::fs;
use std::fmt;
use chrono::{DateTime, Local};
#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub file_type: FileType,
    pub metadata: Metadata,
}

impl TryFrom<DirEntry> for File {
    type Error = std::io::Error;
    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {

        Ok(Self {
            name: entry.file_name().to_string_lossy().to_string(),
            file_type: FileType::try_from(&entry)?,
            metadata: Metadata::try_from(entry.metadata()?)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    modified_time: SystemTime,
    access_time: SystemTime,
    creation_time: SystemTime,
    size: Size,
}

impl Metadata {
    pub fn modified_time_to_string(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.modified_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn access_time_to_string(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.access_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn creation_time(&self) -> String {
        let datetime : DateTime<Local> = DateTime::from(self.creation_time);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn size_to_string(&self) -> String {
        self.size.to_string()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Size {
    Bytes(u64),
    KB(u64),
    MB(u64),
    GB(u64),
}

impl From<u64> for Size {
    fn from(size: u64) -> Self {
        if size < 1024 {
            Size::Bytes(size)
        }
        else if size < 1024 * 1024 {
            Size::KB(size / 1024)
        }
        else if size < 1024 * 1024 * 1024 {
            Size::MB(size / 1024 / 1024)
        }
        else {
            Size::GB(size / 1024 / 1024 / 1024)
        }
    }
}

impl ToString for Size{
    fn to_string(&self) -> String{
        match self {
            Size::Bytes(size) => format!("{}B", size),
            Size::KB(size) => format!("{}KB", size),
            Size::MB(size) => format!("{} MB", size),
            Size::GB(size) => format!("{} GB", size),
        }
    }
}

impl TryFrom<StdMetadata> for Metadata {
    type Error = std::io::Error;
    fn try_from(metadata: StdMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            modified_time: metadata.modified()?,
            access_time: metadata.accessed()?,
            creation_time: metadata.created()?,
            size: metadata.len().into(),
        })
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