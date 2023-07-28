
// TODO use lazy_static! here
// https://stackoverflow.com/questions/37405835/populating-a-static-const-with-an-environment-variable-at-runtime-in-rust
// TODO no idea why it's saying not used when it is by result...
pub fn is_debug() -> bool {
    match std::env::var("RABRY_DEBUG") {
        Ok(s) => s != "0" && s != "false",
        Err(_) => false
    }
}
