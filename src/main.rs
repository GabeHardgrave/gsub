use gsub::gsub::gsub;
use gsub::opts::Opts;

fn main() -> std::io::Result<()> {
    let options = Opts::parse()?;
    let file_iter = options.file_iter_config()?;
    let mut replacer = options.replacer()?;
    let presenter = options.presenter();

    file_iter.into_iter().for_each(|fd_result| {
        match gsub(fd_result, &mut replacer, &options) {
            Ok(Some(msg)) => presenter.wax(msg),
            Err(msg) => presenter.wax(msg),
            _ => {},
        }
    });
    Ok(())
}