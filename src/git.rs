use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};
use git2::Repository;

pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub is_dirty: bool,
}

pub fn get_worktrees(repo_path: &str) -> Result<Vec<WorktreeInfo>> {
    // First, find the main repository path
    let main_repo_path = find_main_repo_path(repo_path)?;

    let repo = Repository::open(&main_repo_path).context("failed to open git repo")?;
    let worktrees = repo.worktrees().context("failed to get worktrees")?;
    let mut worktree_infos = Vec::new();

    // Add the main repository as the first worktree
    let main_path = fs::canonicalize(&main_repo_path)
        .with_context(|| format!("Failed to canonicalize path: {}", main_repo_path))?
        .to_string_lossy()
        .to_string();
    let main_branch =
        get_worktree_branch(&main_repo_path).unwrap_or_else(|_| "unknown".to_string());
    let main_dirty = is_worktree_dirty(&main_repo_path).unwrap_or(false);
    worktree_infos.push(WorktreeInfo {
        name: "main".to_string(),
        path: main_path,
        branch: main_branch,
        is_dirty: main_dirty,
    });

    // Add additional worktrees
    for i in 0..worktrees.len() {
        if let Some(name) = worktrees.get(i) {
            // Try to get the worktree path
            let worktree_path = get_worktree_path(&main_repo_path, name);
            let branch = worktree_path
                .as_ref()
                .and_then(|p| get_worktree_branch(p).ok())
                .unwrap_or_else(|| "unknown".to_string());
            let is_dirty = worktree_path
                .as_ref()
                .map(|p| is_worktree_dirty(p).unwrap_or(false))
                .unwrap_or(false);
            worktree_infos.push(WorktreeInfo {
                name: name.to_string(),
                path: worktree_path.unwrap_or_else(|| name.to_string()),
                branch,
                is_dirty,
            });
        }
    }
    Ok(worktree_infos)
}

/// Find the main repository path from any worktree or the main repo itself
fn find_main_repo_path(path: &str) -> Result<String> {
    let path =
        fs::canonicalize(path).with_context(|| format!("Failed to canonicalize path: {}", path))?;

    // Check if this is a worktree (has .git file)
    let git_file_path = path.join(".git");
    if git_file_path.is_file() {
        // This is a worktree, read the gitdir from the .git file
        let git_content = fs::read_to_string(&git_file_path)
            .with_context(|| format!("Failed to read .git file at {git_file_path:?}"))?;
        if let Some(gitdir) = git_content.strip_prefix("gitdir: ") {
            let gitdir_path = gitdir.trim();
            // The gitdir points to the worktree directory in the main repo
            // We need to go up two levels to get to the main repo:
            // .git/worktrees/worktree-name/ -> .git/worktrees/ -> .git/ -> main-repo
            let worktree_dir = Path::new(gitdir_path)
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid gitdir path: {}", gitdir_path))?;
            let worktrees_dir = worktree_dir.parent().ok_or_else(|| {
                anyhow::anyhow!("Invalid worktree directory path: {:?}", worktree_dir)
            })?;
            let main_repo = worktrees_dir.parent().ok_or_else(|| {
                anyhow::anyhow!("Invalid worktrees directory path: {:?}", worktrees_dir)
            })?;
            Ok(main_repo.to_string_lossy().to_string())
        } else {
            // Fallback to the original path
            Ok(path.to_string_lossy().to_string())
        }
    } else {
        // This is already the main repository
        Ok(path.to_string_lossy().to_string())
    }
}

/// Try to get the worktree path by looking for a sibling directory to the repo with the worktree name
fn get_worktree_path(repo_path: &str, worktree_name: &str) -> Option<String> {
    let repo_dir = match fs::canonicalize(repo_path) {
        Ok(path) => path,
        Err(_) => return None,
    };
    let parent = repo_dir.parent()?;
    let candidate = parent.join(worktree_name);
    if candidate.exists() {
        // Return canonicalized (absolute) path
        match fs::canonicalize(&candidate) {
            Ok(canonical_path) => Some(canonical_path.to_string_lossy().to_string()),
            Err(_) => Some(candidate.to_string_lossy().to_string()),
        }
    } else {
        None
    }
}

pub fn get_branches(repo_path: &str) -> Result<(Vec<String>, Vec<String>)> {
    let repo = Repository::open(repo_path).context("failed to open git repo")?;
    let local_branches = get_branches_type(&repo, git2::BranchType::Local)?;
    let remote_branches = get_branches_type(&repo, git2::BranchType::Remote)?;
    Ok((local_branches, remote_branches))
}

fn get_branches_type(repo: &Repository, branch_type: git2::BranchType) -> Result<Vec<String>> {
    let branches = repo
        .branches(Some(branch_type))
        .context("failed to get branches")?;
    let mut branch_names = Vec::new();
    for branch in branches {
        let (branch_ref, _) = branch?;
        if let Some(name) = branch_ref.name()? {
            if name != "origin/HEAD" {
                let clean_name = name.trim_start_matches("origin/");
                branch_names.push(clean_name.to_string());
            }
        }
    }
    Ok(branch_names)
}

pub fn create_worktree(repo_path: &str, branch_name: &str) -> Result<String> {
    let path = fs::canonicalize(Path::new(repo_path))?;
    let parent_dir = path.parent().context("failed to get parent directory")?;

    // Get repo name from path
    let repo_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("repo");

    // Use the format ../$REPO_NAME-$BRANCH_NAME
    // Sanitize branch name by replacing forward slashes with hyphens
    let sanitized_branch_name = branch_name.replace('/', "-");
    let worktree_name = format!("{repo_name}-{sanitized_branch_name}");
    let new_folder = parent_dir.join(&worktree_name);

    println!("Creating worktree with name: '{worktree_name}'");
    println!("Worktree directory: {new_folder:?}");

    // Check if worktree already exists
    if new_folder.exists() {
        return Err(anyhow::anyhow!(
            "Worktree directory '{}' already exists. Please remove it first or use a different branch name.",
            worktree_name
        ));
    }

    // Open the repository
    let repo = Repository::open(repo_path).context("failed to open git repo")?;

    // Check if worktree already exists in the repository
    let existing_worktrees = repo.worktrees().context("failed to get worktrees")?;
    for i in 0..existing_worktrees.len() {
        if let Some(name) = existing_worktrees.get(i) {
            if name == worktree_name {
                return Err(anyhow::anyhow!(
                    "Worktree '{}' already exists in the repository. Please remove it first.",
                    worktree_name
                ));
            }
        }
    }

    // Check if branch exists, create if it doesn't
    let branch_exists = repo
        .find_branch(branch_name, git2::BranchType::Local)
        .is_ok();
    if !branch_exists {
        let head = repo.head().context("failed to get head")?;
        let commit = head.peel_to_commit().context("failed to peel to commit")?;
        match repo.branch(branch_name, &commit, false) {
            Ok(_) => println!("Branch '{branch_name}' created."),
            Err(e) => {
                if e.code() == git2::ErrorCode::Exists {
                    println!("Branch '{branch_name}' already exists, using existing branch.",);
                } else {
                    return Err(anyhow::anyhow!(
                        "Failed to create branch '{}': {}",
                        branch_name,
                        e
                    ));
                }
            }
        }
    } else {
        println!("Branch '{branch_name}' already exists, using existing branch.",);
    }

    // Create the worktree using git2
    let opts = git2::WorktreeAddOptions::new();
    let _worktree = repo
        .worktree(&worktree_name, &new_folder, Some(&opts))
        .with_context(|| format!("Failed to create worktree '{worktree_name}'"))?;

    // Checkout the branch in the worktree
    let worktree_repo = Repository::open(&new_folder).context("failed to open worktree repo")?;
    let branch_ref = repo
        .find_branch(branch_name, git2::BranchType::Local)
        .with_context(|| format!("Failed to find branch '{branch_name}'"))?;
    let branch_oid = branch_ref
        .get()
        .target()
        .with_context(|| format!("Failed to get branch target for '{branch_name}'"))?;

    worktree_repo
        .checkout_tree(&worktree_repo.find_object(branch_oid, None)?, None)
        .with_context(|| format!("Failed to checkout branch '{branch_name}' in worktree"))?;

    // Set the HEAD to point to the branch
    worktree_repo
        .set_head(&format!("refs/heads/{branch_name}"))
        .with_context(|| format!("Failed to set HEAD for worktree '{worktree_name}'"))?;

    println!("Worktree created at {new_folder:?}");
    Ok(worktree_name)
}

pub fn remove_worktree(repo_path: &str, worktree_name: &str) -> Result<()> {
    // Find the worktree path
    let worktree_path = get_worktree_path(repo_path, worktree_name)
        .ok_or_else(|| anyhow::anyhow!("Could not find worktree path for {}", worktree_name))?;
    // Use git CLI to remove the worktree (git2 doesn't have direct worktree removal)
    let output = Command::new("git")
        .args(["worktree", "remove", "--force", &worktree_path])
        .output()
        .with_context(|| format!("Failed to run git worktree remove for {worktree_path}"))?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to remove worktree: {}\n{}",
            worktree_path,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    println!("Worktree '{worktree_name}' removed successfully");
    Ok(())
}

pub fn change_directory(path: &str) -> Result<()> {
    std::env::set_current_dir(path)
        .with_context(|| format!("failed to change directory to: {path}"))?;
    println!("Changed directory to: {path}");
    Ok(())
}

pub fn merge_worktrees(repo_path: &str, source: &str, target: &str) -> Result<()> {
    // Get worktree paths
    let source_path = get_worktree_path(repo_path, source)
        .ok_or_else(|| anyhow::anyhow!("Could not find worktree path for {}", source))?;
    let target_path = get_worktree_path(repo_path, target)
        .ok_or_else(|| anyhow::anyhow!("Could not find worktree path for {}", target))?;

    // Check if worktrees are dirty
    let source_dirty = is_worktree_dirty(&source_path)?;
    let target_dirty = is_worktree_dirty(&target_path)?;

    if source_dirty {
        return Err(anyhow::anyhow!(
            "Source worktree '{}' has uncommitted changes. Please commit or stash them first.",
            source
        ));
    }

    if target_dirty {
        return Err(anyhow::anyhow!(
            "Target worktree '{}' has uncommitted changes. Please commit or stash them first.",
            target
        ));
    }

    // Get branch names
    let source_branch = get_worktree_branch(&source_path)?;
    let target_branch = get_worktree_branch(&target_path)?;

    println!("Merging '{source}' ({source_dirty}) into '{target}' ({target_branch})",);

    // Switch to target worktree and merge source branch using git CLI
    let output = Command::new("git")
        .args(["merge", &source_branch])
        .current_dir(&target_path)
        .output()
        .with_context(|| format!("Failed to merge {source_branch} into {target_branch}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to merge worktrees: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        ));
    }

    println!("Successfully merged '{source}' into '{target}'");
    Ok(())
}

pub fn pull_all_worktrees(repo_path: &str) -> Result<()> {
    // Open the repository
    let repo = Repository::open(repo_path).context("failed to open git repo")?;

    // Fetch all remote branches using git2
    let remote_callbacks = git2::RemoteCallbacks::new();
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(remote_callbacks);

    // Get all remotes
    let remotes = repo.remotes().with_context(|| "Failed to get remotes")?;

    for name in remotes.iter().flatten() {
        let mut remote = repo
            .find_remote(name)
            .with_context(|| format!("Failed to find remote '{name}'"))?;
        remote
            .fetch(&[] as &[&str], Some(&mut fetch_options), None)
            .with_context(|| format!("Failed to fetch from remote '{name}'"))?;
        println!("Fetched from remote '{name}'");
    }

    println!("Fetched all remote branches");

    // Get all worktrees
    let worktrees = get_worktrees(repo_path)?;

    for worktree in &worktrees {
        let worktree_path = &worktree.path;

        // Check if worktree is dirty
        if worktree.is_dirty {
            println!(
                "Skipping worktree '{}' - has uncommitted changes",
                worktree.name
            );
            continue;
        }

        // Get the branch name
        let branch_name = &worktree.branch;

        // Check if there are updates available
        let status_output = Command::new("git")
            .args(["status", "--porcelain", "--branch"])
            .current_dir(worktree_path)
            .output()
            .with_context(|| format!("Failed to check status for worktree {}", worktree.name))?;

        let status_str = String::from_utf8_lossy(&status_output.stdout);

        // Check if behind remote
        if status_str.contains("[behind") {
            println!(
                "Pulling updates for worktree '{}' (branch: {})",
                worktree.name, branch_name
            );

            let pull_output = Command::new("git")
                .args(["pull"])
                .current_dir(worktree_path)
                .output()
                .with_context(|| format!("Failed to pull for worktree {}", worktree.name))?;

            if pull_output.status.success() {
                println!(
                    "Successfully pulled updates for worktree '{}'",
                    worktree.name
                );
            } else {
                println!(
                    "Failed to pull updates for worktree '{}': {}",
                    worktree.name,
                    String::from_utf8_lossy(&pull_output.stderr)
                );
            }
        } else {
            println!("Worktree '{}' is up to date", worktree.name);
        }
    }

    Ok(())
}

fn get_worktree_branch(worktree_path: &str) -> Result<String> {
    // For worktrees, the .git is a file that points to the worktree directory in the main repo
    let git_file_path = Path::new(worktree_path).join(".git");
    if git_file_path.is_file() {
        // This is a worktree, read the gitdir from the .git file
        let git_content = fs::read_to_string(&git_file_path)
            .with_context(|| format!("Failed to read .git file at {git_file_path:?}"))?;
        if let Some(gitdir) = git_content.strip_prefix("gitdir: ") {
            let gitdir_path = gitdir.trim();
            let head_path = Path::new(gitdir_path).join("HEAD");
            let head_content = fs::read_to_string(&head_path)
                .with_context(|| format!("Failed to read HEAD file at {head_path:?}"))?;
            if let Some(stripped) = head_content.strip_prefix("ref: refs/heads/") {
                Ok(stripped.trim().to_string())
            } else {
                Ok("detached".to_string())
            }
        } else {
            Ok("unknown".to_string())
        }
    } else {
        // This is a regular repository, read .git/HEAD directly
        let head_path = Path::new(worktree_path).join(".git/HEAD");
        let head_content = fs::read_to_string(&head_path)
            .with_context(|| format!("Failed to read HEAD file at {head_path:?}"))?;
        if let Some(stripped) = head_content.strip_prefix("ref: refs/heads/") {
            Ok(stripped.trim().to_string())
        } else {
            Ok("detached".to_string())
        }
    }
}

fn is_worktree_dirty(worktree_path: &str) -> Result<bool> {
    let repo = Repository::open(worktree_path).context("failed to open worktree repo")?;

    // Check if there are any changes in the working directory
    let mut options = git2::StatusOptions::new();
    options.include_untracked(true);
    options.include_ignored(false);

    let statuses = repo
        .statuses(Some(&mut options))
        .with_context(|| "Failed to get status")?;

    Ok(!statuses.is_empty())
}

pub fn get_worktree_commit_hash(worktree_path: &str) -> Result<String> {
    let repo = Repository::open(worktree_path).context("failed to open worktree repo")?;
    let head = repo.head().context("failed to get head")?;
    let commit = head.peel_to_commit().context("failed to peel to commit")?;
    Ok(commit.id().to_string()[..8].to_string()) // Return first 8 characters of commit hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_worktree_branch_head_branch() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        let head_path = git_dir.join("HEAD");
        fs::write(&head_path, "ref: refs/heads/feature/test\n").unwrap();
        let branch = get_worktree_branch(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(branch, "feature/test");
    }

    #[test]
    fn test_get_worktree_branch_head_detached() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        let head_path = git_dir.join("HEAD");
        fs::write(&head_path, "e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1\n").unwrap();
        let branch = get_worktree_branch(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(branch, "detached");
    }

    #[test]
    fn test_is_worktree_dirty_clean() {
        let dir = tempdir().unwrap();

        // Initialize a git repository
        let output = Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to initialize git repository");

        if !output.status.success() {
            panic!(
                "Failed to initialize git repository: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Create an initial commit to make the repository valid
        let _output = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to set git config");

        let _output = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to set git config");

        // Create a file and commit it
        fs::write(dir.path().join("README.md"), "# Test Repository\n").unwrap();

        let _output = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to add file to git");

        let _output = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to commit to git");

        // Now test the dirty check - should be clean after commit
        let dirty = is_worktree_dirty(dir.path().to_str().unwrap()).unwrap();
        assert!(!dirty);
    }
}
