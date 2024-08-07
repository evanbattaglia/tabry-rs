mod lexer;
mod parser;
mod compiler;

use winnow::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LangError {
    #[error("lex error: {0}")]
    LexError(String),
    #[error("parse error: {0}")]
    ParseError(String),
    // #[error("parse error: {0}")]
    // ParseError(#[source] winnow::error::ParseError<&'static str, winnow::error::ContextError>),
    #[error("compile error: {0}")]
    CompileError(#[from] compiler::CompileError),
}

pub fn compile(
    tabry_file_str: &str
) -> Result<crate::core::config::TabryConf, LangError> {
    let tokens = lexer::lex.parse(tabry_file_str)
        .map_err(|e| LangError::LexError(e.to_string()))?;
    let parse_tree = parser::parse_tabry.parse(&tokens)
        .map_err(|e| LangError::ParseError(format!("{:#?}", e)))?;
    let res = compiler::compile(parse_tree)?;
    Ok(res)
}



