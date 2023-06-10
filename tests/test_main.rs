use assert_cmd::prelude::*;
use build_fs_tree::*;
use std::fs;
use std::process::Command;

#[test]
fn run_script() {
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
fn run_from_workspace_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tree = MergeableFileSystemTree::<&str, &str>::from(dir! {
        "package.json" => build_fs_tree::file!(r#"{"scripts": {"test": "echo hello from workspace root"}}"#),
        "pnpm-workspace.yaml" => build_fs_tree::file!("packages: ['packages/*']"),
        "packages" => dir! {
            "foo" => dir! {
                "package.json" => build_fs_tree::file!(r#"{"scripts": {"test": "echo hello from foo"}}"#),
            },
        },
    });
    tree.build(&temp_dir.path().to_path_buf()).unwrap();

    Command::cargo_bin("pn")
        .unwrap()
        .current_dir(temp_dir.path().join("packages/foo"))
        .args(["--workspace-root", "run", "test"])
        .assert()
        .success()
        .stdout("hello from workspace root\n");
}

#[test]
fn workspace_root_not_found_error() {
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
