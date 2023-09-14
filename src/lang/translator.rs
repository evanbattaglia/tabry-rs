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
            }
            ast::TopLevelStatement::Arg(arg_statement) => {
              args.push(translate_arg(arg_statement));
            }
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

fn translate_arg(ast: &ast::ArgStatement) -> types::TabryArg {
  types::TabryArg::TabryConcreteArg(
    types::TabryConcreteArg {
      name: ast.name.as_ref().map(ToString::to_string),
      options: vec![],
      optional: false,
      varargs: match ast.arg_type {
        ast::ArgType::VarArgs(_) => true,
        _ => false
      }
    }
  )
}

