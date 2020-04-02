use std::{io, fs};
use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};
use crate::opts::Opts;
use crate::replacer::Replacer;
use crate::presenter::{Msg, ToMsg};
use crate::file_data::{FileData, OverWrite};

pub static GSUB_EXT_PATTERN: &str = r"((.*)(\.)gsub)$";
static GSUB_EXT_NAME: &str = "gsub";
static GSUB_EXT: &str = ".gsub";

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

fn add_gsub_ext(path: impl AsRef<Path>) -> PathBuf {
    let mut file_name = path.as_ref().to_path_buf();
    let new_ext = file_name
        .extension()
        .map(OsString::from)
        .map(|mut ext| {
            ext.push(OsStr::new(GSUB_EXT));
            ext
        }).unwrap_or_else(|| OsString::from(GSUB_EXT_NAME));
    file_name.set_extension(new_ext);
    file_name
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use regex::RegexSet;

    #[test]
    fn adds_expected_gsub_ext_to_a_file_with_no_ext() {
        let p = PathBuf::from("gabagool");
        let p_gsubd = add_gsub_ext(p);
        assert_eq!(p_gsubd.to_string_lossy(), "gabagool.gsub")
    }

    #[test]
    fn adds_expected_gsub_ext_to_a_file_with_an_ext() {
        let p = PathBuf::from("gabagool.txt");
        let p_gsubd = add_gsub_ext(p);
        assert_eq!(p_gsubd.to_string_lossy(), "gabagool.txt.gsub")
    }

    #[test]
    fn gsub_ext_pattern_matches_against_expected_file_paths() {
        let rs = RegexSet::new(&[GSUB_EXT_PATTERN]).expect("didn't compile");

        assert!(rs.is_match("somefile.gsub"));
        assert!(rs.is_match(".some-other-file.txt.gsub"));
        assert!(rs.is_match("./some_dir/some_file.gsub"));

        assert!(!rs.is_match("main.rs"));
        assert!(!rs.is_match("gsub.txt"));
        assert!(!rs.is_match("main.gsub.rs"));
    }
}