use std::time::SystemTime;
use std::fs::DirEntry;
use std::fs::FileType as StdFileType;
use std::fs::Metadata as StdMetadata;
use std::path::PathBuf;
use std::fs;
use std::fmt;
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
}

impl TryFrom<StdMetadata> for Metadata {
    type Error = std::io::Error;
    fn try_from(metadata: StdMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            modified_time: metadata.modified()?,
            access_time: metadata.accessed()?,
            creation_time: metadata.created()?,
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