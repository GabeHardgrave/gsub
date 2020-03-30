use std::io;
use std::path::Path;
use std::fs::{File, OpenOptions, Metadata};
use walkdir::{self, DirEntry};
use crate::tools::to_io_err;

pub struct FileData {
    pub file: File,
    pub meta_data: Metadata,
    dir_entry: DirEntry,
}

impl FileData {
    pub fn path(&self) -> &Path { self.dir_entry.path() }

    pub fn open(dir_entry: DirEntry, meta_data: Metadata, read_only: bool) -> io::Result<FileData> {
        let file = OpenOptions::new()
            .read(true)
            .write(!read_only)
            .append(false)
            .create(false)
            .open(dir_entry.path())
            .map_err(|io_err| {
                format!("Failed to open {}: {}", dir_entry.path().to_string_lossy(), io_err)
            })
            .map_err(to_io_err)?;

        Ok(Self {
            file: file,
            dir_entry: dir_entry,
            meta_data: meta_data,
        })
    }
}