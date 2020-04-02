use std::io;
use regex::{self, Regex};
use crate::tools::ToStringOption;
use crate::file_data::SizedReader;

#[derive(Debug)]
pub struct Replacer<'a> {
    pattern: Regex,
    replacement: &'a str,
}

impl<'a> Replacer<'a> {
    pub fn new(pattern: &'_ str, replacement: &'a str) -> Result<Replacer<'a>, regex::Error> {
        let pattern = Regex::new(pattern)?;
        Ok(Replacer { pattern, replacement, })
    }

    pub fn replace(&self, fd: &mut impl SizedReader) -> io::Result<Option<String>> {
        let mut buffer = String::with_capacity(fd.byte_size());
        fd.read_to_string(&mut buffer)?;
        Ok(self.pattern.replace_all(&buffer, self.replacement).to_option())
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
        let r = Replacer::new("Spongebob", "Squidward")
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
        let r = Replacer::new("capicola", "gabagool")
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
        let r = Replacer::new(
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
        let r = Replacer::new(
            "capicola",
            "gabagool"
        ).expect("What's wrong with gabagool");
        let f1_new = r.replace(&mut f1).expect("shoulda replaced capicola");
        let f2_new = r.replace(&mut f2).expect("shoulda replaced capicola");
        assert_eq!(f1_new.unwrap(), "gabagool isn't vegan");
        assert_eq!(f2_new.unwrap(), "gabagool is gluten free");
    }
}