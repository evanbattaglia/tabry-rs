// TODO use lazy_static! here
// https://stackoverflow.com/questions/37405835/populating-a-static-const-with-an-environment-variable-at-runtime-in-rust
// and actually juts see what people use for logging in rust
pub fn is_debug() -> bool {
    match std::env::var("TABRY_DEBUG") {
        Ok(s) => s != "0" && s != "false",
        Err(_) => false,
    }
}
