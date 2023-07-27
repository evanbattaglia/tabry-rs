mod types;
mod token_matching;
mod config_wrapper;
mod machine_state;
mod machine;

use anyhow::Context;
use types::TabryConf;

fn config_from_file(filename: &str) -> anyhow::Result<TabryConf> {
    let conf_str = std::fs::read_to_string(filename).
        with_context(|| "reading file failed")?;
    let conf: TabryConf = serde_json::from_str(&conf_str).
        with_context(|| "parsing file failed")?;
    Ok(conf)
}

fn run() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    let config_file = args.get(1).with_context(|| "missing argument context_file")?;
    let config = config_from_file(&config_file).with_context(|| "invalid config file")?;

    let mut machine = machine::Machine::new(config);
    for arg in &args[2..] {
        // TODO: use std::error::Error trait in Machine so can use "?" here instead of unwrap()
        machine.next(&arg).unwrap();
    }

    println!("{}", serde_json::to_string_pretty(&machine.state)?);
    Ok(())
}

fn main() {
    run().unwrap();
}


