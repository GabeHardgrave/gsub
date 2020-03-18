use std::io::{Error, ErrorKind};
use structopt::StructOpt;
use std::path::PathBuf;
use regex::Regex;

static DEFAULT_FILE_SIZE: &str = "4194304";

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

    /// Skip files larger than the given number of bytes.
    #[structopt(short = "m", long = "skip-files-larger-than", default_value = DEFAULT_FILE_SIZE)]
    pub max_file_size: usize,
}

impl Opts {
    pub fn parse_regex_from_pattern(&self) -> std::io::Result<Regex> {
        let re = Regex::new(&self.pattern).map_err(|regex_err| {
            Error::new(ErrorKind::Other, regex_err.to_string())
        })?;

        Ok(re)
    }
}
