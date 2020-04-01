use std::io;
use regex::Regex;
use crate::file_data::SizedReader;
use crate::tools::{to_io_err, ToStringOption};

#[derive(Debug)]
pub struct Replacer<'a> {
    pattern: Regex,
    replacement: &'a str,
    buffer: String,
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

    pub fn replace(&mut self, fd: &mut impl SizedReader) -> io::Result<Option<String>> {
        self.prep_buffer_for_new_file(fd);
        fd.read_to_string(&mut self.buffer)?;
        Ok(self.pattern.replace_all(&self.buffer, self.replacement).to_option())
    }

    fn prep_buffer_for_new_file(&mut self, fd: & impl SizedReader) {
        self.buffer.clear();
        if fd.byte_size() > self.buffer.capacity() {
            self.buffer.reserve(fd.byte_size() as usize - self.buffer.capacity());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_data::tests::MockFileData;

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
        let replaced = r.replace(&mut file)
            .expect("'Spongebob' should've been replaced with 'Squidward'")
            .unwrap();
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
        assert!(r.replace(&mut file).unwrap().is_none());
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
        let dryed_code = r.replace(&mut wet_code).expect("Unable to dedup");
        assert_eq!(dryed_code.unwrap(), expected_dry_code);
    }

    #[test]
    fn replaces_multiple_files_in_a_row_correctly() {
        let mut f1 = MockFileData::new("capicola isn't vegan");
        let mut f2 = MockFileData::new("capicola is gluten free");
        let mut r = Replacer::new(
            "capicola",
            "gabagool"
        ).expect("What's wrong with gabagool");
        let f1_new = r.replace(&mut f1).expect("shoulda replaced capicola");
        let f2_new = r.replace(&mut f2).expect("shoulda replaced capicola");
        assert_eq!(f1_new.unwrap(), "gabagool isn't vegan");
        assert_eq!(f2_new.unwrap(), "gabagool is gluten free");
    }
}