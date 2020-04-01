use std::io;
use std::path::PathBuf;
use structopt::StructOpt;
use crate::file_iterator::FileIterConfig;
use crate::replacer::Replacer;
use crate::presenter::Presenter;
use crate::tools::io_err;
use crate::{DEFAULT_FILE_SIZE_STR, CURRENT_DIR};

#[derive(Debug, StructOpt)]
#[structopt(name = "gsub", about = "Regex substitution for files and directories")]
pub struct Opts {
    #[structopt(short, long)]
    pub dry_run: bool,

    /// Copies files instead of editing them
    #[structopt(short, long)]
    pub copy_on_write: bool,

    #[structopt(short, long)]
    pub verbose: bool,

    /// The pattern you want to replace
    pub pattern: String,

    /// String for replacement
    pub replacement: String,

    /// Skip files larger than the given number of bytes.
    #[structopt(short = "m", long = "skip-files-larger-than", default_value = DEFAULT_FILE_SIZE_STR)]
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
    pub fn parse() -> io::Result<Self> {
        let mut opts = Self::from_args();
        if opts.files.is_empty() {
            opts.files.push(CURRENT_DIR.into())
        }
        if opts.copy_on_write && opts.dry_run {
            return Err(io_err("--dry-run and --copy-on-write are incompatible flags".to_string()));
        }
        Ok(opts)
    }

    pub fn replacer(&self) -> io::Result<Replacer> {
        Replacer::new(&self.pattern, &self.replacement)
    }

    pub fn presenter(&self) -> Presenter {
        Presenter::new(self.verbose)
    }

    pub fn file_iter_config(&self) -> io::Result<FileIterConfig> {
        FileIterConfig::new(&self.files)
            .read_only(self.dry_run || self.copy_on_write)
            .skip_files_larger_than(self.max_file_size)
            .skip_hidden_files(!self.show_hidden_files)
            .skip_files_that_match(&self.files_to_skip)
            .map_err(io_err)
    }
}