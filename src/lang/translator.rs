use super::ast;
use crate::types;
use crate::config;

use std::collections::HashMap;

pub fn translate(ast: &ast::TabryFile) -> config::TabryConf {
    let mut cmd: Option<String> = None;
    let mut aliases = vec![];
    let mut args = vec![];
    let mut flags = vec![];
    let mut subs = vec![];

    for statement in &ast.statements {
        match statement {
            ast::TopLevelStatement::Cmd(stmt) => {
                cmd = Some(stmt.string.to_string())
            },
            ast::TopLevelStatement::Arg(stmt) => {
              args.push(translate_arg(stmt));
            },
            ast::TopLevelStatement::Flag(stmt) => {
            },
            ast::TopLevelStatement::Sub(stmt) => {
                let sub = translate_sub(stmt);
                subs.push(types::TabrySub::TabryConcreteSub(sub));
            },
            ast::TopLevelStatement::Desc(stmt) => {
            },
            ast::TopLevelStatement::Include(stmt) => {
            },
        }
    }
    
    config::TabryConf {
        cmd,
        main: types::TabryConcreteSub {
            name: None,
            aliases,
            description: None,
            args,
            flags,
            subs,
        },
        arg_includes: HashMap::new(),
        option_includes: HashMap::new()
    }
}

fn translate_sub(ast: &ast::SubStatement) -> types::TabryConcreteSub {
    unimplemented!()
}

fn translate_opt(ast: &ast::OptsStatement) -> types::TabryOpt {
    match ast {
        ast::OptsStatement::File(..) => types::TabryOpt::File,
        ast::OptsStatement::Dir(..) => types::TabryOpt::Dir,
        ast::OptsStatement::Const { value, .. } => types::TabryOpt::Const { value: value.to_string() },
        ast::OptsStatement::Shell { value, .. } => types::TabryOpt::Shell { value: value.to_string() },
    }
}

fn translate_arg(ast: &ast::ArgStatement) -> types::TabryArg {
  let mut options = vec![];
  let mut name = ast.name.as_ref().map(ToString::to_string);

  if let Some(block) = ast.block.as_ref() {
      for stmt in &block.statements {
          match stmt {
              ast::ArgBlockStatement::Opts(stmt) => {
                  options.push(translate_opt(&stmt));
              },
              ast::ArgBlockStatement::Include(stmt) => {
                  options.push(types::TabryOpt::Include { value: stmt.id.id() })
              },
              ast::ArgBlockStatement::Name(stmt) => {
                  name = Some(stmt.string.to_string())
              },
              ast::ArgBlockStatement::Title(stmt) => {
                  // TODO
              },
              ast::ArgBlockStatement::Desc(stmt) => {
                  // TODO
              },
          }
      }
  }

  types::TabryArg::TabryConcreteArg(
    types::TabryConcreteArg {
      name,
      options,
      optional: ast.opt_modifier.is_some(),
      varargs: match ast.arg_type {
        ast::ArgType::VarArgs(_) => true,
        _ => false
      }
    }
  )
}

