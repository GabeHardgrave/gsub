use std::io;
use std::path::PathBuf;
use structopt::StructOpt;
use crate::file_iterator::FileIterConfig;
use crate::replacer::Replacer;
use crate::presenter::Presenter;

static DEFAULT_FILE_SIZE: &str = "4194304";

#[derive(Debug, StructOpt)]
#[structopt(name = "gsub", about = "Regex substitution for files and directories")]
pub struct Opts {
    #[structopt(short, long)]
    pub dry_run: bool,

    #[structopt(short, long)]
    pub verbose: bool,

    /// The pattern you want to replace
    pub pattern: String,

    /// String for replacement
    pub replacement: String,

    /// Skip files larger than the given number of bytes.
    #[structopt(short = "m", long = "skip-files-larger-than", default_value = DEFAULT_FILE_SIZE)]
    pub max_file_size: usize,

    /// Files/Directories to skip
    #[structopt(short = "e", long = "except")]
    pub files_to_skip: Vec<String>,

    /// Do not skip hidden files and directories
    #[structopt(short = "h", long = "hidden")]
    pub show_hidden_files: bool,

    /// List of files/directories you want to gsub on. If unspecified, uses the current directory.
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>,
}

impl Opts {
    pub fn parse() -> Self {
        let mut opts = Self::from_args();
        if opts.files.is_empty() {
            opts.files.push(".".into()) // current directory
        }
        opts
    }

    pub fn replacer(&self) -> io::Result<Replacer> {
        Replacer::new(&self.pattern, &self.replacement)
    }

    pub fn presenter(&self) -> Presenter {
        Presenter::new(self.verbose)
    }

    pub fn file_iter_config(&self) -> io::Result<FileIterConfig> {
        FileIterConfig::new(&self.files)
            .read_only(self.dry_run)
            .skip_files_larger_than(self.max_file_size)
            .skip_hidden_files(!self.show_hidden_files)
            .skip_files_that_match(&self.files_to_skip)
            .map_err(|regex_err| {
                io::Error::new(io::ErrorKind::Other, regex_err.to_string())
            })
    }
}