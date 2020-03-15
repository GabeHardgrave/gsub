use std::io::{Read, Write, Seek, SeekFrom, Error, ErrorKind};
use std::path::PathBuf;
use std::fs::OpenOptions;
use structopt::StructOpt;
use regex::Regex;

#[derive(Debug, StructOpt)]
#[structopt(name = "gsub", about = "Bulk substitutions on a given file")]
struct Opts {
    #[structopt(short, long)]
    dry_run: bool,

    /// The pattern you want to replace
    pattern: String,

    /// String for replacement
    replacement: String,

    /// The file you want to make substitutions on
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();
    let re = Regex::new(&options.pattern).map_err(|regex_err| {
        Error::new(ErrorKind::Other, regex_err.to_string())
    })?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open(options.file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let new_contents = re.replace_all(&contents, options.replacement.as_str());
    if options.dry_run {
        println!("Would have replaced `{}` with `{}`", contents, new_contents);
    } else {
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&new_contents.as_bytes())?;
    }
    Ok(())
}