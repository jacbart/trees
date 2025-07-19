#[test]
fn test_branch_name_sanitization() {
    // Test that branch names with forward slashes are sanitized
    let branch_name = "feat/shell-integration";
    let sanitized = branch_name.replace('/', "-");
    assert_eq!(sanitized, "feat-shell-integration");
}

#[test]
fn test_worktree_name_format() {
    // Test the worktree name format with sanitized branch names
    let repo_name = "trees";
    let branch_name = "feat/shell-integration";
    let sanitized_branch_name = branch_name.replace('/', "-");
    let worktree_name = format!("{repo_name}-{sanitized_branch_name}");

    assert_eq!(worktree_name, "trees-feat-shell-integration");
}

#[test]
fn test_multiple_slashes_sanitization() {
    // Test that multiple forward slashes are all replaced
    let branch_name = "feature/user/authentication";
    let sanitized = branch_name.replace('/', "-");
    assert_eq!(sanitized, "feature-user-authentication");
}
