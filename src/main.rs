use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, env, error::Error, ffi::OsString, fmt, fs::File, process::Command,
};

#[derive(Debug)]
struct MissingScriptError(String);

impl fmt::Display for MissingScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing script {:?}", self.0)
    }
}

impl Error for MissingScriptError {}

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct NodeManifest {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

fn main() {
    use ansi_term::Color::*;
    if let Err(error) = run() {
        eprintln!(
            "{prefix} {error}",
            prefix = Black.on(Red).paint("\u{2009}ERROR\u{2009}"),
            error = Red.paint(error.to_string()),
        );
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    match &*args[1] {
        "run" | "run-script" => {
            let manifest: NodeManifest = serde_json::de::from_reader(File::open("package.json")?)?;
            let script_name = &args[2];
            if let Some(script) = manifest.scripts.get(script_name) {
                eprintln!("> {:?}", script);
                run_script(script)?;
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
            run_script(&args[1..].join(" "))?;
            Ok(())
        }
    }
}

fn run_script(script: &str) -> Result<(), Box<dyn Error>> {
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
