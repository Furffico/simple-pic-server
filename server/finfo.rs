use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

use crate::static_var::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub is_file: bool,
    pub is_folder: bool,
    pub is_symlink: bool,
    pub modified: Option<f64>,
    pub accessed: Option<f64>,
    pub created: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fileinfo {
    pub name: String,
    pub urlpath: String,
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub name: String,
    pub urlpath: String,
    pub children: Vec<Fileinfo>,
}

fn conv_systemtime_to_f64<E>(syst: Result<SystemTime, E>) -> Option<f64> {
    match syst {
        Ok(v) => v
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(None, |w| Some(w.as_secs_f64())),
        Err(_) => None,
    }
}

impl From<std::fs::Metadata> for FileMetadata {
    fn from(mtd: std::fs::Metadata) -> Self {
        Self {
            is_file: mtd.is_file(),
            is_folder: mtd.is_dir(),
            is_symlink: mtd.is_symlink(),
            modified: conv_systemtime_to_f64(mtd.modified()),
            accessed: conv_systemtime_to_f64(mtd.accessed()),
            created: conv_systemtime_to_f64(mtd.created()),
        }
    }
}

impl From<std::fs::DirEntry> for Fileinfo {
    fn from(entry: std::fs::DirEntry) -> Self {
        let metadata: Option<FileMetadata> = match entry.metadata() {
            Ok(v) => Some(v.into()),
            Err(_) => None,
        };
        Self {
            name: entry.file_name().to_str().unwrap().to_string(),
            urlpath: entry
                .path()
                .to_str()
                .unwrap()
                .strip_prefix(".")
                .unwrap_or_default()
                .to_string(),
            metadata: metadata,
        }
    }
}

impl FolderInfo {
    pub fn new(path: &Path) -> Result<Self> {
        let readdir = std::fs::read_dir(path)?;

        let hide_dotfile: bool = CONFIG.get_bool("hide_dotfile").unwrap();
        let files: Vec<Fileinfo> = readdir
            .filter_map(|entry| -> Option<Fileinfo> {
                match entry {
                    Ok(entry) => {
                        let hidden = hide_dotfile
                            && entry
                                .file_name()
                                .to_str()
                                .unwrap_or_default()
                                .starts_with(".");
                        if hidden {
                            None
                        } else {
                            Some(entry.into())
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect();
        Ok(Self {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string(),
            urlpath: path
                .to_str()
                .unwrap()
                .strip_prefix(".")
                .unwrap_or_default()
                .to_string(),
            children: files,
        })
    }
}
