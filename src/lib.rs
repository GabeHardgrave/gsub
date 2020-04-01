use lazy_static::lazy_static;

pub static CURRENT_DIR: &str = ".";
pub static GSUB_EXT_NAME: &str = "gsub";
pub static GSUB_EXT: &str = ".gsub";
pub static DEFAULT_FILE_SIZE_STR: &str = "4194304";
lazy_static! {
    pub static ref DEFAULT_FILE_SIZE_INT: u64 = DEFAULT_FILE_SIZE_STR.parse::<u64>().unwrap();
}

pub mod opts;
pub mod gsub;
pub mod tools;
pub mod replacer;
pub mod file_data;
pub mod presenter;
pub mod file_iterator;