use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use build_fs_tree::{dir, file, Build, MergeableFileSystemTree};
use pretty_assertions::assert_eq;
use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn run_script() {
    let temp_dir = tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    fs::write(
        package_json_path,
        r#"{"name": "pn", "version": "0.0.0", "scripts": {"test": "echo hello world"}}"#,
    )
    .unwrap();

    Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["run", "test"])
        .assert()
        .success()
        .stdout("hello world\n")
        .stderr(format!(
            "\n> pn@0.0.0 {}\n> echo hello world\n\n",
            &temp_dir.path().display()
        ));
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
        .stdout("hello from workspace root\n")
        .stderr(format!(
            "\n> @ {}\n> echo hello from workspace root\n\n",
            &temp_dir.path().display()
        ));
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

#[test]
fn list_script_names() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("package.json"),
        include_str!("fixtures/list-script-names/package.json"),
    )
    .unwrap();
    let assertion = Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("run")
        .assert()
        .success();
    let output = assertion.get_output();
    let received = String::from_utf8_lossy(&output.stdout);
    eprintln!("STDOUT:\n{received}\n");
    let expected = include_str!("fixtures/list-script-names/stdout.txt").replace("\r\n", "\n");
    assert_eq!(received.trim(), expected.trim());
}

#[test]
fn list_no_script_names() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
    Command::cargo_bin("pn")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("run")
        .assert()
        .success()
        .stdout("There are no scripts in package.json\n");
}
