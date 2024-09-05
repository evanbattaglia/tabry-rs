mod compiler;
mod lexer;
mod parser;

use thiserror::Error;
use winnow::Parser;

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

pub fn compile(tabry_file_str: &str) -> Result<crate::core::config::TabryConf, LangError> {
    let tokens = lexer::lex
        .parse(tabry_file_str)
        .map_err(|e| LangError::LexError(e.to_string()))?;
    let parse_tree = parser::parse_tabry
        .parse(&tokens)
        .map_err(|e| LangError::ParseError(format!("{:#?}", e)))?;
    let res = compiler::compile(parse_tree)?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn test_integration_examples_from_language_reference() {
        for example in
            each_file_in_dir_with_extension("fixtures/examples_from_language_reference", "tabry")
        {
            let tabry_file_str = load_fixture_file_text(&example);
            let res = compile(&tabry_file_str).unwrap();
            let expected = load_fixture_file::<crate::core::config::TabryConf>(
                example.replace(".tabry", ".json").as_str(),
            );
            eprintln!("checking example {example}");
            assert_json_eq!(res, expected);
        }
    }
}
