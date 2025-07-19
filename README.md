# Trees - Git Worktrees Simplified

A lightweight CLI tool for managing git worktrees with minimal flags and interactive TUI selection, inspired by tools like `fzf`, `skim`, and `zoxide`.

## Features

- **Minimal flags**: Most operations require just 1-2 arguments
- **Interactive TUI**: Uses the `ff` crate for fuzzy selection (similar to fzf)
- **Shell integration**: Supports `trees-bin shell zsh`, `trees-bin shell bash`, and `trees-bin shell fish`
- **Worktree management**: Create, list, remove, and merge worktrees
- **Branch management**: Automatically handles local and remote branches
- **Pull updates**: Fetch and pull updates for all worktrees
- **Fallback support**: Graceful fallback to simple selection when TUI is not available

## Installation

```bash
cargo install --path .
```

## Shell Integration

Add to your shell configuration:

### Zsh
```bash
eval "$(trees-bin shell zsh)"
```

### Bash
```bash
eval "$(trees-bin shell bash)"
```

### Fish
```bash
eval "$(trees-bin shell fish)"
```

This will:
- Create a `trees` function that can be used to navigate worktrees
- Add shell completion for the `trees` command
- Handle both interactive and non-interactive usage

## Usage

### Basic Commands

```bash
# Interactive worktree selection (default behavior)
trees

# List worktrees
trees list

# Add a new worktree (interactive branch selection)
trees add

# Remove worktree (interactive selection)
trees rm

# Merge worktrees (interactive selection)
trees merge

# Pull updates for all worktrees
trees pull
```

### Direct Binary Usage

```bash
# Using the trees-bin binary directly
trees-bin
trees-bin list
trees-bin add
trees-bin rm
trees-bin merge
trees-bin pull

# With specific path
trees-bin --path /path/to/repo
trees-bin --path /path/to/repo add
```

## Examples

### Interactive Selection

When no arguments are provided, `trees` opens an interactive TUI for selection:

```bash
trees        # Shows worktree list for selection
trees add    # Shows branch list for selection
trees rm     # Shows worktree list for selection
trees merge  # Shows worktree list twice (source, then target)
```

### Pull Updates

Update all worktrees with remote changes:

```bash
trees pull
# Fetches all remotes and pulls updates for worktrees that are behind
```

## Architecture

- **`trees-bin`**: The main Rust binary with all functionality
- **`trees`**: A shell function created by `trees-bin shell` that calls `trees-bin`

## Dependencies

- `git2` - Git operations
- `clap` - CLI argument parsing
- `ff` - Fuzzy finder TUI (similar to fzf)
- `anyhow` - Error handling

## Development

```bash
cargo build
cargo test
cargo run --bin trees-bin -- --help
```
