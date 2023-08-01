mod types;
mod token_matching;
mod config;
mod machine_state;
mod machine;
mod result;
mod options_finder;
mod util;
mod shell_tokenizer;
mod cached_jsons;
mod config_finder;

use anyhow::Context;

fn print_options(config_filename: &str, tokens: &[String], last_token: &str) -> anyhow::Result<()> {
    let config = config::TabryConf::from_file(&config_filename).with_context(|| "invalid config file")?;
    let mut machine = machine::Machine::new(config);
    for token in tokens {
        // TODO: use std::error::Error trait in Machine so can use "?" here instead of unwrap()
        machine.next(&token).unwrap();
    }

    if util::is_debug() {
        println!("{}", serde_json::to_string_pretty(&machine.state)?);
    }

    let result = machine.to_result();
    let options_finder = options_finder::OptionsFinder::new(result);
    let opts = options_finder.options(last_token)?;

    for opt in opts.options {
        println!("{}", opt);
    }

    if opts.special_options.len() > 0 {
        println!("");
        for opt in opts.special_options {
            println!("{}", opt);
        }
    }
    Ok(())
}

/*
this is broken

// This runs using the filename plus args as tokens
fn run_as_args() -> anyhow::Result<()> {
    // TODO can prpobably use match to simplify this
    let args = std::env::args().collect::<Vec<String>>();
    let [config_file, tokens@.., last_token]: (&str, &[String], &str) = &args[..] else {
        panic!("wrong usage (TODO nicer message");
    }

    print_options(config_file, &tokens[..], last_token)?;

    Ok(())
}
*/

// This runs using the filename plus 2nd arg as compline (shellsplits ARGV[2])
fn run_as_compline(args: &Vec<String>) -> anyhow::Result<()> {
    // TODO can maybe use match to simplify this
    let compline = args.get(1).with_context(|| "missing compline")?;
    let comppoint = args.get(2).with_context(|| "missing comppoint")?;
    let comppoint = comppoint.parse::<usize>()?;

    let tokenized_result = shell_tokenizer::split_with_comppoint(compline, comppoint)?;
    let args = tokenized_result.arguments;
    let last_arg = tokenized_result.last_argument;

    let config_file = config_finder::find_tabry_config(&tokenized_result.command_basename)?;
    let compiled_config_file = cached_jsons::resolve_and_compile_cache_file(&config_file)?;

    //println!("config_file={config_file:?}, tokens={args:?} last_token={last_arg:?}");
    print_options(&compiled_config_file, &args[..], &last_arg)?;
    Ok(())
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 2 && args.get(1).unwrap() == "commands" {
        for command in config_finder::all_supported_commands().unwrap() {
            println!("{}", command);
        }
        std::process::exit(0);
    }
    run_as_compline(&args).unwrap();
}


