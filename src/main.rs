use std::io;
use std::error::Error;
use regex::RegexSet;
use ignore::{self, DirEntry, WalkState};
use gsub::gsub::gsub;
use gsub::opts::Opts;
use gsub::file_data::OpenFileData;

fn io_err<E>(e: E) -> io::Error
    where E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, e)
}

fn get_allowed_file_entry(
    entry_result: Result<DirEntry, ignore::Error>,
    blacklist: &RegexSet,
) -> Result<DirEntry, WalkState>
{
    let entry = entry_result.map_err(|_| WalkState::Continue)?;
    let file_type = entry.file_type().ok_or(WalkState::Continue)?;
    debug_assert!(
        !file_type.is_symlink(),
        "walk_builder() should've been configured to protect against symlinks"
    );

    let blacklisted = blacklist.is_match(&entry.file_name().to_string_lossy());
    let is_file = file_type.is_file();
    match (is_file, blacklisted) {
        (true, false) => Ok(entry),
        (true, true) => Err(WalkState::Continue),
        (false, false) => Err(WalkState::Continue),
        (false, true) => Err(WalkState::Skip),
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse().map_err(io_err)?;
    let replacer = opts.replacer().map_err(io_err)?;
    let blacklist = opts.dir_entry_blacklist().map_err(io_err)?;
    let opener = opts.open_opts();
    let presenter = opts.presenter();
    let walker = opts.walk_builder().build_parallel();

    walker.run(|| {
        Box::new(|result| {
            let entry = match get_allowed_file_entry(result, &blacklist) {
                Ok(e) => e,
                Err(walk_state) => return walk_state,
            };
            match gsub(opener.open_fd(entry), &replacer, &opts) {
                Ok(Some(msg)) | Err(msg) => presenter.wax(msg),
                Ok(None) => {},
            }
            WalkState::Continue
        })
    });

    Ok(())
}