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
}

#[derive(Subcommand)]
pub enum Commands {
    /// list worktrees
    List {},
    /// add worktree
    Add {},
}
