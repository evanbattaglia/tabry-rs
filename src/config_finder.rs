// find *.tabry of *.json for command in RABRY_CONFIG_PATH

use thiserror::Error;
const EXTENSIONS: [&str; 2] = [".tabry", ".json"];

#[derive(Error, Debug)]
#[error("config for {0} cannot be found in RABRY_IMPORT_PATH ({1})")]
pub struct ConfigFinderError(String, String);


pub fn import_path() -> String {
    let mut import_path = std::env::var("RABRY_IMPORT_PATH").unwrap_or("".to_owned());
    if import_path == "" {
        import_path = "./".to_owned();
    }
    import_path
}

pub fn find_tabry_config(command_name: &str) -> Result<String, ConfigFinderError> {
    for import_dir in import_path().split(":") {
        for ext in &EXTENSIONS {
            let mut path = std::path::PathBuf::from(import_dir.clone());
            path.push(format!("{}{}", command_name, ext));
            let path = path.to_str().unwrap();
            // if exists:
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
    }

    Err(ConfigFinderError(command_name.to_owned(), import_path()))
}

pub fn all_supported_commands() -> Result<Vec<String>, std::io::Error> {
    let mut res = vec![];
    for import_dir in import_path().split(":") {
        let read_dir = std::fs::read_dir(import_dir);
        if read_dir.is_err() {
            continue;
        }
        for entry in read_dir? {
            let path = entry?.path();
            let path = path.to_str();
            let path = match path {
                Some(path) => path,
                None => continue,
            };
            for ext in &EXTENSIONS {
                if path.ends_with(ext) {
                    let cmd = path[0..path.len() - ext.len()].split("/").last();
                    if cmd.is_some() {
                        res.push(cmd.unwrap().to_owned());
                    }
                }
            }
        }
    }
    Ok(res)
}
