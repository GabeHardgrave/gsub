use std::io;
use std::fs::{File, OpenOptions, Metadata};
use walkdir::{self, WalkDir, DirEntry};
use crate::opts::Opts;

pub struct FileIter {
    files: WalkDir,
}

impl Opts {
    pub fn file_iter(&self) -> FileIter {
        FileIter { files: WalkDir::new(&self.file) }
    }
}

fn pluck_dir_entry_and_metadata(
    dir_entry: walkdir::Result<DirEntry>,
) -> Option<(DirEntry, Metadata)> {
    let entry = dir_entry.ok()?;
    let meta_data = entry.metadata().ok()?;
    Some((entry, meta_data))
}

fn open_file(dir_entry: DirEntry) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open(dir_entry.path())
}

impl FileIter {
    pub fn each_file_with_size(self) -> impl Iterator<Item = (File, usize)> {
        self.files
            .into_iter()
            .filter_map(pluck_dir_entry_and_metadata)
            .filter(|(_entry, meta_data)| meta_data.is_file())
            .filter_map(|(entry, meta_data)| {
                open_file(entry).map(|f| (f, meta_data.len() as usize)).ok()
            })
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
        for (file, _size) in file_iter.each_file_with_size() {
            let md = file.metadata().unwrap();
            assert!(md.is_file(), "{:?} was not a file", md)
        }
    }
}
