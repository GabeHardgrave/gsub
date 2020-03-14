use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gsub", about = "Bulk substitutions on a given file")]
struct Config {
    #[structopt(short, long)]
    dry_run: bool,

    /// The pattern you want to replace
    pattern: String,

    /// String for replacement
    replacement: String,

    /// The file you want to make substitions on
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let config = Config::from_args();
    println!("Parsed gaba Config {:?}", config);
}
