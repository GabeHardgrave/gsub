use std::io;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions, Metadata};
use walkdir::{self, WalkDir, DirEntry};
use crate::opts::Opts;

pub struct FileData {
    pub file: File,
    pub meta_data: Metadata,
    dir_entry: DirEntry,
}

impl FileData {
    pub fn path(&self) -> &Path {
        self.dir_entry.path()
    }
}

type DynFileIter = Box<dyn Iterator<Item=walkdir::Result<DirEntry>>>;

struct FileIterConfig {
    max_file_size: u64,
    dir_entries: DynFileIter,
    read_only: bool,
}

impl FileIterConfig {
    fn new(max_file_size: usize, paths: Vec<PathBuf>, read_only: bool) -> FileIterConfig {
        let dir_entries: DynFileIter = if paths.is_empty() {
            Box::new(WalkDir::new(".").into_iter())
        } else {
            Box::new(paths.into_iter().map(WalkDir::new).flat_map(WalkDir::into_iter))
        };

        FileIterConfig {
            max_file_size: max_file_size as u64,
            dir_entries: dir_entries,
            read_only: read_only
        }
    }

    fn files(self) -> impl Iterator<Item=FileData> {
        let max_file_size = self.max_file_size;
        let read_only = self.read_only;

        self.dir_entries
            .filter_map(pluck_dir_entry_and_metadata)
            .filter(move |(_entry, meta_data)| meta_data.len() <= max_file_size)
            .filter(|(_entry, meta_data)| meta_data.is_file())
            .filter_map(move |(entry, meta_data)| {
                open_file(&entry, read_only)
                    .map(|f| (entry, meta_data, f))
                    .ok()
            })
            .map(|(entry, meta_data, file)| FileData {
                file: file,
                meta_data: meta_data,
                dir_entry: entry,
            })
    }
}

impl Opts {
    pub fn files(&self) -> impl Iterator<Item=FileData> {
        self.file_iter_config().files()
    }

    fn file_iter_config(&self) -> FileIterConfig {
        // Getting `FileIterConfig::new` to accept a reference proved fairly challenging.
        // Since only one `Opts` struct will exist, and since `.files` is a user defined list,
        // I'm really not worried about the runtime penalty of cloning
        let paths = self.files.clone();
        FileIterConfig::new(self.max_file_size, paths, self.dry_run)
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

fn open_file(dir_entry: &DirEntry, read_only: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(!read_only)
        .append(false)
        .create(false)
        .open(dir_entry.path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};

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
            vec!["test-files/file_iterator_tests".into()],
            true,
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
            vec![
                "test-files/big_enough.txt".into(),
                "test-files/too_big.txt".into(),
            ],
            true,
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

    #[test]
    fn only_opens_files_with_correct_permissions() {
        fs::File::create("test-files/no-touching").expect("unable to create file");
        let mut f = FileIterConfig::new(
            100,
            vec!["test-files/no-touching".into()],
            true,
        ).files().map(|fd| fd.file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_err(), "{:?}", attempt);
        fs::remove_file("test-files/no-touching").expect("unable to clean up test");

        fs::File::create("test-files/ok-touching").expect("unable to create file");
        let mut f = FileIterConfig::new(
            100,
            vec!["test-files/ok-touching".into()],
            false,
        ).files().map(|fd| fd.file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_ok(), "{:?}", attempt);
        fs::remove_file("test-files/ok-touching").expect("unable to clean up test");
    }
}