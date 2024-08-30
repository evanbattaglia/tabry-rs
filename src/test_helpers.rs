use serde::Deserialize;
use std::fs;

pub fn load_fixture_file<T: for<'a> Deserialize<'a>>(filename: &str) -> T {
    let file_str = fs::read_to_string(format!("fixtures/{filename}")).unwrap();
    serde_json::from_str::<T>(&file_str).unwrap()
}
