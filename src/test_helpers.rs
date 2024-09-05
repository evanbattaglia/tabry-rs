use serde::Deserialize;
use std::borrow::Cow;
use std::fs;

pub fn load_fixture_file_text(filename: &str) -> String {
    let path: Cow<str> = if filename.starts_with("fixtures/") {
        Cow::Borrowed(filename)
    } else {
        Cow::Owned(format!("fixtures/{filename}"))
    };
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|_| panic!("Failed to read fixtures/{filename}"))
}

pub fn each_file_in_dir_with_extension<'a>(
    dir: &'a str,
    extension: &'a str,
) -> impl Iterator<Item = String> + 'a {
    let dir_iter = fs::read_dir(dir).unwrap_or_else(|_| panic!("Failed to read dir {dir}"));
    dir_iter
        .filter(move |entry| {
            let entry = entry
                .as_ref()
                .unwrap_or_else(|_| panic!("Error with DirEntry {entry:?}"));
            match entry.path().extension() {
                Some(ext) => {
                    let actual_ext = ext
                        .to_str()
                        .unwrap_or_else(|| panic!("Bad UTF-8 {entry:?}"));
                    actual_ext == extension
                }
                None => false,
            }
        })
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
}

pub fn load_fixture_file<T: for<'a> Deserialize<'a>>(filename: &str) -> T {
    let file_str = load_fixture_file_text(filename);
    serde_json::from_str::<T>(&file_str)
        .unwrap_or_else(|_| panic!("Failed to parse fixtures/{filename}"))
}
