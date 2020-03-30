use std::io;
use std::borrow::Cow::{Borrowed, Owned};
use regex::Regex;
use crate::tools::to_io_err;
use crate::file_data::SizedRW;

#[derive(Debug)]
pub struct Replacer<'a> {
    pattern: Regex,
    replacement: &'a str,
    buffer: String,
}

pub enum GsubResult {
    NoChange,
    Replaced(String),
    Error(io::Error),
}

impl<'a> Replacer<'a> {
    pub fn new(pattern: &'_ str, replacement: &'a str) -> io::Result<Replacer<'a>> {
        let re = Regex::new(pattern)
            .map_err(|regex_err| { regex_err.to_string() })
            .map_err(to_io_err)?;

        Ok(Replacer {
            pattern: re,
            replacement: replacement,
            buffer: String::new(),
        })
    }

    pub fn old_contents(&self) -> &str { &self.buffer }

    pub fn gsub(&mut self, fd: &mut impl SizedRW) -> GsubResult {
        self.prep_buffer_for_new_file(fd);
        if let Err(e) = fd.read_to_string(&mut self.buffer) {
            return GsubResult::Error(e);
        }
        self.replace(&self.buffer)
    }

    fn prep_buffer_for_new_file(&mut self, fd: & impl SizedRW) {
        self.buffer.clear();
        if fd.byte_size() > self.buffer.capacity() {
            self.buffer.reserve(fd.byte_size() as usize - self.buffer.capacity());
        }
    }

    fn replace(&self, s: &'a str) -> GsubResult {
        match self.pattern.replace_all(&s, self.replacement) {
            Borrowed(_) => GsubResult::NoChange,
            Owned(new_s) => GsubResult::Replaced(new_s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_data::tests::MockFileData;

    impl GsubResult {
        fn expect(self, msg: &'static str) -> String {
            match self {
                Self::Replaced(s) => s,
                _ => panic!(msg)
            }
        }

        fn no_change(&self) -> bool {
            match self {
                Self::NoChange => true,
                _ => false,
            }
        }
    }

    #[test]
    fn initializer_rejects_bad_patterns() {
        let r = Replacer::new("*", "irrelevant");
        assert!(r.is_err(), "'*' did not generate a regex error");
        let expected_err_msg = "\
regex parse error:
    *
    ^
error: repetition operator missing expression";
        assert_eq!(
            r.unwrap_err().to_string(),
            expected_err_msg,
        );
    }

    #[test]
    fn replaces_simple_words() {
        let mut r = Replacer::new("Spongebob", "Squidward")
            .expect("a simple word like 'Spongebob' is getting rejected by Regex");
        let mut file = MockFileData::new(
            "Who lives in an Easter-Island Head under the sea?\nSpongebob Tentacles!"
        );
        let replaced = r.gsub(&mut file)
            .expect("'Spongebob' should've been replaced with 'Squidward'");
        assert_eq!(
            replaced,
            "Who lives in an Easter-Island Head under the sea?\nSquidward Tentacles!".to_string()
        );
    }

    #[test]
    fn returns_no_change_when_no_replacement_can_be_made() {
        let mut r = Replacer::new("capicola", "gabagool")
            .expect("What's wrong with 'capicola'?");
        let mut file = MockFileData::new(
            "The best part of The Sopranos is the gabagool!"
        );
        assert!(r.gsub(&mut file).no_change());
    }

    #[test]
    fn replaces_multiline_patterns() {
        let mut wet_code = MockFileData::new("\
foo()
bar()
baz()

// some other stuff

foo()
bar()
gabagool()\
        ");
        let mut r = Replacer::new(
            r"foo\(\)\nbar\(\)",
            "foo_and_bar()"
        ).expect("What's wrong with a multiline replacement?");
        let expected_dry_code = "\
foo_and_bar()
baz()

// some other stuff

foo_and_bar()
gabagool()\
        ".to_string();
        let dryed_code = r.gsub(&mut wet_code).expect("Unable to dedup");
        assert_eq!(dryed_code, expected_dry_code);
    }
}