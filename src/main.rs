use std::io::{Write, Seek, SeekFrom};
use gsub::opts::Opts;
use gsub::replacer::GsubResult;

fn main() -> std::io::Result<()> {
    let options = Opts::parse();
    let file_iter = options.file_iter_config()?;
    let mut replacer = options.replacer()?;
    let presenter = options.presenter();

    for fd_result in file_iter {
        let mut fd = match fd_result {
            Ok(fd) => fd,
            Err(e) => {
                presenter.wax_verbose(|| e);
                continue;
            },
        };

        let new_contents = match replacer.gsub(&mut fd) {
            GsubResult::NoChange => continue,
            GsubResult::Error(e) => {
                presenter.wax_verbose(|| {
                    format!("Skipping {} because {}", fd.path().to_string_lossy(), e)
                });
                continue;
            },
            GsubResult::Replaced(s) => s,
        };

        if options.dry_run {
            presenter.wax(format!("Would have replaced `{}` with `{}`",
                replacer.old_contents(),
                new_contents
            ));
        } else {
            fd.seek(SeekFrom::Start(0))?;
            fd.write_all(&new_contents.as_bytes())?;
            presenter.wax(format!("Updated {}", fd.path().to_string_lossy()));
        }
    }
    Ok(())
}