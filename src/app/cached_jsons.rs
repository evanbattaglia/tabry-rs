/// Given a "foo.tabry" file, checks if there is a compiled version
/// under the name "foo.tabry.cachejson", and it is _newer_ than
/// the tabry file. If there is, uses that as the tabry config; if there isn't, kicks off the
/// compiler (in the future I'd like to have the compiler in rust) and then uses it for completion.
/// (this could be done in shell but it would add a bit of time to run every tab completion)
use std::fs;
use std::time::SystemTime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TabryCacheError {
    #[error("error compiling tabry file -- IO error: {0}")]
    CompileFileError(#[from] std::io::Error),
    #[error("error compiling tabry file -- invalid tabry file encountered: {0}")]
    CompileError(#[from] crate::lang::LangError),
    #[error("error compiling tabry file -- JSON serialization error: {0}")]
    JSONSerializationError(#[from] serde_json::Error),
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
        let tabry_file = fs::read_to_string(filename)?;
        let compiled = crate::lang::compile(&tabry_file);
        let json = serde_json::to_string(&compiled?)?;
        fs::write(&cache_filename, json)?;
        // TODO ideally, shouldn't bother reading and decoding the JSON file since we alredy have the
        // compiled form in memory
    }

    Ok(cache_filename)
}
