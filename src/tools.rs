use std::io;
use std::ffi::OsStr;

pub fn to_io_err(msg: String) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        msg
    )
}

pub fn is_hidden<R>(path: R) -> bool where R: AsRef<OsStr> {
    path.as_ref()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}