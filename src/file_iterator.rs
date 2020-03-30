use std::io;
use std::path::Path;
use std::fs::Metadata;
use regex::{self, RegexSet};
use walkdir::{self, WalkDir, DirEntry};
use crate::DEFAULT_FILE_SIZE_INT;
use crate::tools::is_hidden;
use crate::file_data::FileData;

pub struct FileIterConfig {
    max_file_size: u64,
    read_only: bool,
    skip_hidden_files: bool,
    paths: Vec<WalkDir>,
    blacklist: RegexSet,
}

impl FileIterConfig {
    const NO_PATHS: [&'static str; 0] = [];

    pub fn new<P, I>(paths: I) -> Self
        where P: AsRef<Path>,
              I: IntoIterator<Item = P>
    {
        Self {
            paths: paths.into_iter().map(WalkDir::new).collect(),
            read_only: true,
            skip_hidden_files: true,
            max_file_size: *DEFAULT_FILE_SIZE_INT,
            blacklist: RegexSet::new(&Self::NO_PATHS).unwrap(),
        }
    }

    pub fn skip_files_larger_than(self, size: usize) -> Self {
        Self { max_file_size: size as u64, ..self }
    }

    pub fn read_only(self, read_only: bool) -> Self {
        Self { read_only: read_only, ..self }
    }

    pub fn skip_hidden_files(self, skip_hidden_files: bool) -> Self {
        Self { skip_hidden_files: skip_hidden_files, ..self }
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
    type Item = io::Result<FileData>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut paths = self.paths;
        let blacklist = self.blacklist;
        let max_file_size = self.max_file_size;
        let read_only = self.read_only;
        let skip_hidden_files = self.skip_hidden_files;

        let not_blacklisted = move |entry: &DirEntry| {
            if skip_hidden_files && is_hidden(entry.file_name()) {
                return false;
            }
            !blacklist.is_match(&entry.path().to_string_lossy())
        };

        let pluck_meta_data = |entry: DirEntry| {
            entry.metadata().map(|meta_data| (entry, meta_data)).ok()
        };

        let open_small_file = move |(entry, meta_data): (DirEntry, Metadata)| {
            if !meta_data.is_file() || meta_data.len() > max_file_size {
                return None;
            }
            Some(FileData::open(entry, meta_data, read_only))
        };

        // This conditional is to support a non-trivial optimization. Namely, that
        // walkdir::FilterEntry is more efficient than the standard Iterator::filter method.
        // This is because FilterEntry skips recursing into directories that don't match the
        // filter, whereas the standard `filter` iterator adaptor still descends into a directory
        // that doesn't satisfy the predicate.
        //
        // I originally tried to use
        // ```
        // paths
        //     .into_iter()
        //     .flat_map(move |walk_dir| {
        //         walk_dir.into_iter().filter_entry(not_blacklisted)
        //     })
        //     .filter(/* the rest of the filter adaptors */)
        // ```
        //
        // Unfortunately, this led to compiler errors as `not_blacklisted` was being used after
        // move.
        //
        // I couldn't find a generic way around this compiler error, so this was the optimization
        // I decided on. By unwrapping the WalkDir struct when there is only one, we can use
        // .filter_entry over .filter without generating compiler errors.
        if paths.len() == 1 {
            let walk_dir = paths.pop().unwrap();
            Box::new(walk_dir
                .into_iter()
                .filter_entry(not_blacklisted)
                .filter_map(Result::ok)
                .filter_map(pluck_meta_data)
                .filter_map(open_small_file)
            )
        } else {
            Box::new(paths
                .into_iter()
                .flat_map(WalkDir::into_iter)
                .filter_map(Result::ok)
                .filter(not_blacklisted)
                .filter_map(pluck_meta_data)
                .filter_map(open_small_file)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn skips_hidden_files() {
        fs::create_dir_all("test-files/hidden-tests")
            .expect("unable to creat directory");
        let files = [
            "test-files/hidden-tests/.secret",
            "test-files/hidden-tests/public",
        ];
        files.iter().for_each(|f| {
            fs::File::create(f).expect("unable to create file");
        });

        let fi = FileIterConfig::new(&["test-files/hidden-tests"])
            .skip_hidden_files(true);
        let files_searched: Vec<String> = fi
            .into_iter()
            .map(Result::unwrap)
            .map(|fd| fd.path().to_string_lossy().to_string())
            .collect();
        assert_eq!(files_searched.len(), 1);
        assert_eq!(files_searched[0], "test-files/hidden-tests/public".to_string());

        fs::remove_dir_all("test-files/hidden-tests")
            .expect("unable to clean up test");
    }

    #[test]
    fn skips_files_in_blacklist() {
        fs::create_dir_all("test-files/blacklist-tests")
            .expect("unable to creat directory");
        let files = [
            "test-files/blacklist-tests/test.py",
            "test-files/blacklist-tests/test.rb",
            "test-files/blacklist-tests/test.rs",
        ];
        files.iter().for_each(|f| {
            fs::File::create(f).expect("unable to create file");
        });

        let fi = FileIterConfig::new(&["test-files/blacklist-tests"])
            .skip_files_that_match(&[r"(.*)\.py", r"(.*)\.rb"])
            .expect("failed to compile regex set");

        let files_searched: Vec<String> = fi
            .into_iter()
            .map(Result::unwrap)
            .map(|fd| fd.path().to_string_lossy().to_string())
            .collect();
        assert_eq!(files_searched.len(), 1);
        assert_eq!(files_searched[0], "test-files/blacklist-tests/test.rs".to_string());

        fs::remove_dir_all("test-files/blacklist-tests")
            .expect("unable to clean up test");
    }

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
        let files_searched = fi.into_iter().map(Result::unwrap).map(|f| {
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

        let mut files: Vec<FileData> = fi.into_iter().map(Result::unwrap).collect();
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
            .map(|fd| fd.unwrap().file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_err(), "{:?}", attempt);

        fs::remove_file("test-files/no-touching").expect("unable to clean up test");
    }

    #[test]
    fn opens_files_in_write_mode_when_specified() {
        fs::File::create("test-files/ok-touching").expect("unable to create file");

        let fi = FileIterConfig::new(&["test-files/ok-touching"])
            .read_only(false);
        let mut f = fi.into_iter().map(|fd| fd.unwrap().file).next().expect("didn't find the expected file");
        let attempt = f.write_all(b"I'm touching");
        assert!(attempt.is_ok(), "{:?}", attempt);

        fs::remove_file("test-files/ok-touching").expect("unable to clean up test");
    }
}