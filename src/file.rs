use std::time::SystemTime;
use std::fs::DirEntry;
use std::fs::FileType as StdFileType;
use std::fs::Metadata as StdMetadata;
use std::path::PathBuf;
use std::fs;
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