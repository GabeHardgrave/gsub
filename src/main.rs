use std::fs;
use gsub::opts::Opts;
use gsub::file_data::OverWrite;
use gsub::replacer::GsubResult;
use gsub::tools::add_gsub_ext;

fn main() -> std::io::Result<()> {
    let options = Opts::parse()?;
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
            presenter.wax(format!("Would have updated {}", fd.path().to_string_lossy()));
        } else if options.copy_on_write {
            let new_file_name = add_gsub_ext(fd.path());
            fs::write(new_file_name, new_contents)?;
        } else {
            fd.overwrite(&new_contents.as_bytes())?;
            presenter.wax(format!("Updated {}", fd.path().to_string_lossy()));
        }
    }
    Ok(())
}