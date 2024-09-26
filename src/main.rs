use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

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
    /// `source <(tabry bash --no-auto); _tabry_complete_one_command mycmd`
    Bash {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use _tabry_complete_one_command for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,

        #[arg(long)]
        /// Unique function ID (useful for making sure multiple tabry versions don't conflict)
        uniq_fn_id: Option<String>,
    },

    /// Output completion script for zsh
    /// Usage in ~/.zsh_profile: `source <(tabry zsh)` or
    /// `source <(tabry zsh --no-auto); _tabry_complete_one_command mycmd`
    Zsh {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use _tabry_complete_one_command for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,

        #[arg(long)]
        /// Unique function ID (useful for making sure multiple tabry versions don't conflict)
        uniq_fn_id: Option<String>,
    },

    /// Output completion script for fish
    /// Usage in ~/.config/fish/config.fish: `tabry fish | source` or
    /// `tabry fish | source; tabry_completion_init mycmd`
    Fish {
        #[arg(long)]
        /// Do not automatically init completions for all tabry/JSON files in TABRY_IMPORT_PATH
        /// (manually use tabry_completion_init for each command if you use this option)
        no_auto: bool,

        #[arg(index=1)]
        /// Import path (colon-separated)
        import_path: Option<String>,

        #[arg(long)]
        /// Unique function ID (useful for making sure multiple tabry versions don't conflict)
        uniq_fn_id: Option<String>,
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

        /// Include descriptions in completions (for fish shell only)
        #[clap(long, short, action)]
        include_descriptions: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    use Subcommands::*;
    use tabry::app::*;
    let cli = Cli::parse();
    match cli.command {
        Complete { compline, comppoint, include_descriptions } => run_as_compline(&compline, &comppoint, include_descriptions)?,
        Compile => compile()?,
        Commands => commands(),
        Bash { import_path, no_auto, uniq_fn_id } => bash(import_path.as_deref(), no_auto, uniq_fn_id.as_deref()),
        Zsh { import_path, no_auto, uniq_fn_id } => zsh(import_path.as_deref(), no_auto, uniq_fn_id.as_deref()),
        Fish { import_path, no_auto, uniq_fn_id } => fish(import_path.as_deref(), no_auto, uniq_fn_id.as_deref()),
    }
    Ok(())
}
