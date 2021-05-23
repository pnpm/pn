use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
struct MissingScriptError(String);

impl fmt::Display for MissingScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ERROR Missing script {}", self.0)
    }
}

impl Error for MissingScriptError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    match &*args[1] {
        "run" | "run-script" => {
            let manifest = read_manifest(Path::new("package.json"))?;
            let script_name = &args[2];
            if let Some(script) = manifest["scripts"][script_name].as_str() {
                eprintln!("> {:?}", script);
                let mut path_env = OsString::from("node_modules/.bin");
                if let Some(path) = env::var_os("PATH") {
                    path_env.push(":");
                    path_env.push(path);
                }
                let mut child = Command::new("sh")
                    .env("PATH", path_env)
                    .arg("-c")
                    .arg(script)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn()?;
                child.wait()?;
                Ok(())
            } else {
                Err(Box::new(MissingScriptError(script_name.into())))
            }
        }
        "install" | "i" | "update" | "up" => {
            pass_to_pnpm(&args[1..])?;
            Ok(())
        }
        _ => {
            let mut path_env = OsString::from("node_modules/.bin");
            if let Some(path) = env::var_os("PATH") {
                path_env.push(":");
                path_env.push(path);
            }
            let mut child  = Command::new("sh")
                .env("PATH", path_env)
                .arg("-c")
                .arg(&args[1..].join(" "))
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()?;
            child.wait()?;
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
    let mut child = Command::new("pnpm")
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;
    child.wait()?;
    Ok(())
}
