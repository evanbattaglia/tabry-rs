mod types;
mod token_matching;
mod config_wrapper;
mod machine_state;
mod machine;

use anyhow::Context;
use types::TabryConf;

// #[derive(Debug, thiserror::Error)]
// enum TabryConfigError {
//     #[error("Could not read tabry config: {0}")]
//     IoError(#[from] std::io::Error),
//     #[error("Tabry config bad JSON: {0}")]
//     JsonError(#[from] serde_json::Error),
// }

fn config_from_file(filename: &str) -> anyhow::Result<TabryConf> {
    let conf_str = std::fs::read_to_string(filename)?;
    let conf: TabryConf = serde_json::from_str(&conf_str)?;
    Ok(conf)
}

fn run() -> anyhow::Result<()> {
    let mut args_iter = std::env::args().into_iter();
    let config_file = args_iter.next().with_context(|| "missing argument context_file")?;
    let mut machine = machine::Machine::new(config_from_file(&config_file)?);
    for arg in args_iter {
        // TODO: use std::error::Error trait in Machine so can use "?" here instead of unwrap()
        machine.next(&arg).unwrap();
    }
    Ok(())
}

fn main() {
    run().unwrap();
}


