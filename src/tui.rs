use anyhow::Result;
use ff::{TuiConfig, run_tui_with_config};
use std::env;
use std::io::{self, Write};

pub struct TuiSelector;

impl TuiSelector {
    fn calculate_height(item_count: usize) -> u16 {
        // Add 2 for the search prompt and some padding
        let needed_height = (item_count + 2) as u16;
        // Cap at 15 lines maximum to avoid taking too much screen space
        needed_height.min(15)
    }

    pub fn select_worktree(worktrees: &[String]) -> Result<Option<String>> {
        if worktrees.is_empty() {
            println!("No worktrees found");
            return Ok(None);
        }

        // Check if we're in a proper terminal
        if env::var("TERM").is_err() || env::var("TERM").unwrap() == "dumb" {
            // Fallback to simple selection if not in a proper terminal
            return Self::fallback_selection(worktrees);
        }

        let items = worktrees.to_vec();
        let height = Self::calculate_height(items.len());
        let mut config = TuiConfig::with_height(height);
        config.show_help_text = false;

        match run_tui_with_config(items, false, config) {
            Ok(selected) => {
                if selected.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(selected[0].clone()))
                }
            }
            Err(_) => {
                // Fallback to simple selection if TUI fails
                Self::fallback_selection(worktrees)
            }
        }
    }

    pub fn select_branch(branches: &[String]) -> Result<Option<String>> {
        if branches.is_empty() {
            println!("No branches found");
            return Ok(None);
        }

        // Check if we're in a proper terminal
        if env::var("TERM").is_err() || env::var("TERM").unwrap() == "dumb" {
            // Fallback to simple selection if not in a proper terminal
            return Self::fallback_selection(branches);
        }

        let items = branches.to_vec();
        let height = Self::calculate_height(items.len());
        let mut config = TuiConfig::with_height(height);
        config.show_help_text = false;

        match run_tui_with_config(items, false, config) {
            Ok(selected) => {
                if selected.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(selected[0].clone()))
                }
            }
            Err(_) => {
                // Fallback to simple selection if TUI fails
                Self::fallback_selection(branches)
            }
        }
    }

    pub fn create_new_branch() -> Result<Option<String>> {
        print!("Enter new branch name: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let branch_name = input.trim();
        if branch_name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(branch_name.to_string()))
        }
    }

    pub fn confirm_deletion(worktree_name: &str) -> Result<bool> {
        let options = vec![
            format!("Yes - Delete {}", worktree_name),
            "No - Cancel".to_string(),
        ];
        let height = Self::calculate_height(options.len());
        let mut config = TuiConfig::with_height(height);
        config.show_help_text = false;

        match run_tui_with_config(options, false, config) {
            Ok(selected) => {
                if selected.is_empty() {
                    Ok(false)
                } else {
                    Ok(selected[0].starts_with("Yes"))
                }
            }
            Err(_) => {
                // Fallback to simple confirmation
                Self::fallback_confirmation(worktree_name)
            }
        }
    }

    fn fallback_selection(items: &[String]) -> Result<Option<String>> {
        println!("Select an option:");
        for (i, item) in items.iter().enumerate() {
            println!("{}. {}", i + 1, item);
        }
        print!("Enter number (1-{}): ", items.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let choice: usize = input.trim().parse().unwrap_or(0);
        if choice > 0 && choice <= items.len() {
            Ok(Some(items[choice - 1].clone()))
        } else {
            Ok(None)
        }
    }

    fn fallback_confirmation(worktree_name: &str) -> Result<bool> {
        print!("Are you sure you want to delete {}? (y/N): ", worktree_name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }
}
