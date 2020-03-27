use std::borrow::Cow::{Borrowed, Owned};
use std::io::{self, Error, ErrorKind};
use regex::Regex;
use crate::opts::Opts;
#[derive(Debug)]
pub struct Replacer<'a> {
    pattern: Regex,
    replacement: &'a str,
}

impl Opts {
    pub fn get_replacer(&self) -> io::Result<Replacer> {
        Replacer::new(&self.pattern, &self.replacement)
    }
}

impl<'a> Replacer<'a> {
    fn new(pattern: &'_ str, replacement: &'a str) -> io::Result<Replacer<'a>> {
        let re = Regex::new(pattern).map_err(|regex_err| {
            Error::new(ErrorKind::Other, regex_err.to_string())
        })?;

        Ok(Replacer {
            pattern: re,
            replacement: replacement,
        })
    }

    pub fn replace(&self, s: &'a str) -> Option<String> {
        match self.pattern.replace_all(&s, self.replacement) {
            Borrowed(_) => None,
            Owned(new_s) => Some(new_s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let og = "Who lives in an Easter-Island Head under the sea?\nSpongebob Tentacles!";
        let replaced = r.replace(og)
            .expect("'Spongebob' should've been replaced with 'Squidward'");
        assert_eq!(
            replaced,
            "Who lives in an Easter-Island Head under the sea?\nSquidward Tentacles!".to_string()
        );
    }

    #[test]
    fn returns_none_when_no_replacement_can_be_made() {
        let r = Replacer::new("capicola", "gabagool")
            .expect("What's wrong with 'capicola'?");
        let og = "The best part of The Sopranos is the gabagool!";
        assert!(r.replace(og).is_none());
    }

    #[test]
    fn replaces_multiline_patterns() {
        let wet_code = "\
foo()
bar()
baz()

// some other stuff

foo()
bar()
gabagool()\
        ";
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
        let dryed_code = r.replace(wet_code).expect("Unable to dedup");
        assert_eq!(dryed_code, expected_dry_code);
    }
}