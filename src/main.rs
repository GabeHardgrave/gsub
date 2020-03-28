use std::io::{Read, Write, Seek, SeekFrom};
use structopt::StructOpt;
use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();
    let replacer = options.get_replacer()?;

    for mut fd in options.files() {
        let mut contents = String::with_capacity(fd.meta_data.len() as usize);
        if let Err(_e) = fd.file.read_to_string(&mut contents) {
            continue;
        }

        let new_contents = match replacer.replace(&contents) {
            Some(s) => s,
            None => continue,
        };
        if options.dry_run {
            println!("Would have replaced `{}` with `{}`", contents, new_contents);
        } else {
            fd.file.seek(SeekFrom::Start(0))?;
            fd.file.write_all(&new_contents.as_bytes())?;
        }
    }
    Ok(())
}
