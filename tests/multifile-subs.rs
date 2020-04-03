use std::fs::File;
use std::io::Write;
use std::fs;
use std::process::Command;

static CONTENTS: &str = "
My baby takes the moooornin train
He works from nine til five aaaand then
He takes another home again
To find me waitin' for him
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
fn test_sub_multiple_files() {
    fs::create_dir_all("test-files/test_sub_multiple_files").expect("unable to create directory");
    setup_file("test-files/test_sub_multiple_files/a");
    setup_file("test-files/test_sub_multiple_files/b");
    setup_file("test-files/test_sub_multiple_files_c");

    Command::new("./target/debug/gsub")
        .arg("moooornin train")
        .arg("afternoon plane")
        .arg("test-files/test_sub_multiple_files")
        .arg("test-files/test_sub_multiple_files_c")
        .output()
        .expect("unable to execute gsub");

    let expected = "
My baby takes the afternoon plane
He works from nine til five aaaand then
He takes another home again
To find me waitin' for him
";
    let files = vec![
        "test-files/test_sub_multiple_files/a",
        "test-files/test_sub_multiple_files/b",
        "test-files/test_sub_multiple_files_c",
    ];
    for f in files {
        let contents = fs::read_to_string(f).expect("unable to read file");
        assert_eq!(contents, expected)
    }

    cleanup_file("test-files/test_sub_multiple_files_c");
    fs::remove_dir_all("test-files/test_sub_multiple_files").unwrap()
}
