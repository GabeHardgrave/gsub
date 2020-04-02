use std::path::PathBuf;
use std::fs::OpenOptions;
use structopt::StructOpt;
use ignore::WalkBuilder;
use regex::{self, RegexSet};
use crate::replacer::Replacer;
use crate::presenter::Presenter;
use crate::gsub::GSUB_EXT_PATTERN;
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
    pub max_file_size: u64,

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
    pub fn parse() -> Result<Self, &'static str> {
        let opts = Self::from_args();
        if opts.copy_on_write && opts.dry_run {
            return Err("--dry-run and --copy-on-write are incompatible flags");
        }
        Ok(opts)
    }

    pub fn replacer(&self) -> Result<Replacer, regex::Error> {
        Replacer::new(&self.pattern, &self.replacement)
    }

    pub fn presenter(&self) -> Presenter {
        Presenter::new(self.verbose)
    }

    pub fn open_opts(&self) -> OpenOptions {
        let read_only = self.copy_on_write || self.dry_run;
        let mut open_opts = OpenOptions::new();
        open_opts.read(true)
            .write(!read_only)
            .append(false)
            .create(false);
        open_opts
    }

    pub fn dir_entry_blacklist(&self) -> Result<RegexSet, regex::Error> {
        RegexSet::new([GSUB_EXT_PATTERN.to_string()]
            .iter()
            .chain(self.files_to_skip.iter())
        )
    }

    pub fn walk_builder(&self) -> WalkBuilder {
        let mut wb = self.base_walk_builder();
        wb.follow_links(false)
            .max_filesize(Some(self.max_file_size))
            .hidden(!self.show_hidden_files);
        wb
    }

    fn base_walk_builder(&self) -> WalkBuilder {
        let mut paths = self.files.iter();
        let mut wb = WalkBuilder::new(paths
            .next()
            .unwrap_or(&PathBuf::from(CURRENT_DIR)));
        paths.for_each(|p| { wb.add(p); });
        wb
    }
}