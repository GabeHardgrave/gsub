use std::fs::File;
use std::io::Write;
use std::fs;
use std::process::Command;

static CONTENTS: &str = "
How much wood
could a wood chuck chuck
if a wood chuck could chuck
wood
";

fn setup_file(name: &str) {
    assert!(name.starts_with("test-files/"));
    File::create(name)
        .expect("couldn't setup test file")
        .write_all(CONTENTS.as_bytes())
        .expect("couldn't write to test file")
}

fn cleanup_file(name: &str) {
    assert!(name.starts_with("test-files/"));
    fs::remove_file(name).expect("failed to delete file");
}

#[test]
fn test_simple_subs() {
    setup_file("test-files/simple-subs");
    Command::new("./target/debug/gsub")
        .arg("wood")
        .arg("would")
        .arg("test-files/simple-subs")
        .output()
        .expect("unable to execute gsub");
    let file_contents = fs::read_to_string("test-files/simple-subs").expect("unable to read file");
    let expected = "
How much would
could a would chuck chuck
if a would chuck could chuck
would
";
    assert_eq!(file_contents, expected);
    cleanup_file("test-files/simple-subs");
}

#[test]
fn test_simple_subs_dry_run() {
    setup_file("test-files/simple-subs-dry-run");
    Command::new("./target/debug/gsub")
        .arg("wood")
        .arg("would")
        .arg("test-files/simple-subs-dry-run")
        .arg("--dry-run")
        .output()
        .expect("unable to execute gsub");
    let file_contents = fs::read_to_string("test-files/simple-subs-dry-run")
        .expect("unable to read file");
    assert_eq!(file_contents, CONTENTS);
    cleanup_file("test-files/simple-subs-dry-run");
}

#[test]
fn test_no_subs() {
    setup_file("test-files/no-subs");
    Command::new("./target/debug/gsub")
        .arg("gabagool")
        .arg("FAILURE IF THIS SHOWS UP")
        .arg("test-files/no-subs")
        .output()
        .expect("unable to execute gsub");
    let file_contents = fs::read_to_string("test-files/no-subs").expect("unable to read file");
    assert_eq!(file_contents, CONTENTS);
    cleanup_file("test-files/no-subs");
}
