use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::combinator::preceded;
use winnow::combinator::opt;
use winnow::PResult;
use winnow::Parser;
use winnow::{
    combinator::repeat,
    combinator::seq,
    error::StrContext,
    error::StrContextValue,
    token::any,
};

use super::lexer::Token;
// TODO errors are still hard to figure out, doesn't seem like the context() calls are doing much
// TODO () in args:
//   arg (foo bar ok)

// Takes lex tokens and produces a TabryFile, a parse tree.
// This is a representation of the tabry file that is close to the original source.

// TODO: in this parse tree, anything that comes from Lexer as &'a str, make it a &'a str ehre too
// (instead of a copied String)
// although I'm not sure it's worth the headache...
//--- end lexer---

// Raw tokens / building blocks

// I'm not sure if there's a better way to do all this...
fn parse_identifier<'a>(i: &mut &'a [Token]) -> PResult<&'a str> {
    any
        .verify(|t| matches!(t, Token::Identifier(_)))
        .context(StrContext::Expected(StrContextValue::Description("identifier")))
        .parse_next(i)
        .map(|t| match t {
            Token::Identifier(s) => s,
            _ => unreachable!(),
        })
}

// Matches Identifier, IdentifierWithAliases, String.
// foo -> name "foo"
// foo,bar,waz -> name "foo", aliases "bar" and "waz"
// "foo,bar" -> name "foo,bar"
fn parse_identifier_and_aliases<'a>(i: &mut &'a [Token]) -> PResult<NameAndAliases> {
    let id_and_aliases = any
        .verify(|t|
            matches!(t,
                Token::Identifier(_)
                | Token::IdentifierWithAliases(_)
                | Token::String(_)
            )
        )
        .context(StrContext::Expected(StrContextValue::Description("identifier/string or identifier with aliases")))
        .parse_next(i)?;

    let (name, aliases) = match id_and_aliases {
      Token::Identifier(s) => (s.to_string(), vec![]),
      Token::String(s) => (s, vec![]),
      Token::IdentifierWithAliases(v) => {
        let name = v.get(0).unwrap().to_string();
        let aliases = v[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();
        (name, aliases)
      },
      _ => unreachable!(),
    };

    Ok(NameAndAliases { name, aliases })
}

fn parse_string_literal<'a>(i: &mut &'a [Token]) -> PResult<String> {
    any
        .verify(|t| matches!(t, Token::String(_)))
        .context(StrContext::Expected(StrContextValue::Description("string literal")))
        .parse_next(i)
        .map(|t| match t {
            Token::String(s) => s,
            _ => unreachable!(),
        })
}

fn parse_at_identifier<'a>(i: &mut &'a [Token]) -> PResult<&'a str> {
    any
        .verify(|t| matches!(t, Token::AtIdentifier(_)))
        .context(StrContext::Expected(StrContextValue::Description("at identifier")))
        .parse_next(i)
        .map(|t| match t {
            Token::AtIdentifier(s) => s,
            _ => unreachable!(),
        })
}

fn parse_at_identifiers<'a>(i: &mut &'a [Token]) -> PResult<Vec<&'a str>> {
    repeat(0.., parse_at_identifier).parse_next(i)
}

#[derive(Clone, Debug)]
pub struct TabryFile {
    pub statements: Vec<Statement>
}

// =========== SIMPLE STATEMENTS (CAN'T TAKE A BLOCK) ===========

#[derive(Clone, Debug)]
pub struct CmdStatement {
    pub name: String,
}

fn parse_cmd_statement<'a>(i: &mut &'a [Token]) -> PResult<CmdStatement> {
    let mut parser = preceded(Token::Identifier("cmd"), parse_identifier);
    let name = parser.parse_next(i)?;
    Ok(CmdStatement { name: name.to_string() })
}

#[derive(Clone, Debug)]
pub struct DescStatement {
    pub desc: String,
}

fn parse_desc_statement<'a>(i: &mut &'a [Token]) -> PResult<DescStatement> {
    let mut parser = preceded(Token::Identifier("desc"), parse_string_literal);
    let desc = parser.parse_next(i)?;
    Ok(DescStatement { desc })
}

#[derive(Clone, Debug)]
pub struct TitleStatement {
    pub title: String,
}

fn parse_title_statement<'a>(i: &mut &'a [Token]) -> PResult<TitleStatement> {
    let mut parser = preceded(Token::Identifier("title"), parse_string_literal);
    let title = parser.parse_next(i)?;
    Ok(TitleStatement { title })
}

#[derive(Clone, Debug)]
pub struct IncludeStatement {
    pub includes: Vec<String>,
}

fn parse_include_statement<'a>(i: &mut &'a [Token]) -> PResult<IncludeStatement> {
    seq!(IncludeStatement {
        _: Token::Identifier("include"),
        includes: repeat(
            1..,
            parse_at_identifier.map(|s| s.to_string())
        )
    }).parse_next(i)
}

#[derive(Clone, Debug)]
pub enum OptsStatement {
    File,
    Dir,
    Const { values: Vec<String> },
    Shell { value: String },
    Delegate { value: String },
}

// TODO: optimization would be to do an Either<Vec<String>, Vec<Vec<String>> since most of the
// time. same for parse_identifier_or_list below.
// it's only 1
// TODO: should really handle strings "foo!","bar!"???? (requires lexer change)
// Matches: 'foo', '("foo")', '(a,b "c!" d)'
fn parse_identifier_and_aliases_or_list<'a>(i: &mut &'a [Token]) -> PResult<Vec<NameAndAliases>> {
    alt((
            parse_identifier_and_aliases.map(|v| vec![v]),
            delimited(
                Token::OpenParen,
                repeat(1.., parse_identifier_and_aliases),
                Token::CloseParen
            )
    )).parse_next(i)
}

// Matches: 'foo', '(foo bar)'
fn parse_identifier_or_list<'a>(i: &mut &'a [Token]) -> PResult<Vec<&'a str>> {
    alt((
            parse_identifier.map(|v| vec![v]),
            delimited(
                Token::OpenParen,
                repeat(1.., parse_identifier),
                Token::CloseParen
            )
    )).parse_next(i)
}

fn parse_opts_id_string_or_list<'a>(i: &mut &'a [Token]) -> PResult<Vec<String>> {
    alt((
        // opts const foo
        parse_string_literal.map(|s| vec![s]),
        // opts const "bar"
        parse_identifier.map(|s| vec![s.to_string()]),
        // opts const (foo "bar")
        delimited(
            Token::OpenParen,
            repeat(1.., alt((
                parse_string_literal,
                parse_identifier.map(|s| s.to_string())
            ))),
            Token::CloseParen
        )
    ))
    .context(StrContext::Label("opts const options"))
    .context(StrContext::Expected(StrContextValue::Description("identifier or string or (a \"b\" c) list of values")))
    .parse_next(i)
}

fn parse_opts_statement<'a>(i: &mut &'a [Token]) -> PResult<OptsStatement> {
    preceded(
        Token::Identifier("opts"),
        alt((
                Token::Identifier("file").map(|_| OptsStatement::File),
                Token::Identifier("dir").map(|_| OptsStatement::Dir),
                seq!(OptsStatement::Const {
                    _: Token::Identifier("const"),
                    values: parse_opts_id_string_or_list
                }),
                seq!(OptsStatement::Shell {
                    _: Token::Identifier("shell"),
                    value: parse_string_literal
                }),
                seq!(OptsStatement::Delegate {
                    _: Token::Identifier("delegate"),
                    value: parse_string_literal
                })
        ))
            .context(StrContext::Expected(StrContextValue::Description("opts type (file, dir, const, etc.) and value if appropriate")))
    ).parse_next(i)
}


// ============ SUB, FLAG, ARG, DEFARGS, DEFOPTS STATEMENTS (CAN TAKE A BLOCK) ============

#[derive(Clone, Debug)]
pub struct DefArgsStatement {
    pub name: String,
    pub statements: Vec<Statement>,
}

fn parse_defargs_statement<'a>(i: &mut &'a [Token]) -> PResult<DefArgsStatement> {
   seq!(DefArgsStatement {
      _: Token::Identifier("defargs"),
      name: parse_at_identifier
          .map(|s| s.to_string())
          .context(StrContext::Label("defargs at identifier")),
      _: Token::OpenBrace,
      statements: repeat(0.., parse_statement_inside_sub)
          .context(StrContext::Label("defargs block"))
          .context(StrContext::Expected(StrContextValue::Description("statements"))),
      _: Token::CloseBrace
    }).parse_next(i)
}

#[derive(Clone, Debug)]
pub struct DefOptsStatement {
    pub name: String,
    pub statements: Vec<Statement>,
}

fn parse_defopts_statement<'a>(i: &mut &'a [Token]) -> PResult<DefOptsStatement> {
   seq!(DefOptsStatement {
      _: Token::Identifier("defopts"),
      name: parse_at_identifier
          .map(|s| s.to_string())
          .context(StrContext::Label("defopts at identifier")),
      _: Token::OpenBrace,
      statements: repeat(0.., parse_statement_inside_arg)
          .context(StrContext::Label("defopts block"))
          .context(StrContext::Expected(StrContextValue::Description("statements"))),
      _: Token::CloseBrace
    }).parse_next(i)
}

#[derive(Clone, Debug)]
pub struct NameAndAliases {
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SubStatement {
    pub names_and_aliases: Vec<NameAndAliases>,
    pub description: Option<String>,
    pub includes: Vec<String>,
    pub statements: Vec<Statement>,
}

fn parse_sub_statement<'a>(i: &mut &'a[Token]) -> PResult<SubStatement> {
    let (names_and_aliases, description, includes, statements_in_block) = seq!(
      _: Token::Identifier("sub"),
      parse_identifier_and_aliases_or_list
          .context(StrContext::Label("sub name and aliases or list of sub names and aliases")),
      opt(parse_string_literal),
      parse_at_identifiers,
      opt(
          delimited(
              Token::OpenBrace,
              repeat(0.., parse_statement_inside_sub)
                  .context(StrContext::Label("sub block"))
                  .context(StrContext::Expected(StrContextValue::Description("statements"))),
              Token::CloseBrace
          )
      )
    ).parse_next(i)?;

    let includes = includes.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let statements = statements_in_block.unwrap_or_default();
    Ok(SubStatement { names_and_aliases, includes, description, statements })
}

#[derive(Clone, Debug)]
pub struct ArgStatement {
    pub names: Vec<String>,
    pub optional: bool,
    pub varargs: bool,
    pub description: Option<String>,
    pub includes: Vec<String>,
    pub statements: Vec<Statement>,
}

fn parse_arg_statement<'a>(i: &mut &'a[Token]) -> PResult<ArgStatement> {
    let (optional, varargs, names, description, includes, statements_in_block) = seq!(
        opt(Token::Identifier("opt")).map(|t| t.is_some()),
        alt((
                Token::Identifier("arg"),
                Token::Identifier("varargs")
        )).map(|t| t.is_identifier("varargs")),
        opt(parse_identifier_or_list.context(StrContext::Label("arg name"))),
        opt(parse_string_literal),
        parse_at_identifiers,
        opt(
            delimited(
                Token::OpenBrace,
                repeat(0.., parse_statement_inside_arg)
                .context(StrContext::Label("arg block"))
                .context(StrContext::Expected(StrContextValue::Description("statements"))),
                Token::CloseBrace
            )
        )
    ).parse_next(i)?;
    let names : Vec<String> = names.map(|v| v.iter().map(|id| id.to_string()).collect()).unwrap_or_default();
    let includes = includes.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let arg = ArgStatement {
        names,
        optional,
        varargs,
        includes,
        description,
        statements: statements_in_block.unwrap_or_default(),
    };
    Ok(arg)
}

#[derive(Clone, Debug)]
pub struct FlagStatement {
    pub names_and_aliases: Vec<NameAndAliases>,
    pub required: bool,
    pub has_arg: bool,
    pub description: Option<String>,
    pub includes: Vec<String>,
    pub statements: Vec<Statement>,
}

fn parse_flag_statement<'a>(i: &mut &'a[Token]) -> PResult<FlagStatement> {
    let (required, has_arg, names_and_aliases, description, includes, statements_in_block) = seq!(
      opt(Token::Identifier("reqd")).map(|t| t.is_some()),
      alt((
          Token::Identifier("flag"),
          Token::Identifier("flagarg")
      )).map(|t| t.is_identifier("flagarg")),
      parse_identifier_and_aliases_or_list
          .context(StrContext::Label("flag name and aliases or list of sub names and aliases")),
      opt(parse_string_literal),
      parse_at_identifiers,
      opt(
          delimited(
              Token::OpenBrace,
              repeat(0.., parse_statement_inside_flag)
                  .context(StrContext::Label("flag block"))
                  .context(StrContext::Expected(StrContextValue::Description("statements"))),
              Token::CloseBrace
          )
      )
    ).parse_next(i)?;
    // TODO dry up with parse_sub_statement
    let includes = includes.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    Ok(FlagStatement {
        names_and_aliases,
        has_arg,
        includes,
        description,
        required,
        statements: statements_in_block.unwrap_or_default(),
    })
}


// =========== STATEMENT ENUMS (BASED ON CONTEXT) ===========

#[derive(Clone, Debug)]
pub enum Statement {
    // in top-level, sub, arg, flag, defargs, defopts
    Include(IncludeStatement),

    // in top-level, sub, arg, flag
    Desc(DescStatement),

    // in top-level, sub, defargs
    Sub(SubStatement),
    Arg(ArgStatement),
    Flag(FlagStatement),

    // In arg, flag, defopts
    Opts(OptsStatement),

    // In arg
    Title(TitleStatement),

    // In top-level
    Cmd(CmdStatement),
    DefArgs(DefArgsStatement),
    DefOpts(DefOptsStatement),
}

fn parse_statement_inside_sub<'a>(i: &mut &'a [Token]) -> PResult<Statement> {
    alt((
        parse_desc_statement.map(Statement::Desc),
        parse_include_statement.map(Statement::Include),
        parse_sub_statement.map(Statement::Sub),
        parse_arg_statement.map(Statement::Arg),
        parse_flag_statement.map(Statement::Flag),
    ))
        .context(StrContext::Expected(StrContextValue::Description("desc, include, sub, arg, or flag statement")))
        .parse_next(i)
}

fn parse_statement_inside_arg<'a>(i: &mut &'a [Token]) -> PResult<Statement> {
    alt((
        parse_desc_statement.map(Statement::Desc),
        parse_include_statement.map(Statement::Include),
        parse_title_statement.map(Statement::Title),
        parse_opts_statement.map(Statement::Opts),
    ))
        .context(StrContext::Expected(StrContextValue::Description("desc, include, opts, or title statement")))
    .parse_next(i)
}

fn parse_statement_inside_flag<'a>(i: &mut &'a [Token]) -> PResult<Statement> {
    alt((
        parse_desc_statement.map(Statement::Desc),
        parse_include_statement.map(Statement::Include),
        parse_opts_statement.map(Statement::Opts),
    ))
        .context(StrContext::Expected(StrContextValue::Description("desc, include, or opts statement")))
    .parse_next(i)
}


fn parse_statement_top_level<'a>(i: &mut &'a [Token]) -> PResult<Statement> {
    alt((
        parse_cmd_statement.map(Statement::Cmd),
        parse_desc_statement.map(Statement::Desc),
        parse_include_statement.map(Statement::Include),
        parse_sub_statement.map(Statement::Sub),
        parse_arg_statement.map(Statement::Arg),
        parse_flag_statement.map(Statement::Flag),
        parse_defargs_statement.map(Statement::DefArgs),
        parse_defopts_statement.map(Statement::DefOpts),
    ))
        .context(StrContext::Expected(StrContextValue::Description("cmd, desc, include, sub, arg, or flag statement")))
    .parse_next(i)
}

// ==================================

pub fn parse_tabry(i: &mut &[Token]) -> PResult<TabryFile> {
    let statements = repeat(0.., parse_statement_top_level).parse_next(i)?;
    Ok(TabryFile { statements })
}

