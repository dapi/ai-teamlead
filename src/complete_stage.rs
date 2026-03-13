use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{Config, FlowStatuses};
use crate::github::GhProjectClient;
use crate::runtime::RuntimeLayout;
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum StageOutcome {
    PlanReady,
    NeedsClarification,
    Blocked,
}

impl StageOutcome {
    pub fn target_status<'a>(&self, statuses: &'a FlowStatuses) -> &'a str {
        match self {
            Self::PlanReady => &statuses.waiting_for_plan_review,
            Self::NeedsClarification => &statuses.waiting_for_clarification,
            Self::Blocked => &statuses.analysis_blocked,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlanReady => "plan-ready",
            Self::NeedsClarification => "needs-clarification",
            Self::Blocked => "blocked",
        }
    }
}

pub fn run_complete_stage(
    shell: &dyn Shell,
    session_uuid: &str,
    outcome: &StageOutcome,
    message: &str,
) -> Result<()> {
    let repo_root = resolve_repo_root(shell)?;
    let config = Config::load_from_repo_root(&repo_root)?;
    let runtime = RuntimeLayout::from_repo_root(&repo_root);

    let manifest = runtime
        .load_session_manifest(session_uuid)?
        .ok_or_else(|| anyhow::anyhow!("session not found: {session_uuid}"))?;

    if manifest.status == "completed" {
        eprintln!("warning: session {session_uuid} is already completed, skipping");
        return Ok(());
    }

    let issue_number = manifest.issue_number;
    let worktree_root = resolve_worktree_root()?;
    let artifacts_dir = std::env::var("AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR")
        .unwrap_or_else(|_| format!("specs/issues/{issue_number}"));
    let branch = std::env::var("AI_TEAMLEAD_ANALYSIS_BRANCH")
        .unwrap_or_else(|_| format!("analysis/issue-{issue_number}"));

    let commit_message = format!("analysis(#{issue_number}): {message}");

    // Step 1: git add + commit (if there are changes)
    let committed = git_add_and_commit(shell, &worktree_root, &artifacts_dir, &commit_message)?;

    // Step 2: git push
    if committed {
        git_push(shell, &worktree_root, &branch)?;
    }

    // Step 3: create draft PR (only for plan-ready)
    if matches!(outcome, StageOutcome::PlanReady) && committed {
        let pr_title = format!("analysis(#{issue_number}): {message}");
        let pr_body = format!(
            "Ref #{issue_number}\n\nOutcome: plan-ready\nArtifacts: `{artifacts_dir}/`"
        );
        create_draft_pr_if_needed(shell, &worktree_root, &branch, &pr_title, &pr_body)?;
    }

    // Step 4: update GitHub Project status
    let target_status = outcome.target_status(&config.issue_analysis_flow.statuses);
    update_project_status(shell, &repo_root, &config, &manifest, target_status)?;

    // Step 5: update runtime state
    // Order matters: update_issue_flow_status first, then session status.
    // session status = "completed" is the idempotency guard — writing it last
    // ensures retries re-execute all preceding steps.
    runtime.update_issue_flow_status(issue_number, target_status)?;
    runtime.update_session_status(session_uuid, "completed")?;

    println!(
        "complete-stage: issue=#{issue_number} outcome={} status={target_status}",
        outcome.as_str(),
    );

    Ok(())
}

fn resolve_repo_root(shell: &dyn Shell) -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    // Fallback: primary worktree from git
    let cwd = std::env::current_dir().context("failed to get cwd")?;
    let output = shell.run(&cwd, "git", &["worktree", "list", "--porcelain"])?;
    let first_line = output
        .lines()
        .find(|l| l.starts_with("worktree "))
        .ok_or_else(|| anyhow::anyhow!("cannot determine primary worktree"))?;
    Ok(PathBuf::from(
        first_line.strip_prefix("worktree ").unwrap(),
    ))
}

fn resolve_worktree_root() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_WORKTREE_ROOT") {
        return Ok(PathBuf::from(root));
    }
    std::env::current_dir().context("failed to get cwd")
}

fn git_add_and_commit(
    shell: &dyn Shell,
    worktree: &Path,
    artifacts_dir: &str,
    commit_message: &str,
) -> Result<bool> {
    let artifacts_path = worktree.join(artifacts_dir);
    if !artifacts_path.exists() {
        eprintln!("complete-stage: no artifacts directory at {artifacts_dir}, skipping commit");
        return Ok(false);
    }

    shell.run(worktree, "git", &["add", artifacts_dir])?;

    // Check if there are staged changes.
    // Using --name-only instead of --quiet: --quiet uses exit code 1 for "has changes"
    // which shell.run() would interpret as an error, masking real git failures.
    let staged_files = shell.run(worktree, "git", &["diff", "--cached", "--name-only"])?;
    if staged_files.is_empty() {
        eprintln!("complete-stage: no staged changes, skipping commit");
        return Ok(false);
    }

    shell.run(worktree, "git", &["commit", "-m", commit_message])?;
    Ok(true)
}

fn git_push(shell: &dyn Shell, worktree: &Path, branch: &str) -> Result<()> {
    shell
        .run(worktree, "git", &["push", "origin", branch])
        .context("failed to push analysis branch")?;
    Ok(())
}

fn create_draft_pr_if_needed(
    shell: &dyn Shell,
    worktree: &Path,
    branch: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    match shell.run(
        worktree,
        "gh",
        &[
            "pr", "list", "--head", branch, "--json", "number", "--jq", "length",
        ],
    ) {
        Ok(count) if count.trim() != "0" => {
            eprintln!("complete-stage: draft PR already exists for branch {branch}");
            return Ok(());
        }
        Ok(_) => {} // count == 0, proceed to create
        Err(e) => {
            eprintln!("complete-stage: warning: failed to check existing PRs: {e}");
            eprintln!("complete-stage: skipping PR creation");
            return Ok(());
        }
    }

    let result = shell.run(
        worktree,
        "gh",
        &["pr", "create", "--draft", "--title", title, "--body", body],
    );
    match result {
        Ok(url) => println!("complete-stage: created draft PR: {url}"),
        Err(e) => eprintln!("complete-stage: warning: failed to create draft PR: {e}"),
    }
    Ok(())
}

fn update_project_status(
    shell: &dyn Shell,
    repo_root: &Path,
    config: &Config,
    manifest: &crate::runtime::SessionManifest,
    target_status: &str,
) -> Result<()> {
    let github = GhProjectClient::new(shell);
    let snapshot = github.load_project_snapshot(repo_root, &config.github.project_id)?;

    let issue_item = snapshot
        .items
        .iter()
        .find(|item| {
            item.issue_number == manifest.issue_number
                && item.matches_repo(&manifest.github_owner, &manifest.github_repo)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("issue #{} not found in project", manifest.issue_number)
        })?;

    let option_id = snapshot.option_id_by_name(target_status)?;
    github.update_status(
        repo_root,
        &config.github.project_id,
        &issue_item.item_id,
        &snapshot.status_field_id,
        option_id,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FlowStatuses;
    use clap::ValueEnum;

    fn sample_statuses() -> FlowStatuses {
        FlowStatuses {
            backlog: "Backlog".into(),
            analysis_in_progress: "Analysis In Progress".into(),
            waiting_for_clarification: "Waiting for Clarification".into(),
            waiting_for_plan_review: "Waiting for Plan Review".into(),
            ready_for_implementation: "Ready for Implementation".into(),
            analysis_blocked: "Analysis Blocked".into(),
        }
    }

    #[test]
    fn parses_valid_outcomes_via_value_enum() {
        let variants = StageOutcome::value_variants();
        assert_eq!(variants.len(), 3);

        let plan_ready = StageOutcome::from_str("plan-ready", true).unwrap();
        assert_eq!(plan_ready, StageOutcome::PlanReady);

        let needs_clar = StageOutcome::from_str("needs-clarification", true).unwrap();
        assert_eq!(needs_clar, StageOutcome::NeedsClarification);

        let blocked = StageOutcome::from_str("blocked", true).unwrap();
        assert_eq!(blocked, StageOutcome::Blocked);
    }

    #[test]
    fn rejects_invalid_outcome_via_value_enum() {
        let result = StageOutcome::from_str("unknown", true);
        assert!(result.is_err());
    }

    #[test]
    fn maps_outcome_to_correct_status() {
        let statuses = sample_statuses();
        assert_eq!(
            StageOutcome::PlanReady.target_status(&statuses),
            "Waiting for Plan Review"
        );
        assert_eq!(
            StageOutcome::NeedsClarification.target_status(&statuses),
            "Waiting for Clarification"
        );
        assert_eq!(
            StageOutcome::Blocked.target_status(&statuses),
            "Analysis Blocked"
        );
    }
}
