use std::path::Path;
use std::fs::{File, OpenOptions, Metadata};
use std::io::{Result, Read, Write, Seek, SeekFrom};
use walkdir::{self, DirEntry};
use crate::tools::to_io_err;

pub struct FileData {
    pub file: File,
    pub meta_data: Metadata,
    dir_entry: DirEntry,
}

impl FileData {
    pub fn path(&self) -> &Path { self.dir_entry.path() }

    pub fn open(dir_entry: DirEntry, meta_data: Metadata, read_only: bool) -> Result<FileData> {
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

pub trait ByteSized {
    fn byte_size(&self) -> usize;
}

impl ByteSized for FileData {
    fn byte_size(&self) -> usize { self.meta_data.len() as usize }
}

impl Read for FileData {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { self.file.read(buf) }
}

impl Write for FileData {
    fn write(&mut self, buf: &[u8]) -> Result<usize> { self.file.write(buf) }
    fn flush(&mut self) -> Result<()> { self.file.flush() }
}

impl Seek for FileData {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> { self.file.seek(pos) }
}

pub trait SizedRW: ByteSized + Read + Write + Seek {}
impl SizedRW for FileData {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Cursor;
    pub struct MockFileData(Cursor<Vec<u8>>);
    impl MockFileData {
        pub fn new(bytes: impl Into<Vec<u8>>) -> MockFileData {
            MockFileData(Cursor::new(bytes.into()))
        }
    }
    impl ByteSized for MockFileData {
        fn byte_size(&self) -> usize { self.0.get_ref().len() }
    }
    impl Seek for MockFileData {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> { self.0.seek(pos) }
    }
    impl Read for MockFileData {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> { self.0.read(buf) }
    }
    impl Write for MockFileData {
        fn write(&mut self, buf: &[u8]) -> Result<usize> { self.0.write(buf) }
        fn flush(&mut self) -> Result<()> { self.0.flush() }
    }
    impl SizedRW for MockFileData {}
}