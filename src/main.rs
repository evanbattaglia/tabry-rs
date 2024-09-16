use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tabry")]
#[command(version = "0.0.1")]
#[command(about = "Tabry tab completion engine")]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Output completion script for bash
    /// Usage in ~/.bash_profile: `source <(tabry bash)` or
    /// `source <(tabry bash --no-auto); _tabry_rs_complete_one_command mycmd`
    Bash {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use _tabry_rs_complete_one_command for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,
    },

    /// Output completion script for zsh
    /// Usage in ~/.zsh_profile: `source <(tabry zsh)` or
    /// `source <(tabry zsh --no-auto); _tabry_rs_complete_one_command mycmd`
    Zsh {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use _tabry_rs_complete_one_command for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,
    },

    /// Output completion script for bash
    /// Usage in ~/.bash_profile: `tabry fish | source` or
    /// `tabry fish | source; tabry_completion_init mycmd`
    Fish {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use tabry_completion_init for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,
    },

    /// List commands for which there is a .tabry/.json file in TABRY_IMPORT_PATH
    Commands,

    /// Compile a tabry file to json (usually done automatically via tabry complete).
    /// Usage: tabry compile < [tabry file] > [json file]
    Compile,

    /// Return completions (usually used via shell script)
    Complete {
        /// TODO desc
        compline: String,
        /// TODO desc
        comppoint: String,
    },
}

fn main() -> anyhow::Result<()> {
    use Subcommands::*;
    use tabry::app::*;
    let cli = Cli::parse();
    match cli.command {
        Complete { compline, comppoint } => run_as_compline(&compline, &comppoint)?,
        Compile => compile()?,
        Commands => commands(),
        Bash { import_path, no_auto } => bash(import_path.as_deref(), no_auto),
        Zsh { import_path, no_auto } => zsh(import_path.as_deref(), no_auto),
        Fish { import_path, no_auto } => fish(import_path.as_deref(), no_auto),
    }
    Ok(())
}
