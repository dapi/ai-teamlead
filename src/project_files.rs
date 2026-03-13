use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPaths {
    pub customization_root: PathBuf,
    pub settings_path: PathBuf,
    pub flows_dir: PathBuf,
    pub issue_analysis_flow_path: PathBuf,
    pub readme_path: PathBuf,
}

impl ProjectPaths {
    pub fn from_repo_root(repo_root: &Path) -> Self {
        let customization_root = repo_root.join(".ai-teamlead");
        let settings_path = customization_root.join("settings.yml");
        let flows_dir = customization_root.join("flows");
        let issue_analysis_flow_path = flows_dir.join("issue-analysis-flow.md");
        let readme_path = customization_root.join("README.md");

        Self {
            customization_root,
            settings_path,
            flows_dir,
            issue_analysis_flow_path,
            readme_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ProjectPaths;

    #[test]
    fn builds_expected_project_paths() {
        let paths = ProjectPaths::from_repo_root(Path::new("/repo"));
        assert_eq!(paths.customization_root, Path::new("/repo/.ai-teamlead"));
        assert_eq!(
            paths.settings_path,
            Path::new("/repo/.ai-teamlead/settings.yml")
        );
        assert_eq!(paths.flows_dir, Path::new("/repo/.ai-teamlead/flows"));
        assert_eq!(
            paths.issue_analysis_flow_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis-flow.md")
        );
        assert_eq!(paths.readme_path, Path::new("/repo/.ai-teamlead/README.md"));
    }
}
