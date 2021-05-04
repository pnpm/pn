use std::{path::{Path}};
use std::fs::File;
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    match &*args[1] {
        "run" | "run-script" => {
            let manifest = read_manifest(Path::new("package.json"))?;
            let script = manifest["scripts"][&args[2]].as_str().unwrap();
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
        _ => {
            pass_to_pnpm(&args[1..])?;
            Ok(())
        }
    }
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, std::io::Error> {
    let file = File::open(path)?;
    let manifest: serde_json::Value = serde_json::de::from_reader(file)?;
    Ok(manifest)
}

fn pass_to_pnpm(args: &[String]) -> Result<(), std::io::Error> {
    let mut child  = Command::new("pnpm")
        .args(args)
        .stdout(std::process::Stdio::inherit())
        .spawn()?;
    child.wait()?;
    Ok(())
}