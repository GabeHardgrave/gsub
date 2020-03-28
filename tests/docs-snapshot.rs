use std::process::Command;

#[test]
fn help_docs_match_expected_snapshot() {
    let expected_docs = "\
gsub 0.1.0
Regex substitution for files and directories

USAGE:
    gsub [FLAGS] [OPTIONS] <pattern> <replacement> [files]...

FLAGS:
    -d, --dry-run    
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -m, --skip-files-larger-than <max-file-size>    Skip files larger than the given number of bytes [default: 4194304]

ARGS:
    <pattern>        The pattern you want to replace
    <replacement>    String for replacement
    <files>...       List of files/directories you want to gsub on. If unspecified, uses the current directory
"
        .to_string();

    let help_docs_raw = Command::new("./target/debug/gsub")
        .arg("--help")
        .output()
        .expect("can't execute `gsub --help`")
        .stdout;
    let help_docs = String::from_utf8(help_docs_raw)
        .expect("Help docs aren't valid UTF8");
    assert_eq!(expected_docs, help_docs)
}
