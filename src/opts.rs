use std::io::{Error, ErrorKind};
use structopt::StructOpt;
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug, StructOpt)]
#[structopt(name = "gsub", about = "Bulk substitutions on a given file")]
pub struct Opts {
    #[structopt(short, long)]
    pub dry_run: bool,

    /// The pattern you want to replace
    pub pattern: String,

    /// String for replacement
    pub replacement: String,

    /// The file you want to make substitutions on
    #[structopt(parse(from_os_str))]
    pub file: PathBuf,
}

impl Opts {
    pub fn parse_regex_from_pattern(&self) -> std::io::Result<Regex> {
        let re = Regex::new(&self.pattern).map_err(|regex_err| {
            Error::new(ErrorKind::Other, regex_err.to_string())
        })?;

        Ok(re)
    }
}
