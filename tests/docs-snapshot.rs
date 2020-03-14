use std::process::Command;

#[test]
fn test_docs_match_expected_snapshot() {
    let expected_docs = "\
gsub 0.1.0
Bulk substitutions on a given file

USAGE:
    gsub [FLAGS] <pattern> <replacement> <file>

FLAGS:
    -d, --dry-run    
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <pattern>        The pattern you want to replace
    <replacement>    String for replacement
    <file>           The file you want to make substitions on
"
        .to_string();

    let help_docs_raw = Command::new("./target/debug/gsub")
        .arg("--help")
        .output()
        .expect("can't execute `gsub --help`")
        .stdout;
    let help_docs = String::from_utf8(help_docs_raw).expect("Help docs aren't valid UTF8");
    assert_eq!(expected_docs, help_docs)
}
