use structopt::StructOpt;
use std::path::PathBuf;

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

    /// Skip files larger than the given number of bytes.
    #[structopt(short = "m", long = "skip-files-larger-than", default_value = DEFAULT_FILE_SIZE)]
    pub max_file_size: usize,

    /// List of files/directories you want to gsub on
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>
}