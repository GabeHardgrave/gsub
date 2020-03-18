use std::io::{Read, Write, Seek, SeekFrom};
use structopt::StructOpt;

use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();
    let re = options.parse_regex_from_pattern()?;
    let files_and_sizes = options
        .file_iter()
        .each_file_with_size()
        .filter(|(_file, size)| *size <= options.max_file_size);

    for (mut file, size) in files_and_sizes {
        let mut contents = String::with_capacity(size);
        file.read_to_string(&mut contents)?;

        let new_contents = re.replace_all(&contents, options.replacement.as_str());
        if options.dry_run {
            println!("Would have replaced `{}` with `{}`", contents, new_contents);
        } else {
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&new_contents.as_bytes())?;
        }
    }
    Ok(())
}
