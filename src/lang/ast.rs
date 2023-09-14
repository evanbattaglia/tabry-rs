#[rust_sitter::grammar("tabry")]
pub mod grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct TabryFile {
        pub statements: Vec<TopLevelStatement>
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
      pub string: TabryString,
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
      pub opt_modifier: Option<() >,
      pub arg_type: ArgType,
      pub name: Option<TabryString>,
    }

    #[derive(Debug)]
    pub struct FlagStatement {
      #[rust_sitter::leaf(text = "flag")]
      _cmd: (),
      pub string: TabryString,
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

pub use grammar::*;

impl std::fmt::Display for TabryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            grammar::TabryString::Unquoted { string } =>
                write!(f, "{}", string),
            grammar::TabryString::Quoted { string, .. } =>
                write!(f, "{}", string.replace("\\\"", "\"").replace("\\\\", "\\"))
        }
    }
}

