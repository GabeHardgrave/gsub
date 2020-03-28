use std::io::{Read, Write, Seek, SeekFrom};
use structopt::StructOpt;
use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();
    let replacer = options.get_replacer()?;

    let mut buffer = String::new();
    for mut fd in options.files() {
        buffer.clear();
        if fd.meta_data.len() as usize > buffer.capacity() {
            buffer.reserve(fd.meta_data.len() as usize - buffer.capacity());
        }

        if let Err(e) = fd.file.read_to_string(&mut buffer) {
            if options.verbose {
                println!(
                    "Skipping {} because {}",
                    fd.path().to_string_lossy(),
                    e,
                );
            }
            continue;
        }

        let new_contents = match replacer.replace(&buffer) {
            Some(s) => s,
            None => continue,
        };
        if options.dry_run {
            println!("Would have replaced `{}` with `{}`", buffer, new_contents);
        } else {
            fd.file.seek(SeekFrom::Start(0))?;
            fd.file.write_all(&new_contents.as_bytes())?;
        }
    }
    Ok(())
}
