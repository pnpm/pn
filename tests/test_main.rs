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
        .args(["run", "test"])
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn test_workspace_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path();
    fs::write(
        temp_dir_path.join("package.json"),
        r#"{"scripts": {"test": "echo hello from workspace root"}}"#,
    )
    .unwrap();
    fs::write(
        temp_dir_path.join("pnpm-workspace.yaml"),
        r#"packages: ["packages/*"]"#,
    )
    .unwrap();
    let project_foo_path = temp_dir_path.join("packages/foo");
    fs::create_dir_all(&project_foo_path).unwrap();
    fs::write(
        project_foo_path.join("package.json"),
        r#"{"scripts": {"test": "echo hello from foo"}}"#,
    )
    .unwrap();

    Command::cargo_bin("pn")
        .unwrap()
        .current_dir(project_foo_path)
        .args(["--workspace-root", "run", "test"])
        .assert()
        .success()
        .stdout("hello from workspace root\n");
}

#[test]
fn test_workspace_root_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    fs::write(
        package_json_path,
        r#"{"scripts": {"test": "echo hello world"}}"#,
    )
    .unwrap();

    let assertion = Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["--workspace-root", "run", "test"])
        .assert()
        .failure();
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stderr.contains("--workspace-root may only be used in a workspace"));
}
