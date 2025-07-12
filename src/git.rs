use std::{fs, path::Path};

use anyhow::{Context, Result};
use git2::Repository;

pub fn get_branches(path: String) -> Result<()> {
    let l_branches = get_branches_type(path.to_owned(), git2::BranchType::Local)?;
    let r_branches = get_branches_type(path.to_owned(), git2::BranchType::Remote)?;
    // let selected = select(input)?;
    println!("local branches: \n\n{}\n", l_branches);
    println!("remote branches: \n\n{}\n", r_branches);
    Ok(())
}

fn get_branches_type(path: String, branch_type: git2::BranchType) -> Result<String> {
    // open repo
    let repo = Repository::open(path).context("failed to open git repo")?;

    // get remote branches
    let branches = repo
        .branches(Some(branch_type))
        .context("failed to get branches")?;

    let mut input: String = "".to_owned();
    let mut first_run: bool = true;

    branches.for_each(|branch| match branch {
        Ok(br) => {
            match br.0.name().unwrap() {
                Some(b) => {
                    if b == "origin/HEAD".to_owned() {
                        return;
                    };
                    match first_run {
                        true => first_run = false,
                        false => input.push_str("\n"),
                    }
                    input.push_str(b.trim_start_matches("origin/"))
                }
                None => (),
            };
        }
        Err(e) => panic!("failed to get branch: {}", e),
    });

    Ok(input)
}

pub fn create_git_worktree(path_str: &str, branch_name: &str) -> Result<()> {
    let path = fs::canonicalize(Path::new(path_str))?;
    // Create new folder one level up from provided path
    let parent_dir = path.parent().context("failed to get parent directory")?;
    let folder_base_name = path.file_name().context("failed to get base name")?;
    let base_name = match folder_base_name.to_str() {
        Some(n) => n,
        None => "",
    };
    let worktree_name = format!("{}-{}", base_name, branch_name);
    let new_folder = parent_dir.join(worktree_name.to_owned());

    fs::create_dir_all(&new_folder)
        .with_context(|| format!("Failed to create directory: {:?}", new_folder))?;

    // open repo
    let bare_path = path.to_owned().join(".git");
    let repo = Repository::open_bare(bare_path).context("failed to open git repo")?;

    // check if the branch already exists
    match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(_) => {
            println!("Branch '{}' already exists.", branch_name);
        }
        Err(_) => {
            let head = repo.head().context("failed to get head")?;

            let peel = head.peel_to_commit().context("failed to peel to commit")?;

            let commit = repo
                .find_commit(peel.id())
                .context("failed to find commit")?;

            // create a new branch
            repo.branch(branch_name, &commit, false)
                .with_context(|| format!("failed to create branch '{}'", branch_name))?;
            println!("Branch '{}' created.", branch_name);
        }
    }

    // lookup worktree first
    let mut new_tree = true;
    let trees = repo.worktrees().context("failed to get repo's worktrees")?;
    trees.iter().for_each(|tree| {
        let t = match tree {
            Some(t) => t,
            None => "",
        };
        println!("{}", t);
        if t == worktree_name {
            new_tree = false
        }
    });

    if new_tree {
        // create the worktree
        let worktree = repo
            .worktree(&worktree_name, &new_folder, None)
            .with_context(|| format!("failed to create worktree in {:?}", new_folder))?;
        println!("Worktree created at {:?}", worktree.path());
    } else {
        println!("Already exists, idk what to do now...")
    }

    Ok(())
}
