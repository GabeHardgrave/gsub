use std::fs::File;
use std::fs::OpenOptions;
use crate::opts::Opts;
use walkdir::{WalkDir, DirEntry};

pub struct FileIter {
    files: WalkDir,
}

impl Opts {
    pub fn file_iter(&self) -> FileIter {
        FileIter { files: WalkDir::new(&self.file) }
    }
}

fn is_file(dir_entry: &DirEntry) -> bool {
    match dir_entry.metadata() {
        Ok(meta_data) => meta_data.is_file(),
        _ => false,
    }
}

fn open_file(dir_entry: DirEntry) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open(dir_entry.path())
}

impl FileIter {
    pub fn each_file(self) -> impl Iterator<Item = std::io::Result<File>> {
        self.files
            .into_iter()
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .filter(is_file)
            .map(open_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_iter_skips_directories() {
        fs::create_dir_all("test-files/file_iter_tests/alpha/one")
            .expect("unable to create directory");
        fs::create_dir_all("test-files/file_iter_tests/alpha/two")
            .expect("unable to create directory");
        fs::create_dir_all("test-files/file_iter_tests/beta/one")
            .expect("unable to create directory");
        fs::create_dir_all("test-files/file_iter_tests/beta/two")
            .expect("unable to create directory");
        fs::File::create("test-files/file_iter_tests/one").expect("unable to create file");
        fs::File::create("test-files/file_iter_tests/two").expect("unable to create file");

        let file_iter = FileIter { files: WalkDir::new("test-files/file_iter_tests") };
        for file in file_iter.each_file().map(Result::unwrap) {
            let md = file.metadata().unwrap();
            assert!(md.is_file(), "{:?} was not a file", md)
        }
    }
}
