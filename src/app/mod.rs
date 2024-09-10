// see lib.rs for hierarchy description
mod cached_jsons;
mod config_finder;
mod shell_tokenizer;

/// Main app functionality
use anyhow::Context;
use std::io::Read;

use crate::{
    core::{config, util},
    engine::{machine, options_finder},
    lang,
};

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

// This runs using the filename plus 2nd arg as compline (shellsplits ARGV[2])
pub fn run_as_compline(compline: &str, comppoint: &str) -> anyhow::Result<()> {
    let comppoint = comppoint.parse::<usize>()?;

    let tokenized_result = shell_tokenizer::split_with_comppoint(compline, comppoint)?;
    let args = tokenized_result.arguments;
    let last_arg = tokenized_result.last_argument;

    let config_file = config_finder::find_tabry_config(&tokenized_result.command_basename)?;
    let compiled_config_file = cached_jsons::resolve_and_compile_cache_file(&config_file)?;

    print_options(&compiled_config_file, &args[..], &last_arg)?;
    Ok(())
}

pub fn compile() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    // TODO: I think this could be improved. maybe eyre will help
    let tabry_conf = match lang::compile(&input) {
        Err(e) => {
            eprintln!("compile error: {}", e);
            std::process::exit(1);
        }
        Ok(conf) => conf,
    };
    let json = serde_json::to_string_pretty(&tabry_conf);
    print!("{}", json.unwrap());
}

pub fn commands() {
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

const TABRY_BASH_SH: &str = include_str!("../../shell/tabry_bash.sh");
pub fn bash(imports_path: Option<&str>, no_auto: bool) {
    if let Some(path) = imports_path {
        println!("_tabry_rs_imports_path={}", escape(path));
    }
    println!("_tabry_rs_executable={}", escaped_exe());
    print!("{}", TABRY_BASH_SH);

    if !no_auto {
        // TODO name things consistently between fish + bash
        println!("_tabry_rs_complete_all");
    }
}

const TABRY_FISH_SH: &str = include_str!("../../shell/tabry_fish.fish");
pub fn fish(imports_path: Option<&str>, no_auto: bool) {
    if let Some(path) = imports_path {
        println!("set -x TABRY_IMPORT_PATH {}", escape(path));
    }

    println!("set -x _tabry_rs_executable {}", escaped_exe());
    print!("{}", TABRY_FISH_SH);

    if !no_auto {
        println!("tabry_completion_init_all");
    }
}



