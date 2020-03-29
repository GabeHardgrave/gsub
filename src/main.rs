use std::io::{Read, Write, Seek, SeekFrom};
use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::parse();
    let file_iter = options.file_iter_config()?;
    let replacer = options.replacer()?;
    let presenter = options.presenter();

    let mut buffer = String::new();
    for fd_result in file_iter {
        let mut fd = match fd_result {
            Ok(fd) => fd,
            Err(e) => {
                presenter.wax_verbose(|| e);
                continue;
            },
        };

        buffer.clear();
        if fd.meta_data.len() as usize > buffer.capacity() {
            buffer.reserve(fd.meta_data.len() as usize - buffer.capacity());
        }

        if let Err(e) = fd.file.read_to_string(&mut buffer) {
            presenter.wax_verbose(|| {
                format!("Skipping {} because {}", fd.path().to_string_lossy(), e)
            });
            continue;
        }

        let new_contents = match replacer.replace(&buffer) {
            Some(s) => s,
            None => continue,
        };
        if options.dry_run {
            presenter.wax(format!("Would have replaced `{}` with `{}`", buffer, new_contents));
        } else {
            fd.file.seek(SeekFrom::Start(0))?;
            fd.file.write_all(&new_contents.as_bytes())?;
        }
    }
    Ok(())
}
