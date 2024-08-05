#[rust_sitter::grammar("tabry")]
pub mod grammar {
    // TOP LEVEL and "sub" statements
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct TabryFile {
        pub statements: Vec<TopLevelStatement>
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
        Desc(DescStatement),
        Arg(ArgStatement),
        Sub(SubStatement),
        Flag(FlagStatement),
        Include(IncludeStatement),
    }

    #[derive(Debug)]
    pub struct SubStatement {
      #[rust_sitter::leaf(text = "sub")]
      _sub: (),
      pub name: TabryString,
      pub block: Option<SubBlock>,
    }

    #[derive(Debug)]
    pub struct SubBlock {
      #[rust_sitter::leaf(text = "{")]
      _open: (),
      pub statements: Vec<SubBlockStatement>,
      #[rust_sitter::leaf(text = "}")]
      _close: (),
    }

    #[derive(Debug)]
    pub enum SubBlockStatement {
        Desc(DescStatement),
        Arg(ArgStatement),
        Sub(SubStatement),
        Flag(FlagStatement),
        Include(IncludeStatement),
    }

    // "opts" statements:

    #[derive(Debug)]
    pub enum OptsStatement {
      File(
        #[rust_sitter::leaf(text = "opts")]
        (),
        #[rust_sitter::leaf(text = "file")]
        ()
      ),
      Dir(
        #[rust_sitter::leaf(text = "opts")]
        (),
        #[rust_sitter::leaf(text = "dir")]
        ()
      ),
      Shell {
        #[rust_sitter::leaf(text = "opts")]
        _opts: (),
        #[rust_sitter::leaf(text = "shell")]
        _shell: (),
        value: TabryString,
      },
      Const {
        #[rust_sitter::leaf(text = "opts")]
        _opts: (),
        #[rust_sitter::leaf(text = "const")]
        _const: (),
        value: TabryString,
      },
      Delegate {
        #[rust_sitter::leaf(text = "opts")]
        _opts: (),
        #[rust_sitter::leaf(text = "delegate")]
        _delegate: (),
        value: TabryString,
      },
    }

    #[derive(Debug)]
    pub enum OptType {
      Const(
        #[rust_sitter::leaf(text = "const")]
        ()
      ),
      Delegate(
        #[rust_sitter::leaf(text = "delegate")]
        ()
      ),
      Shell(
        #[rust_sitter::leaf(text = "shell")]
        ()
      ),
      File(
        #[rust_sitter::leaf(text = "file")]
        ()
      ),
      Dir(
        #[rust_sitter::leaf(text = "dir")]
        ()
      ),
    }

    // "arg" statements

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
      pub opt_modifier: Option<()>,
      pub arg_type: ArgType,
      pub name: Option<TabryString>,
      pub includes: Vec<AtIdentifier>,
      pub block: Option<ArgBlock>,
    }

    #[derive(Debug)]
    pub struct ArgBlock {
      #[rust_sitter::leaf(text = "{")]
      _open: (),
      pub statements: Vec<ArgBlockStatement>,
      #[rust_sitter::leaf(text = "}")]
      _close: (),
    }

    #[derive(Debug)]
    pub enum ArgBlockStatement {
        Opts(OptsStatement),
        Include(IncludeStatement),
        Name(NameStatement),
        Title(TitleStatement),
        Desc(DescStatement),
    }

    // "flag" statements

    #[derive(Debug)]
    pub struct FlagStatement {
      #[rust_sitter::leaf(text = "reqd")]
      pub reqd_modifier: Option<()>,
      pub flag_type: FlagType,
      pub string: TabryString,
      pub includes: Vec<AtIdentifier>,
    }

    #[derive(Debug)]
    pub enum FlagType {
      Flag(
        #[rust_sitter::leaf(text = "flag")]
        ()
      ),
      FlagArg(
        #[rust_sitter::leaf(text = "flagarg")]
        ()
      )
    }

    #[derive(Debug)]
    pub enum FlagBlockStatement {
        Opts(OptsStatement),
        Include(IncludeStatement),
        Name(NameStatement),
        Title(TitleStatement),
        Desc(DescStatement),
    }

    // TODO genericize blocks
    #[derive(Debug)]
    pub struct FlagBlock {
      #[rust_sitter::leaf(text = "{")]
      _open: (),
      pub statements: Vec<FlagBlockStatement>,
      #[rust_sitter::leaf(text = "}")]
      _close: (),
    }


    // name, title, include -- can appear in multiple contexts

    #[derive(Debug)]
    pub struct NameStatement {
      #[rust_sitter::leaf(text = "name")]
      _name: (),
      pub string: TabryString,
    }

    #[derive(Debug)]
    pub struct TitleStatement {
      #[rust_sitter::leaf(text = "title")]
      _title: (),
      pub string: TabryString,
    }

    #[derive(Debug)]
    pub struct DescStatement {
      #[rust_sitter::leaf(text = "desc")]
      _desc: (),
      pub string: TabryString,
    }

    #[derive(Debug)]
    pub struct IncludeStatement {
      #[rust_sitter::leaf(text = "include")]
      _include: (),
      pub id: AtIdentifier,
    }


    // BUILDING BLOCKS

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
    pub struct AtIdentifier {
        #[rust_sitter::leaf(pattern = r#"@[a-zA-Z_][a-zA-Z0-9_-]*/"#, transform = |v| v.parse().unwrap())]
        pub id: String
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }

    #[rust_sitter::extra]
    struct Comment {
        #[rust_sitter::leaf(pattern = r"#.*")]
        _comment: (),
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

impl AtIdentifier {
    pub fn id(&self) -> String {
        self.id[0..].to_owned()
    }
}

