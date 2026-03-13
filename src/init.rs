use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::project_files::ProjectPaths;

const PROJECT_README_TEMPLATE: &str = include_str!("../templates/init/README.md");
const SETTINGS_TEMPLATE: &str = include_str!("../templates/init/settings.yml");
const ISSUE_ANALYSIS_FLOW_TEMPLATE: &str = include_str!("../templates/init/issue-analysis-flow.md");

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

pub fn init_project_files(paths: &ProjectPaths) -> Result<InitReport> {
    fs::create_dir_all(&paths.customization_root)
        .with_context(|| format!("failed to create {}", paths.customization_root.display()))?;
    fs::create_dir_all(&paths.flows_dir)
        .with_context(|| format!("failed to create {}", paths.flows_dir.display()))?;

    let mut report = InitReport::default();
    write_if_missing(
        &paths.settings_path,
        SETTINGS_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.readme_path,
        PROJECT_README_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_flow_path,
        ISSUE_ANALYSIS_FLOW_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;

    Ok(report)
}

fn write_if_missing(
    path: &PathBuf,
    content: &str,
    created: &mut Vec<PathBuf>,
    skipped: &mut Vec<PathBuf>,
) -> Result<()> {
    if path.exists() {
        skipped.push(path.clone());
        return Ok(());
    }

    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    created.push(path.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::init_project_files;
    use crate::project_files::ProjectPaths;

    #[test]
    fn initializes_project_files_without_overwriting_existing_files() {
        let temp = tempdir().expect("temp dir");
        let paths = ProjectPaths::from_repo_root(temp.path());

        let first = init_project_files(&paths).expect("first init");
        assert_eq!(first.created.len(), 3);
        assert!(paths.settings_path.exists());
        assert!(paths.readme_path.exists());
        assert!(paths.issue_analysis_flow_path.exists());

        let second = init_project_files(&paths).expect("second init");
        assert_eq!(second.created.len(), 0);
        assert_eq!(second.skipped.len(), 3);
    }
}
