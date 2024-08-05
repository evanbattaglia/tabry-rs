use anyhow::Context;

use tabry::{
    lang::ast,
    app::{
        cached_jsons,
        config_finder,
        shell_tokenizer,
    },
    core::{
        config,
        util,
    },
    engine::{
        machine,
        options_finder,
    }
};

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

    for opt in &opts.options {
        println!("{}", opt);
    }

    if opts.special_options.len() > 0 {
        if opts.options.is_empty() {
            // if no normal options, bash wrapper seems to require an extra empty line :shrug:
            println!("");
        }
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
fn run_as_compline(compline: &str, comppoint: &str) -> anyhow::Result<()> {
    // TODO can maybe use match to simplify this
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

fn usage(cmd_name: Option<&str>) {
    let cmd_name: &str = cmd_name.unwrap_or("tabry");
    eprintln!("Usage: {} <compline> <comppoint>", cmd_name);
    eprintln!("  get completions. usually used via tabry_bash.sh");
    eprintln!("Usage: {} commands", cmd_name);
    eprintln!("  list all commands that tabry can find configs for");
    eprintln!("Usage: {} compile < file.tabry > file.json", cmd_name);
    eprintln!("  compile a tabry file to json");
    std::process::exit(1);
}

fn compile() {
    // TODO
    let ast = ast::parse("
        cmd control-vehicle
        arg {
          opts const car
        }
    ");
    let ast = ast.unwrap();
    let config = tabry::lang::translator::translate(&ast);
    let json = serde_json::to_string(&config);
    print!("{}", json.unwrap());
}

fn commands() {
    for command in config_finder::all_supported_commands().unwrap() {
        println!("{}", command);
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let args_strs = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match args_strs.as_slice() {
        [_, "compile"] => compile(),
        [_, "commands"] => commands(),
        [_, compline, comppoint] => run_as_compline(compline, comppoint).unwrap(),
        _ => usage(args_strs.get(0).copied())
    }
}


