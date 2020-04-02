use std::{io, fs};
use crate::opts::Opts;
use crate::replacer::Replacer;
use crate::tools::add_gsub_ext;
use crate::presenter::{Msg, ToMsg};
use crate::file_data::{FileData, OverWrite};

pub fn gsub(
    fd_result: io::Result<FileData>,
    replacer: &Replacer,
    opts: &Opts,
) -> Result<Option<Msg<String>>, Msg<String>>
{
    let mut fd = fd_result.map_err(ToMsg::verbose)?;
    let replacement = replacer.replace(&mut fd).map_err(|e| {
        format!("Skipping {} because {}", fd.path_str(), e).verbose()
    })?;
    let new_contents = match replacement {
        Some(s) => s,
        None => return Ok(None),
    };
    let success_msg = if opts.dry_run {
        format!("Would have updated {}", fd.path_str()).important()
    } else if opts.copy_on_write {
        let new_file_name = add_gsub_ext(fd.path());
        fs::write(&new_file_name, new_contents).map_err(ToMsg::important)?;
        format!("Created {}", new_file_name.to_string_lossy()).important()
    } else {
        fd.overwrite(&new_contents.as_bytes()).map_err(ToMsg::important)?;
        format!("Updated {}", fd.path_str()).important()
    };
    Ok(Some(success_msg))
}