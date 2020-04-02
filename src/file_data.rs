use std::path::Path;
use std::borrow::Cow;
use std::fs::{File, OpenOptions};
use std::io::{Result, Read, Write, Seek, SeekFrom};
use ignore::{self, DirEntry};

pub struct FileData {
    file: File,
    estimated_size: u64,
    dir_entry: DirEntry,
}

pub trait OpenFileData {
    fn open_fd(&self, entry: DirEntry) -> Result<FileData>;
}

impl OpenFileData for OpenOptions {
    fn open_fd(&self, dir_entry: DirEntry) -> Result<FileData> {
        debug_assert!(dir_entry.file_type().unwrap().is_file());
        let estimated_size = dir_entry.metadata()
            .map(|md| md.len())
            .unwrap_or(0);
        let file = self.open(dir_entry.path())?;
        Ok(FileData { file, estimated_size, dir_entry})
    }
}

impl FileData {
    pub fn path(&self) -> &Path { self.dir_entry.path() }
    pub fn path_str(&self) -> Cow<'_, str> { self.path().to_string_lossy() }
}

// It's kind of tedious to define all of these traits, but it does let us unit test really well
// so \_(*_*)_/

pub trait ByteSized {
    fn byte_size(&self) -> usize;
}

impl ByteSized for FileData {
    fn byte_size(&self) -> usize { self.estimated_size as usize }
}

impl Read for FileData {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { self.file.read(buf) }
}

pub trait SizedReader: ByteSized + Read {}
impl SizedReader for FileData {}

pub trait Truncable {
    fn truncate(&mut self, to: usize) -> Result<()>;
}

impl Truncable for FileData {
    fn truncate(&mut self, to: usize) -> Result<()> {
        self.file.set_len(to as u64)
    }
}

impl Write for FileData {
    fn write(&mut self, buf: &[u8]) -> Result<usize> { self.file.write(buf) }
    fn flush(&mut self) -> Result<()> { self.file.flush() }
}

impl Seek for FileData {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> { self.file.seek(pos) }
}

pub trait OverWrite: Seek + Write + Truncable {
    fn overwrite(&mut self, contents: &[u8]) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        self.write_all(contents)?;
        self.flush()?;
        self.truncate(contents.len())
    }
}
impl OverWrite for FileData {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Cursor;
    use std::str::from_utf8;
    pub struct MockFileData(Cursor<Vec<u8>>);
    impl MockFileData {
        pub fn new(bytes: impl Into<Vec<u8>>) -> MockFileData {
            MockFileData(Cursor::new(bytes.into()))
        }
    }

    impl ByteSized for MockFileData {
        fn byte_size(&self) -> usize { self.0.get_ref().len() }
    }
    impl Read for MockFileData {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> { self.0.read(buf) }
    }
    impl SizedReader for MockFileData {}

    impl Seek for MockFileData {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> { self.0.seek(pos) }
    }
    impl Write for MockFileData {
        fn write(&mut self, buf: &[u8]) -> Result<usize> { self.0.write(buf) }
        fn flush(&mut self) -> Result<()> { self.0.flush() }
    }
    impl Truncable for MockFileData {
        fn truncate(&mut self, to: usize) -> Result<()> {
            self.0.get_mut().truncate(to);
            Ok(())
        }
    }
    impl OverWrite for MockFileData {}

    #[test]
    fn overwrites_the_entire_file_for_larger_diffs() {
        let mut file = MockFileData::new("oat milk is tasty");
        file.write("almond".as_bytes()).expect("WTF?");
        file.overwrite("soy milk is superb".as_bytes()).expect("WTF?");
        assert_eq!(
            from_utf8(file.0.get_ref()).unwrap(),
            "soy milk is superb"
        );
    }

    #[test]
    fn overwrites_the_entire_file_for_smaller_diffs() {
        let mut file = MockFileData::new("oat milk is the fucking bomb");
        file.overwrite("soy milk is the bomb".as_bytes()).expect("WTF");
        assert_eq!(
            from_utf8(file.0.get_ref()).unwrap(),
            "soy milk is the bomb"
        );
    }
}