mod cli;
mod git;
mod menus;
// mod skim;

use anyhow::{Context, Result, ensure};
use clap::Parser;
use git2::Repository;
use std::path::Path;

use crate::git::create_git_worktree;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match &cli.command {
        Some(cli::Commands::List {}) => {
            // first arg path
            let path_arg = cli.config.path;

            // ensure the path exists before opening
            ensure!(
                Path::new(&path_arg).exists(),
                "need an existing repo, set --path or cd to git repo"
            );

            // open repo
            let repo = Repository::open(&path_arg).context("failed to open git repo")?;

            // props

            // get worktrees
            let trees = repo.worktrees().context("failed to get worktrees")?;
            if trees.len() > 0 {
                println!("worktrees: {}", trees.len());
                // print available trees
                trees
                    .iter()
                    .for_each(|tree| println!("{}", tree.expect("printing tree name failed")));
            } else {
                // get branches
                git::get_branches(path_arg)?;
            }
        }
        Some(cli::Commands::Add {}) => {
            // first arg path
            let path_arg = cli.config.path;

            // ensure the path exists before opening
            ensure!(
                Path::new(&path_arg).exists(),
                "need an existing repo, set --path or cd to git repo"
            );

            // create a folder for the set branch
            create_git_worktree(&path_arg, "temp-test")?;
        }
        None => {}
    }

    return Ok(());
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    cli::Cli::command().debug_assert();
}
