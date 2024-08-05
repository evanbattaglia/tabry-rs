/// Given a "foo.tabry" file, checks if there is a compiled version
/// under the name "foo.tabry.cachejson", and it is _newer_ than
/// the tabry file. If there is, uses that as the tabry config; if there isn't, kicks off the
/// compiler (in the future I'd like to have the compiler in rust) and then uses it for completion.
/// (this could be done in shell but it would add a bit of time to run every tab completion)

use std::process::Command;
use std::fs;
use std::time::SystemTime;
use thiserror::Error;

const TABRY_COMPILER: &str = "tabryc";

#[derive(Error, Debug)]
pub enum TabryCacheError {
    #[error("error compiling tabry file")]
    CompileError(#[from] std::io::Error),
    #[error("error compiling tabry file -- non-zero status code. output {0}, stderr {1}, code {2}")]
    CompileStatusError(String, String, String),
}

fn modtime(filename: &str) -> Option<SystemTime> {
    let metadata = fs::metadata(filename).ok()?;
    metadata.modified().ok()
}

pub fn resolve_and_compile_cache_file(filename: &str) -> Result<String, TabryCacheError> {
    if filename.ends_with(".json") {
        return Ok(filename.to_owned());
    }

    let mut cache_filename = filename.to_string();
    cache_filename.push_str(".cachejson");

    let cache_modtime = modtime(&cache_filename);
    let tabry_modtime = modtime(filename);

    // if needs to be recompiled:
    if cache_modtime.is_none() || cache_modtime < tabry_modtime {
        // compile
        let mut cmd = Command::new(TABRY_COMPILER);
        cmd.arg(filename);
        cmd.arg(&cache_filename);
        let output = cmd.output()?;
        if !output.status.success() {
            let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<bad utf8>").to_owned();
            let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<bad utf8>").to_owned();
            let code_str = output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_owned());
            return Err(TabryCacheError::CompileStatusError(stdout, stderr, code_str));
        }
    }

    Ok(cache_filename)
}
