use std::io::{Read, Write, Seek, SeekFrom};
use structopt::StructOpt;

use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::from_args();
    let re = options.parse_regex_from_pattern()?;
    for file_result in options.file_iter().each_file() {
        let mut file = file_result?;
        let mut contents = String::new();
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
