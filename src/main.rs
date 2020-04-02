use std::io;
use std::error::Error;
use gsub::gsub::gsub;
use gsub::opts::Opts;
use gsub::file_data::OpenFileData;

fn io_err<E>(e: E) -> io::Error
    where E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, e)
}

fn main() -> std::io::Result<()> {
    let options = Opts::parse().map_err(io_err)?;
    let replacer = options.replacer().map_err(io_err)?;
    let blacklist = options.dir_entry_blacklist().map_err(io_err)?;
    let opener = options.open_opts();
    let presenter = options.presenter();
    let walker = options.walk_builder().build_parallel();

    walker.run(|| {
        Box::new(|result| {
            let entry = match result {
                Ok(e) => e,
                Err(_) => { return ignore::WalkState::Continue }
            };

            if let Some(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if blacklist.is_match(&entry.file_name().to_string_lossy()) {
                        return ignore::WalkState::Skip;
                    } else {
                        return ignore::WalkState::Continue;
                    }
                }
            } else {
                return ignore::WalkState::Continue; // stdio, maybe we'll support it in the future
            };

            match gsub(
                opener.open_fd(entry),
                &replacer,
                &options,
            ) {
                Ok(Some(msg)) | Err(msg) => presenter.wax(msg),
                Ok(None) => {},
            }
            ignore::WalkState::Continue
        })
    });

    Ok(())
}