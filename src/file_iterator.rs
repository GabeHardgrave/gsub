use std::io;
use std::path::{Path};
use std::fs::{File, OpenOptions, Metadata};
use regex::{self, RegexSet};
use walkdir::{self, WalkDir, DirEntry};

pub struct FileData {
    pub file: File,
    pub meta_data: Metadata,
    dir_entry: DirEntry,
}

impl FileData {
    pub fn path(&self) -> &Path { self.dir_entry.path() }

    fn open(dir_entry: DirEntry, meta_data: Metadata, read_only: bool) -> io::Result<FileData> {
        let file = OpenOptions::new()
            .read(true)
            .write(!read_only)
            .append(false)
            .create(false)
            .open(dir_entry.path())?;

        Ok(Self {
            file: file,
            dir_entry: dir_entry,
            meta_data: meta_data,
        })
    }
}

pub struct FileIterConfig {
    max_file_size: u64,
    read_only: bool,
    paths: Vec<WalkDir>,
    blacklist: RegexSet,
}

impl FileIterConfig {
    const NO_PATHS: [&'static str; 0] = [];
    const DEFAULT_MAX_FILE_SIZE: u64 = 4_194_304;

    pub fn new<P, I>(paths: I) -> Self
        where P: AsRef<Path>,
              I: IntoIterator<Item = P>
    {
        Self {
            paths: paths.into_iter().map(WalkDir::new).collect(),
            read_only: true,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            blacklist: RegexSet::new(&Self::NO_PATHS).unwrap(),
        }
    }

    pub fn skip_files_larger_than(self, size: usize) -> Self {
        Self { max_file_size: size as u64, ..self }
    }

    pub fn read_only(self, read_only: bool) -> Self {
        Self { read_only: read_only, ..self }
    }

    pub fn skip_files_that_match<S, I>(self, patterns: I) -> Result<Self, regex::Error>
        where S: AsRef<str>,
              I: IntoIterator<Item = S>
    {
        let paths_to_skip = RegexSet::new(patterns)?;
        Ok(Self { blacklist: paths_to_skip, ..self })
    }
}

impl IntoIterator for FileIterConfig {
    type Item = FileData;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let paths = self.paths;
        let blacklist = self.blacklist;
        let max_file_size = self.max_file_size;
        let read_only = self.read_only;

        let not_blacklisted = move |dir_entry: &DirEntry| {
            !blacklist.is_match(&dir_entry.path().to_string_lossy())
        };

        let pluck_meta_data = |dir_entry: DirEntry| {
            dir_entry.metadata().map(|meta_data| (dir_entry, meta_data)).ok()
        };

        Box::new(paths
            .into_iter()
            .flat_map(WalkDir::into_iter)
            .filter_map(Result::ok)
            .filter(not_blacklisted)
            .filter_map(pluck_meta_data)
            .filter(move |(_entry, meta_data)| meta_data.len() <= max_file_size)
            .filter(|(_entry, meta_data)| meta_data.is_file())
            .filter_map(move |(entry, meta_data)|
                FileData::open(entry, meta_data, read_only).ok()
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

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

        let fi = FileIterConfig::new(&["test-files/file_iterator_tests"]);
        let files_searched = fi.into_iter().map(|f| {
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

        let fi = FileIterConfig::new(&[
            "test-files/big_enough.txt",
            "test-files/too_big.txt",
        ]).skip_files_larger_than(10);

        let mut files: Vec<FileData> = fi.into_iter().collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files.pop().unwrap().path().to_string_lossy(), "test-files/big_enough.txt");

        fs::remove_file("test-files/big_enough.txt")
            .expect("unable to clean up test");
        fs::remove_file("test-files/too_big.txt")
            .expect("unable to clean up test");
    }

    #[test]
    fn opens_files_in_read_only_mode_when_specified() {
        fs::File::create("test-files/no-touching").expect("unable to create file");

        let fi = FileIterConfig::new(&["test-files/no-touching"])
            .read_only(true);
        let mut f = fi.into_iter()
            .map(|fd| fd.file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_err(), "{:?}", attempt);

        fs::remove_file("test-files/no-touching").expect("unable to clean up test");
    }

    #[test]
    fn opens_files_in_write_mode_when_specified() {
        fs::File::create("test-files/ok-touching").expect("unable to create file");

        let fi = FileIterConfig::new(&["test-files/ok-touching"])
            .read_only(false);
        let mut f = fi.into_iter().map(|fd| fd.file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_ok(), "{:?}", attempt);

        fs::remove_file("test-files/ok-touching").expect("unable to clean up test");
    }
}