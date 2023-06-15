use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use build_fs_tree::{dir, file, Build, MergeableFileSystemTree};
use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn run_script() {
    let temp_dir = tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    fs::write(
        package_json_path,
        r#"{"scripts": {"test": "echo hello world"}}"#,
    )
    .unwrap();

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
    let temp_dir = tempdir().unwrap();
    let tree = MergeableFileSystemTree::<&str, &str>::from(dir! {
        "package.json" => file!(r#"{"scripts": {"test": "echo hello from workspace root"}}"#),
        "pnpm-workspace.yaml" => file!("packages: ['packages/*']"),
        "packages" => dir! {
            "foo" => dir! {
                "package.json" => file!(r#"{"scripts": {"test": "echo hello from foo"}}"#),
            },
        },
    });
    tree.build(&temp_dir).unwrap();

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
    let temp_dir = tempdir().unwrap();
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("STDERR:\n{stderr}\n");
    assert!(stderr.contains("--workspace-root may only be used in a workspace"));
}

#[test]
fn no_package_manifest_error() {
    let temp_dir = tempdir().unwrap();
    let assertion = Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["run", "test"])
        .assert()
        .failure();
    let output = assertion.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("STDERR:\n{stderr}\n");
    assert!(stderr.contains("File not found: "));
    let expected_path = temp_dir
        .path()
        .join("package.json")
        .display()
        .to_string()
        .replace('\\', "\\\\");
    dbg!(&expected_path);
    assert!(stderr.contains(&expected_path));
}
