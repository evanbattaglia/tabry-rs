#[rust_sitter::grammar("tabry")]
mod grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct TabryFile {
        statements: Vec<TopLevelStatement>
    }

    #[derive(Debug)]
    pub enum TabryString {
        Unquoted {
            #[rust_sitter::leaf(pattern = r#"[A-Za-z_][a-zA-Z0-9_,-]*"#, transform = |v| v.parse().unwrap())]
            string: String
        },
        Quoted {
            #[rust_sitter::leaf(text = "\"")]
            _openquote: (),
            #[rust_sitter::leaf(pattern = r#"([^"\\]|\\"|\\\\)*"#, transform = |v| v.parse().unwrap())]
            string: String,
            #[rust_sitter::leaf(text = "\"")]
            _closequote: ()
        }
    }

    #[derive(Debug)]
    pub struct CmdStatement {
      #[rust_sitter::leaf(text = "cmd")]
      _cmd: (),
      string: TabryString,
    }

    #[derive(Debug)]
    pub enum TopLevelStatement {
        Cmd(CmdStatement),

        Arg(ArgStatement),
        // TopLevelStatement(TopLevelStatement)
    }

    #[derive(Debug)]
    pub enum BlockStatement {
        Arg(ArgStatement),
    }

    #[derive(Debug)]
    pub enum ArgType {
      Arg(
        #[rust_sitter::leaf(text = "arg")]
        ()
      ),
      VarArgs(
        #[rust_sitter::leaf(text = "varargs")]
        ()
      )
    }

    #[derive(Debug)]
    pub struct ArgStatement {
      #[rust_sitter::leaf(text = "opt")]
      opt_modifier: Option<() >,
      arg_type: ArgType,
      string: Option<TabryString>,
    }

    #[derive(Debug)]
    pub struct FlagStatement {
      #[rust_sitter::leaf(text = "flag")]
      _cmd: (),
      string: TabryString,
    }

    // );

    // #[derive(Debug)]
    // pub enum TopLevelStatement {
    //     Number(
    //         #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
    //         u32,
    //     ),
    //     Letter(
    //         #[rust_sitter::leaf(pattern = r"[a-z]", transform = |v| v.parse().unwrap())]
    //         String
    //     )
    // }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

fn main() {
 dbg!(grammar::parse("
cmd foo
varargs bar
opt arg
"));
}
