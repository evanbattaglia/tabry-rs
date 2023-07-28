mod types;
mod token_matching;
mod config_wrapper;
mod machine_state;
mod machine;
mod result;
mod options_finder;
mod util;
mod shell_tokenizer;

use anyhow::Context;
use types::TabryConf;

fn config_from_file(filename: &str) -> anyhow::Result<TabryConf> {
    let conf_str = std::fs::read_to_string(filename).
        with_context(|| "reading file failed")?;
    let conf: TabryConf = serde_json::from_str(&conf_str).
        with_context(|| "parsing file failed")?;
    Ok(conf)
}

fn print_options(config_filename: &str, tokens: &[String], last_token: &str) -> anyhow::Result<()> {
    let config = config_from_file(&config_filename).with_context(|| "invalid config file")?;
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
    let opts = options_finder.options(last_token);

    for opt in opts {
        println!("{}", opt);
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
fn run_as_compline() -> anyhow::Result<()> {
    // TODO can maybe use match to simplify this
    let args = std::env::args().collect::<Vec<_>>();
    let config_file = args.get(1).with_context(|| "missing config_file")?;
    let compline = args.get(2).with_context(|| "missing compline")?;

    let all_tokens: Vec<String> = shell_words::split(compline)?;
    if util::is_debug() {
        println!("all_tokens={all_tokens:?}")
    }
    let (tokens, last_token): (&[String], &str) = match &all_tokens[..] {
        &[] => panic!("no command line!"),
        [_cmd_name] => (&all_tokens[1..], ""),
        [_cmd_name, rest@.., last] => (rest, last)
    };

    if compline.ends_with(" ") && last_token != "" {
        // TODO temporary hack until I can implement shell_tokenizer right!!!
        let mut tmp = tokens.iter().map(|s| s.clone()).collect::<Vec<_>>();
        tmp.push(last_token.to_owned());
        if util::is_debug() {
            println!("config_file={config_file:?}, tokens={:?} last_token=''", &tmp[..]);
        }
        print_options(config_file, &tmp[..], "")?;
        
    } else {
        if util::is_debug() {
            println!("config_file={config_file:?}, tokens={tokens:?} last_token={last_token:?}");
        }
        print_options(config_file, &tokens[..], last_token)?;
    }
    Ok(())
}

fn main() {
    run_as_compline().unwrap();
}


