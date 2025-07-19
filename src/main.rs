mod cli;
mod git;
mod tui;

use anyhow::{Result, ensure};
use clap::Parser;
use std::path::Path;

use crate::git::{
    change_directory, create_worktree, get_branches, get_worktree_commit_hash, get_worktrees,
    merge_worktrees, pull_all_worktrees, remove_worktree,
};
use crate::tui::TuiSelector;

const ZSH_SCRIPT: &str = r#"# Trees zsh integration
# Usage: eval "$(trees-bin shell zsh)"

trees() {
    local DIR
    local STATUS
    if [ $# -gt 0 ]; then
        case "$1" in
            add|rm|merge|pull|list|--help|-h|help|--version|-V|--dir-only)
                # These commands don't need --dir-only, run directly
                trees-bin "$@"
                STATUS=$?
                return $STATUS
                ;;
            *)
                # For other cases (like no args), try to get directory
                DIR=$(trees-bin "$@" --dir-only)
                STATUS=$?
                ;;
        esac
    else
        DIR=$(trees-bin --dir-only)
        STATUS=$?
    fi
    if [ -n "$DIR" ]; then
        \cd "$DIR"
    else
        ( exit $STATUS )
    fi
}

# Add completion for trees
_trees() {
    local curcontext="$curcontext" state line
    typeset -A opt_args
    
    _arguments -C \
        '1: :->cmds' \
        '*:: :->args'
    
    case "$state" in
        cmds)
            _values 'trees commands' \
                'add[Add a new worktree]' \
                'list[List worktrees]' \
                'rm[Remove a worktree]' \
                'merge[Merge two worktrees]' \
                'pull[Pull updates for all worktrees]'
            ;;
        args)
            case "$line[1]" in
                add|list|rm|merge)
                    # Could add more specific completion here
                    _files
                    ;;
            esac
            ;;
    esac
}

compdef _trees trees"#;

const BASH_SCRIPT: &str = r#"# Trees bash integration
# Usage: eval "$(trees-bin shell bash)"

trees() {
    local DIR
    local STATUS
    if [ $# -gt 0 ]; then
        case "$1" in
            add|rm|merge|pull|list|--help|-h|help|--version|-V|--dir-only)
                # These commands don't need --dir-only, run directly
                trees-bin "$@"
                STATUS=$?
                return $STATUS
                ;;
            *)
                # For other cases (like no args), try to get directory
                DIR=$(trees-bin "$@" --dir-only)
                STATUS=$?
                ;;
        esac
    else
        DIR=$(trees-bin --dir-only)
        STATUS=$?
    fi
    if [ -n "$DIR" ]; then
        \cd "$DIR"
    else
        ( exit $STATUS )
    fi
}

# Add completion for trees
_trees_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    opts="add list rm merge pull"
    
    if [[ ${cur} == * ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    fi
}

complete -F _trees_completion trees"#;

const FISH_SCRIPT: &str = r#"# Trees fish integration
# Usage: eval "$(trees-bin shell fish)"

function trees
    set DIR
    set STATUS
    if [ $argv[1] ]
        switch $argv[1]
            case "add" "rm" "merge" "pull" "list" "--help" "-h" "help" "--version" "-V" "--dir-only"
                # These commands don't need --dir-only, run directly
                trees-bin $argv
                set STATUS $status
                return $STATUS
            case "*"
                # For other cases (like no args), try to get directory
                set DIR (trees-bin $argv --dir-only)
                set STATUS $status
        end
    else
        set DIR (trees-bin --dir-only)
        set STATUS $status
    end
    if [ -n "$DIR" ]
        \cd "$DIR"
    else
        exit $STATUS
    end
end

# Add completion for trees
complete -c trees -f -a "add list rm merge pull" -d "Git worktree management"
complete -c trees -n "__fish_seen_subcommand_from add" -f -d "Add a new worktree"
complete -c trees -n "__fish_seen_subcommand_from list" -f -d "List worktrees"
complete -c trees -n "__fish_seen_subcommand_from rm" -f -d "Remove a worktree"
complete -c trees -n "__fish_seen_subcommand_from merge" -f -d "Merge two worktrees"
complete -c trees -n "__fish_seen_subcommand_from pull" -f -d "Pull updates for all worktrees""#;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let path_arg = &cli.config.path;

    ensure!(
        Path::new(path_arg).exists(),
        "need an existing repo, set --path or cd to git repo"
    );

    match &cli.command {
        Some(cli::Commands::List) => {
            let worktrees = get_worktrees(path_arg)?;

            if worktrees.is_empty() {
                if !cli.config.dir_only {
                    println!("No worktrees found");
                }
                return Ok(());
            }

            if cli.config.dir_only {
                // For shell integration, just output the first worktree path
                if let Some(worktree) = worktrees.first() {
                    println!("{}", worktree.path);
                }
            } else {
                // Format like git worktree list: /path/to/worktree commit-hash [branch-name]
                for worktree in &worktrees {
                    // Get the commit hash for the worktree
                    let commit_hash = get_worktree_commit_hash(&worktree.path)
                        .unwrap_or_else(|_| "unknown".to_string());
                    let branch_display = if worktree.branch == "detached" {
                        "".to_string()
                    } else {
                        format!(" [{}]", worktree.branch)
                    };
                    println!("{} {} {}", worktree.path, commit_hash, branch_display);
                }
            }
        }
        Some(cli::Commands::Add) => {
            // Use TUI to select or create branch
            let (local_branches, remote_branches) = get_branches(path_arg)?;

            let mut all_branches = Vec::new();
            all_branches.extend(local_branches.iter().map(|b| format!("local: {b}")));
            all_branches.extend(remote_branches.iter().map(|b| format!("remote: {b}")));
            all_branches.push("Create new branch".to_string());

            if let Some(selected) = TuiSelector::select_branch(&all_branches)? {
                let branch_name = if selected == "Create new branch" {
                    if let Some(new_branch) = TuiSelector::create_new_branch()? {
                        if !cli.config.dir_only {
                            println!("Creating new branch: {new_branch}");
                        }
                        new_branch
                    } else {
                        if !cli.config.dir_only {
                            println!("No branch name provided, exiting");
                        }
                        return Ok(());
                    }
                } else {
                    // Extract branch name from selection
                    let branch_name = selected.split(": ").nth(1).unwrap_or(&selected).to_string();
                    if !cli.config.dir_only {
                        println!("Selected branch: {branch_name}");
                    }
                    branch_name
                };

                if !cli.config.dir_only {
                    println!("Creating worktree for branch: {branch_name}");
                }
                let worktree_name = create_worktree(path_arg, &branch_name)?;

                // Get the worktree path and change to it
                let worktrees = get_worktrees(path_arg)?;
                if let Some(worktree) = worktrees.iter().find(|wt| wt.name == worktree_name) {
                    if cli.config.dir_only {
                        println!("{}", worktree.path);
                    } else {
                        change_directory(&worktree.path)?;
                    }
                }
            } else {
                if !cli.config.dir_only {
                    println!("No branch selected, exiting");
                }
            }
        }
        Some(cli::Commands::Rm) => {
            let worktrees = get_worktrees(path_arg)?;

            if worktrees.is_empty() {
                println!("No worktrees found");
                return Ok(());
            }

            let worktree_names: Vec<String> = worktrees
                .iter()
                .map(|wt| {
                    let status = if wt.is_dirty { " (dirty)" } else { "" };
                    format!("{} -> {} ({}){}", wt.name, wt.path, wt.branch, status)
                })
                .collect();

            if let Some(selected) = TuiSelector::select_worktree(&worktree_names)? {
                // Extract worktree name from selection
                if let Some(worktree_name) = selected.split(" -> ").next() {
                    if let Some(worktree) = worktrees.iter().find(|wt| wt.name == worktree_name) {
                        if worktree.is_dirty && !TuiSelector::confirm_deletion(worktree_name)? {
                            println!("Deletion cancelled");
                            return Ok(());
                        }
                        remove_worktree(path_arg, worktree_name)?;
                    }
                }
            }
        }
        Some(cli::Commands::Merge) => {
            let worktrees = get_worktrees(path_arg)?;

            if worktrees.is_empty() {
                println!("No worktrees found");
                return Ok(());
            }

            let worktree_names: Vec<String> = worktrees
                .iter()
                .map(|wt| {
                    let status = if wt.is_dirty { " (dirty)" } else { "" };
                    format!("{} -> {} ({}){}", wt.name, wt.path, wt.branch, status)
                })
                .collect();

            println!("Select source worktree (to merge FROM):");
            let source_selected = TuiSelector::select_worktree(&worktree_names)?;
            let source_name = if let Some(selected) = source_selected {
                if let Some(name) = selected.split(" -> ").next() {
                    name.to_string()
                } else {
                    println!("No source worktree selected, exiting");
                    return Ok(());
                }
            } else {
                println!("No source worktree selected, exiting");
                return Ok(());
            };

            println!("Select target worktree (to merge INTO):");
            let target_selected = TuiSelector::select_worktree(&worktree_names)?;
            let target_name = if let Some(selected) = target_selected {
                if let Some(name) = selected.split(" -> ").next() {
                    name.to_string()
                } else {
                    println!("No target worktree selected, exiting");
                    return Ok(());
                }
            } else {
                println!("No target worktree selected, exiting");
                return Ok(());
            };

            merge_worktrees(path_arg, &source_name, &target_name)?;
        }
        Some(cli::Commands::Pull) => {
            pull_all_worktrees(path_arg)?;
        }
        Some(cli::Commands::Shell { shell }) => match shell.as_str() {
            "zsh" => {
                println!("{}", ZSH_SCRIPT);
            }
            "bash" => {
                println!("{}", BASH_SCRIPT);
            }
            "fish" => {
                println!("{}", FISH_SCRIPT);
            }
            _ => {
                anyhow::bail!(
                    "Unsupported shell: {}. Supported shells: zsh, bash, fish",
                    shell
                );
            }
        },
        None => {
            // Default behavior: show worktrees and allow selection or creation
            let worktrees = get_worktrees(path_arg)?;

            if worktrees.is_empty() {
                // Don't output anything when no worktrees found for shell integration
                return Ok(());
            }

            // Get current directory to filter out the main repo if we're in it
            let current_dir = std::env::current_dir()
                .unwrap_or_else(|_| Path::new(".").to_path_buf())
                .canonicalize()
                .unwrap_or_else(|_| Path::new(".").to_path_buf());
            let current_dir_str = current_dir.to_string_lossy().to_string();

            // Filter out the current directory from available worktrees
            let available_worktrees: Vec<_> = worktrees
                .iter()
                .filter(|wt| wt.path != current_dir_str)
                .collect();

            if available_worktrees.is_empty() {
                // No other worktrees available, don't output anything for shell integration
                return Ok(());
            }

            if cli.config.dir_only {
                // For shell integration, just output the first available worktree path
                if let Some(worktree) = available_worktrees.first() {
                    println!("{}", worktree.path);
                }
                return Ok(());
            }

            let mut options = Vec::new();

            // Add existing worktrees (excluding current directory)
            for worktree in &available_worktrees {
                let status = if worktree.is_dirty { " (dirty)" } else { "" };
                options.push(format!(
                    "{} -> {} ({}){}",
                    worktree.name, worktree.path, worktree.branch, status
                ));
            }

            // Add option to create new worktree
            options.push("Create new worktree".to_string());

            if let Some(selected) = TuiSelector::select_worktree(&options)? {
                if selected == "Create new worktree" {
                    // Handle creation
                    let (local_branches, remote_branches) = get_branches(path_arg)?;

                    let mut all_branches = Vec::new();
                    all_branches.extend(local_branches.iter().map(|b| format!("local: {b}")));
                    all_branches.extend(remote_branches.iter().map(|b| format!("remote: {b}")));
                    all_branches.push("Create new branch".to_string());

                    if let Some(branch_selected) = TuiSelector::select_branch(&all_branches)? {
                        let branch_name = if branch_selected == "Create new branch" {
                            if let Some(new_branch) = TuiSelector::create_new_branch()? {
                                new_branch
                            } else {
                                return Ok(());
                            }
                        } else {
                            // Extract branch name from selection
                            branch_selected
                                .split(": ")
                                .nth(1)
                                .unwrap_or(&branch_selected)
                                .to_string()
                        };

                        let worktree_name = create_worktree(path_arg, &branch_name)?;

                        // Get the worktree path and output it for shell integration
                        let updated_worktrees = get_worktrees(path_arg)?;
                        if let Some(worktree) =
                            updated_worktrees.iter().find(|wt| wt.name == worktree_name)
                        {
                            println!("{}", worktree.path);
                        }
                    }
                } else {
                    // Handle existing worktree selection
                    if let Some(worktree_name) = selected.split(" -> ").next() {
                        if let Some(worktree) = available_worktrees
                            .iter()
                            .find(|wt| wt.name == worktree_name)
                        {
                            println!("{}", worktree.path);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    cli::Cli::command().debug_assert();
}
