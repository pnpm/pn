use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn test_run_script() {
    // Given
    // Create a temporary directory and a package.json file
    let temp_dir = tempfile::tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    fs::write(
        package_json_path,
        r#"{"scripts": {"test": "echo hello world"}}"#,
    )
    .unwrap();

    // When
    // Run the CLI with the "run" command
    Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("run")
        .arg("test")
        .assert()
        .success()
        .stdout("hello world\n");
}
