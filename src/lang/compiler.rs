// Takes the parsed representaiton (parse tree)
// and constructs the compiled tabry config (JSON-compatible)

use std::collections::HashMap;

use thiserror::Error;

use super::parser;
use crate::core::config;
use crate::core::types;
use crate::core::util::is_debug;

#[inline(always)]
fn make_new_sub() -> types::TabryConcreteSub {
    // inline will optimize away the case in compile_sub where these default vec![]s are not
    // used... I think
    types::TabryConcreteSub {
        name: None,
        subs: vec![],
        args: vec![],
        flags: vec![],
        aliases: vec![],
        includes: vec![],
        description: None,
    }
}

// TODO it would be really nice to leave out empty vecs and 'false's in the JSON output
fn add_subs_from_sub_statement(subs: &mut Vec<types::TabrySub>, stmt: parser::SubStatement) {
    for parser::NameAndAliases { name, aliases } in stmt.names_and_aliases {
        let mut sub = make_new_sub();
        sub.name = Some(name);
        sub.aliases = aliases;
        // TODO potential optimization: don't clone any of these if only one. ESPECIALLY cloning
        // the whole statement. maybe I should pass a reference that can be copied, but would have
        // to change references all the way down
        sub.description.clone_from(&stmt.description);
        sub.includes.extend(stmt.includes.clone());
        for stmt_in_block in &stmt.statements {
            process_statement_inside_sub(&mut sub, stmt_in_block.clone());
        }
        subs.push(types::TabrySub::TabryConcreteSub(sub));
    }
}

fn add_opts(opts: &mut Vec<types::TabryOpt>, stmt: parser::OptsStatement) {
    match stmt {
        parser::OptsStatement::File => opts.push(types::TabryOpt::File),
        parser::OptsStatement::Dir => opts.push(types::TabryOpt::Dir),
        parser::OptsStatement::Const { values } => {
            for value in values {
                opts.push(types::TabryOpt::Const { value })
            }
        }
        parser::OptsStatement::Shell { value } => opts.push(types::TabryOpt::Shell { value }),
        parser::OptsStatement::Delegate { value } => opts.push(types::TabryOpt::Delegate { value }),
    }
}

fn add_include_opts(opts: &mut Vec<types::TabryOpt>, includes: Vec<String>) {
    for value in includes {
        opts.push(types::TabryOpt::Include { value });
    }
}

fn add_flags_from_flag_statement(flags: &mut Vec<types::TabryFlag>, stmt: parser::FlagStatement) {
    for parser::NameAndAliases { name, aliases } in stmt.names_and_aliases {
        let mut flag = types::TabryConcreteFlag {
            name,
            aliases,
            description: stmt.description.clone(),
            options: vec![],
            arg: stmt.has_arg,
            required: stmt.required,
        };
        // TODO: again, potential optimizations in this function -- don't clone if only one
        add_include_opts(&mut flag.options, stmt.includes.clone());
        for stmt_in_block in stmt.statements.clone() {
            match stmt_in_block {
                parser::Statement::Desc(desc_stmt) => {
                    if flag.description.is_some() {
                        // TODO errors for real, dedup with cmd
                        panic!("multiple desc statements found");
                    }
                    flag.description = Some(desc_stmt.desc);
                }
                parser::Statement::Opts(opts_stmt) => add_opts(&mut flag.options, opts_stmt),
                parser::Statement::Include(include_stmt) => {
                    add_include_opts(&mut flag.options, include_stmt.includes)
                }
                _ => unreachable!("unhandled statement in compile_flag: {:?}", stmt_in_block),
            }
        }
        flags.push(types::TabryFlag::TabryConcreteFlag(flag));
    }
}

fn make_arg(stmt: &parser::ArgStatement, name: Option<String>) -> types::TabryArg {
    let mut arg = types::TabryConcreteArg {
        name,
        description: stmt.description.clone(),
        varargs: stmt.varargs,
        optional: stmt.optional,
        options: vec![],
    };
    add_include_opts(&mut arg.options, stmt.includes.clone());
    for stmt_in_block in stmt.statements.clone() {
        match stmt_in_block {
            parser::Statement::Opts(opts_stmt) => add_opts(&mut arg.options, opts_stmt),
            parser::Statement::Include(include_stmt) => {
                add_include_opts(&mut arg.options, include_stmt.includes)
            }
            parser::Statement::Title(title_stmt) => {
                // TODO (not supported in types module yet)
                // for now this avoids the 'unused' warning for Title.title
                if is_debug() {
                    eprintln!("ignoring title: {:?}", title_stmt.title);
                }
            }
            parser::Statement::Desc(desc_stmt) => {
                if arg.description.is_some() {
                    // TODO errors for real, dedup with cmd
                    panic!("multiple desc statements found");
                }
                arg.description = Some(desc_stmt.desc);
            }
            _ => unreachable!("unhandled statement in compile_arg: {:?}", stmt_in_block),
        }
    }
    types::TabryArg::TabryConcreteArg(arg)
}

fn add_args_from_arg_statement(args: &mut Vec<types::TabryArg>, stmt: parser::ArgStatement) {
    if stmt.names.is_empty() {
        args.push(make_arg(&stmt, None));
    } else {
        for name in &stmt.names {
            // TODO lots of unnecessary duping to hack around borrow checker, I'm sure there are
            // better ways
            args.push(make_arg(&stmt, Some(name.to_string())));
        }
    }
}

fn process_statement_inside_sub_or_defargs(
    subs: &mut Vec<types::TabrySub>,
    args: &mut Vec<types::TabryArg>,
    flags: &mut Vec<types::TabryFlag>,
    includes: &mut Vec<String>,
    statement: parser::Statement,
) {
    match statement {
        parser::Statement::Sub(child_sub_stmt) => add_subs_from_sub_statement(subs, child_sub_stmt),
        parser::Statement::Arg(arg_stmt) => add_args_from_arg_statement(args, arg_stmt),
        parser::Statement::Flag(flag_stmt) => add_flags_from_flag_statement(flags, flag_stmt),
        parser::Statement::Include(include_stmt) => {
            includes.extend(include_stmt.includes);
        }
        _ => unreachable!(
            "unhandled statement in process_statement_inside_sub_or_defargs: {:?}",
            statement
        ),
    }
}

fn process_statement_inside_sub(sub: &mut types::TabryConcreteSub, statement: parser::Statement) {
    match statement {
        parser::Statement::Desc(desc) => sub.description = Some(desc.desc),
        _ => process_statement_inside_sub_or_defargs(
            &mut sub.subs,
            &mut sub.args,
            &mut sub.flags,
            &mut sub.includes,
            statement,
        ),
    }
}

fn compile_defargs(stmt: parser::DefArgsStatement) -> (String, types::TabryArgInclude) {
    let mut arg_include = types::TabryArgInclude {
        args: vec![],
        flags: vec![],
        subs: vec![],
        includes: vec![],
    };
    for statement in stmt.statements {
        process_statement_inside_sub_or_defargs(
            &mut arg_include.subs,
            &mut arg_include.args,
            &mut arg_include.flags,
            &mut arg_include.includes,
            statement,
        );
    }
    (stmt.name, arg_include)
}

fn compile_defopts(stmt: parser::DefOptsStatement) -> (String, Vec<types::TabryOpt>) {
    let mut opts: Vec<types::TabryOpt> = vec![];
    for stmt_in_block in stmt.statements {
        match stmt_in_block {
            parser::Statement::Opts(opts_stmt) => add_opts(&mut opts, opts_stmt),
            parser::Statement::Include(include_stmt) => {
                add_include_opts(&mut opts, include_stmt.includes)
            }
            _ => unreachable!("unhandled statement in compile_flag: {:?}", stmt_in_block),
        }
    }
    (stmt.name, opts)
}

// In the future I may provide line numbers in error messages, etc.
#[derive(Error, Debug)]
#[error("compile error: {msg}")]
pub struct CompileError {
    msg: String,
}

pub fn compile(tabry_file: parser::TabryFile) -> Result<config::TabryConf, CompileError> {
    let mut conf = config::TabryConf {
        main: make_new_sub(),
        cmd: None,
        arg_includes: HashMap::new(),
        option_includes: HashMap::new(),
    };

    for statement in tabry_file.statements {
        match statement {
            parser::Statement::DefArgs(def_args) => {
                let (name, arg_include) = compile_defargs(def_args);
                conf.arg_includes.insert(name, arg_include);
            }
            parser::Statement::DefOpts(def_opts) => {
                let (name, opt_include) = compile_defopts(def_opts);
                conf.option_includes.insert(name, opt_include);
            }
            parser::Statement::Cmd(cmd) => {
                if conf.cmd.is_some() {
                    return Err(CompileError {
                        msg: "multiple cmd statements found".to_string(),
                    });
                    // TODO errors for real
                }
                conf.cmd = Some(cmd.name);
            }
            _ => process_statement_inside_sub(&mut conf.main, statement),
        }
    }
    Ok(conf)
}
