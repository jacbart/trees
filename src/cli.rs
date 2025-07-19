use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub config: Config,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args)]
pub struct Config {
    /// Set repo path
    #[arg(
        global = true,
        short, long,
        value_name = "PATH",
        default_value_t = String::from("./"),
    )]
    pub path: String,

    /// Output only directory paths (for shell integration)
    #[arg(long, global = true)]
    pub dir_only: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List worktrees
    List,
    /// Add a new worktree
    Add,
    /// Remove a worktree
    Rm,
    /// Merge two worktrees
    Merge,
    /// Pull updates for all worktrees
    Pull,
    /// Show shell integration script
    Shell {
        /// Shell type (zsh, bash, fish)
        shell: String,
    },
}
