use std::io;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use crate::{GSUB_EXT, GSUB_EXT_NAME, CURRENT_DIR};

pub fn to_io_err(msg: String) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        msg
    )
}

pub fn is_hidden<R>(path: R) -> bool where R: AsRef<OsStr> {
    path.as_ref()
        .to_str()
        .map(|s| s.starts_with(".") && s != CURRENT_DIR)
        .unwrap_or(false)
}

pub fn has_gsub_ext<R>(path: R) -> bool where R: AsRef<OsStr> {
    path.as_ref()
        .to_str()
        .map(|s| s.ends_with(GSUB_EXT))
        .unwrap_or(false)
}

pub fn add_gsub_ext(path: impl AsRef<Path>) -> PathBuf {
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
}