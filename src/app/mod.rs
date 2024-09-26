// see lib.rs for hierarchy description
mod cached_jsons;
mod config_finder;
mod shell_tokenizer;

/// Main app functionality
use color_eyre::eyre::{Context, Result, eyre};
use std::io::Read;

use crate::{
    core::{config, util},
    engine::{machine, options_finder},
    lang,
};

fn print_options(config_filename: &str, tokens: &[String], last_token: &str, include_descriptions: bool) -> Result<()> {
    let config =
        config::TabryConf::from_file(config_filename).with_context(|| "invalid config file")?;
    let result =
        machine::Machine::run(config, tokens).with_context(|| "Tabry machine parse error")?;

    if util::is_debug() {
        println!("{}", serde_json::to_string_pretty(&result.state)?);
    }

    let options_finder = options_finder::OptionsFinder::new(result, include_descriptions);
    let opts = options_finder.options(last_token)?;

    for opt in &opts.options {
        match opt.desc.as_ref() {
            Some(desc) => println!("{}	{}", opt.value, desc),
            None => println!("{}", opt.value),
        }
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
pub fn run_as_compline(compline: &str, comppoint: &str, include_descriptions: bool) -> Result<()> {
    let comppoint = comppoint.parse::<usize>().wrap_err_with(|| eyre!("Invalid compoint: {}", comppoint))?;

    let tokenized_result = shell_tokenizer::split_with_comppoint(compline, comppoint).wrap_err_with(|| eyre!("Failed to split compline {} on comppoint {}", compline, comppoint))?;

    let args = tokenized_result.arguments;
    let last_arg = tokenized_result.last_argument;

    let config_file = config_finder::find_tabry_config(&tokenized_result.command_basename)?;
    let compiled_config_file = cached_jsons::resolve_and_compile_cache_file(&config_file)?;

    print_options(&compiled_config_file, &args[..], &last_arg, include_descriptions)?;
    Ok(())
}

pub fn compile() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let tabry_conf = lang::compile(&input)?;
    let json = serde_json::to_string_pretty(&tabry_conf)?;
    print!("{}", json);
    Ok(())
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
pub fn bash(imports_path: Option<&str>, no_auto: bool, uniq_fn_id: Option<&str>) {
    if let Some(path) = imports_path {
        println!("_tabry_imports_path={}", escape(path));
    }
    let fn_id: &str = uniq_fn_id.unwrap_or("");
    println!("_tabry_executable={}", escaped_exe());

    print!("{}", TABRY_BASH_SH.replace("{{UNIQ_FN_ID}}", fn_id));

    if !no_auto {
        // TODO name things consistently between fish + bash
        println!("_tabry_complete_all{}", fn_id);
    }
}

const TABRY_ZSH_SH: &str = include_str!("../../shell/tabry_zsh.sh");
pub fn zsh(imports_path: Option<&str>, no_auto: bool, uniq_fn_id: Option<&str>) {
    if let Some(path) = imports_path {
        println!("_tabry_imports_path={}", escape(path));
    }
    let fn_id: &str = uniq_fn_id.unwrap_or("");
    println!("_tabry_executable={}", escaped_exe());
    print!("{}", TABRY_ZSH_SH.replace("{{UNIQ_FN_ID}}", fn_id));

    if !no_auto {
        // TODO name things consistently between fish + zsh
        println!("_tabry_complete_all{}", fn_id);
    }
}

const TABRY_FISH_SH: &str = include_str!("../../shell/tabry_fish.fish");
pub fn fish(imports_path: Option<&str>, no_auto: bool, uniq_fn_id: Option<&str>) {
    if let Some(path) = imports_path {
        println!("set -x TABRY_IMPORT_PATH {}", escape(path));
    }

    let fn_id: &str = uniq_fn_id.unwrap_or("");

    println!("set -x _tabry_executable{} {}", fn_id, escaped_exe());

    print!("{}", TABRY_FISH_SH.replace("{{UNIQ_FN_ID}}", fn_id));

    if !no_auto {
        println!("tabry_completion_init_all{}", fn_id);
    }
}
