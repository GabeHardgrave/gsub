use std::io;
use std::path::PathBuf;
use std::fs::{File, OpenOptions, Metadata};
use walkdir::{self, WalkDir, DirEntry};
use crate::opts::Opts;

pub struct FileData {
    pub file: File,
    pub meta_data: Metadata,
}

struct FileIterConfig {
    max_file_size: u64,
    files_to_search: Vec<WalkDir>,
}

impl FileIterConfig {
    fn new(max_file_size: usize, files_to_search: &[PathBuf]) -> FileIterConfig {
        let mut files: Vec<WalkDir> = files_to_search.iter().map(WalkDir::new).collect();
        if files.is_empty() {
            files.push(WalkDir::new("."))
        }
        FileIterConfig {
            max_file_size: max_file_size as u64,
            files_to_search: files,
        }
    }

    fn files(self) -> impl Iterator<Item=FileData> {
        let max_file_size = self.max_file_size;
        self.files_to_search
            .into_iter()
            .flatten()
            .filter_map(pluck_dir_entry_and_metadata)
            .filter(move |(_entry, meta_data)| meta_data.len() <= max_file_size)
            .filter(|(_entry, meta_data)| meta_data.is_file())
            .filter_map(|(entry, meta_data)| {
                open_file(entry)
                    .map(|f| (f, meta_data))
                    .ok()
            })
            .map(|(file, meta_data)| FileData {
                file: file,
                meta_data: meta_data,
            })
    }
}

impl Opts {
    pub fn files(&self) -> impl Iterator<Item=FileData> {
        self.file_iter_config().files()
    }

    fn file_iter_config(&self) -> FileIterConfig {
        FileIterConfig::new(self.max_file_size, &self.files)
    }
}

fn pluck_dir_entry_and_metadata(
    dir_entry: walkdir::Result<DirEntry>,
) -> Option<(DirEntry, Metadata)>
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn skips_directories() {
        let dirs = [
            "test-files/file_iterator_tests/alpha",
            "test-files/file_iterator_tests/beta",
        ];
        dirs.iter().for_each(|d| {
            fs::create_dir_all(d).expect("unable to creat directory");
        });

        let files = [
            "test-files/file_iterator_tests/one",
            "test-files/file_iterator_tests/two",
            "test-files/file_iterator_tests/alpha/one",
            "test-files/file_iterator_tests/alpha/two",
            "test-files/file_iterator_tests/beta/one",
            "test-files/file_iterator_tests/beta/two",
        ];
        files.iter().for_each(|f| {
            fs::File::create(f).expect("unable to create file");
        });

        let fi = FileIterConfig::new(
            100,
            &["test-files/file_iterator_tests".into()]
        );
        let files_searched = fi.files().map(|f| {
            assert!(f.meta_data.is_file());
            f
        }).count();
        assert_eq!(files_searched, files.len());

        fs::remove_dir_all("test-files/file_iterator_tests")
            .expect("unable to clean up test");
    }

    #[test]
    fn skips_files_larger_than_max_size() {
        fs::write("test-files/big_enough.txt", b"0123456789")
            .expect("unable to create file");
        fs::write("test-files/too_big.txt",    b"0123456789A")
            .expect("unable to create file");

        let mut files: Vec<File> = FileIterConfig::new(
            10,
            &[
                "test-files/big_enough.txt".into(),
                "test-files/too_big.txt".into(),
            ]
        ).files().map(|fd| fd.file).collect();
        assert_eq!(files.len(), 1);

        let mut buff = Vec::new();
        files.pop().unwrap().read_to_end(&mut buff).expect("unable to check file");

        let s = String::from_utf8(buff).unwrap();
        assert_eq!("0123456789".to_string(), s);

        fs::remove_file("test-files/big_enough.txt")
            .expect("unable to clean up test");
        fs::remove_file("test-files/too_big.txt")
            .expect("unable to clean up test");
    }
}