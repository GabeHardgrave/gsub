use std::io::{Read, Write, Seek, SeekFrom};
use structopt::StructOpt;
use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();

    let replacer = options.get_replacer()?;
    let files_and_metadata = options
        .file_iter()
        .each_file_with_metadata()
        .filter(|(_file, metadata)| metadata.len() as usize <= options.max_file_size);

    for (mut file, metadata) in files_and_metadata {
        let mut contents = String::with_capacity(metadata.len() as usize);
        if let Err(_e) = file.read_to_string(&mut contents) {
            continue;
        }

        let new_contents = match replacer.replace(&contents) {
            Some(s) => s,
            None => continue,
        };
        if options.dry_run {
            println!("Would have replaced `{}` with `{}`", contents, new_contents);
        } else {
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&new_contents.as_bytes())?;
        }
    }
    Ok(())
}
