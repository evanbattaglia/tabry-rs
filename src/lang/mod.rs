mod lexer;
mod parser;
mod compiler;

use winnow::Parser;

pub fn compile(tabry_file_str: &str) -> crate::core::config::TabryConf {
    let tokens = lexer::lex.parse(tabry_file_str).unwrap();
    let parse_tree = parser::parse_tabry.parse(&tokens).unwrap();
    compiler::compile(parse_tree)
}



