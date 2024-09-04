use anyhow::Context;

use std::io::Read;

use tabry::{
    app::{cached_jsons, config_finder, shell_tokenizer},
    core::{config, util},
    engine::{machine, options_finder},
};

// can maybe move some/most of this to app module?

fn print_options(config_filename: &str, tokens: &[String], last_token: &str) -> anyhow::Result<()> {
    let config =
        config::TabryConf::from_file(config_filename).with_context(|| "invalid config file")?;
    let result =
        machine::Machine::run(config, tokens).with_context(|| "Tabry machine parse error")?;

    if util::is_debug() {
        println!("{}", serde_json::to_string_pretty(&result.state)?);
    }

    let options_finder = options_finder::OptionsFinder::new(result);
    let opts = options_finder.options(last_token)?;

    for opt in &opts.options {
        println!("{}", opt);
    }

    if !opts.special_options.is_empty() {
        if opts.options.is_empty() {
            // if no normal options, bash wrapper seems to require an extra empty line :shrug:
            println!();
        }
        println!();
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
    eprintln!("Usage: {} bash", cmd_name);
    eprintln!("  prints bash code to be eval'd to setup tabry completions");
    std::process::exit(1);
}

fn compile() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    // TODO: I think this could be improved. maybe eyre will help
    let tabry_conf = match tabry::lang::compile(&input) {
        Err(e) => {
            eprintln!("compile error: {}", e);
            std::process::exit(1);
        }
        Ok(conf) => conf,
    };
    let json = serde_json::to_string_pretty(&tabry_conf);
    print!("{}", json.unwrap());
}

fn commands() {
    for command in config_finder::all_supported_commands().unwrap() {
        println!("{}", command);
    }
}

fn escape(s: &str) -> String {
    // replace single quote with ' '"'"' to escape it in bash:
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn escaped_exe() -> String {
    escape(std::env::current_exe().unwrap().to_str().unwrap())
}

const TABRY_BASH_SH: &str = include_str!("../shell/tabry_bash.sh");
fn bash(imports_path: Option<&&str>) {
    if let Some(path) = imports_path {
        println!("_tabry_rs_imports_path='{}'", escape(path));
    }

    println!("_tabry_rs_executable='{}'", escaped_exe());
    print!("{}", TABRY_BASH_SH);
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let args_strs = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match args_strs.as_slice() {
        [_, "compile"] => compile(),
        [_, "commands"] => commands(),
        [_, "bash"] => bash(None),
        [_, "bash", imports_path] => bash(Some(imports_path)),
        [_, compline, comppoint] => run_as_compline(compline, comppoint).unwrap(),
        _ => usage(args_strs.first().copied()),
    }
}
