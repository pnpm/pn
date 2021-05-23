use std::env;
use std::error::Error;
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
            match manifest["scripts"][script_name].as_str() {
                None => Err(Box::new(MissingScriptError(script_name.into()))),
                Some(script) => {
                    println!("> {:?}", script);
                    let mut path_env: String = "node_modules/.bin".to_owned();
                    if let Ok(path) = env::var("PATH") {
                        path_env.push(':');
                        path_env += &path;
                    }
                    let mut child = Command::new("sh")
                        .env("PATH", path_env)
                        .arg("-c")
                        .arg(script)
                        .stdout(std::process::Stdio::inherit())
                        .spawn()?;
                    child.wait()?;
                    Ok(())
                }
            }
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
    let mut child = Command::new("pnpm")
        .args(args)
        .stdout(std::process::Stdio::inherit())
        .spawn()?;
    child.wait()?;
    Ok(())
}
