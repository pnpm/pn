use std::{path::{Path}};
use std::fs::File;
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let manifest = read_manifest(Path::new("package.json"))?;
    let script_name: Vec<String> = std::env::args().collect();
    let script = manifest["scripts"][&script_name[2]].as_str().unwrap();
    println!("> {:?}", script);
    let mut path_env: String = "node_modules/.bin".to_owned();
    path_env.push_str(":");
    path_env.push_str(env!("PATH"));
    let mut child  = Command::new("sh")
        .env("PATH", path_env)
        .arg("-c")
        .arg(script)
        .stdout(std::process::Stdio::inherit())
        .spawn()?;
    child.wait()?;
    Ok(())
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, std::io::Error> {
    let file = File::open(path)?;
    let manifest: serde_json::Value = serde_json::de::from_reader(file)?;
    Ok(manifest)
}