use std::io::{Error, ErrorKind};
use regex::Regex;
use crate::opts::Opts;

impl Opts {
    pub fn parse_regex_from_pattern(&self) -> std::io::Result<Regex> {
        let re = Regex::new(&self.pattern).map_err(|regex_err| {
            Error::new(ErrorKind::Other, regex_err.to_string())
        })?;

        Ok(re)
    }
}
