use crate::model::SubmoduleStatus;
use std::path::Path;

pub mod editor;
pub mod export;
pub mod history;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStrategy {
    FastForward,
    Rebase,
    Merge,
}

#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub submodule_name: String,
    pub status: SubmoduleStatus,
    pub description: String,
    pub suggested_action: String,
}

impl UpdateStrategy {
    pub fn to_git2_update(&self) -> git2::SubmoduleUpdate {
        match self {
            Self::FastForward => git2::SubmoduleUpdate::Checkout,
            Self::Rebase => git2::SubmoduleUpdate::Rebase,
            Self::Merge => git2::SubmoduleUpdate::Merge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SubmoduleStatus;

    #[test]
    fn test_update_strategy_fast_forward() {
        assert_eq!(
            UpdateStrategy::FastForward.to_git2_update(),
            git2::SubmoduleUpdate::Checkout
        );
    }

    #[test]
    fn test_update_strategy_rebase() {
        assert_eq!(
            UpdateStrategy::Rebase.to_git2_update(),
            git2::SubmoduleUpdate::Rebase
        );
    }

    #[test]
    fn test_update_strategy_merge() {
        assert_eq!(
            UpdateStrategy::Merge.to_git2_update(),
            git2::SubmoduleUpdate::Merge
        );
    }

    #[test]
    fn test_update_strategy_clone_eq() {
        let a = UpdateStrategy::FastForward;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_update_strategy_debug() {
        assert_eq!(format!("{:?}", UpdateStrategy::FastForward), "FastForward");
        assert_eq!(format!("{:?}", UpdateStrategy::Rebase), "Rebase");
        assert_eq!(format!("{:?}", UpdateStrategy::Merge), "Merge");
    }

    #[test]
    fn test_health_issue_creation() {
        let issue = HealthIssue {
            submodule_name: "test".into(),
            status: SubmoduleStatus::Dirty,
            description: "有未提交修改".into(),
            suggested_action: "提交或 stash".into(),
        };
        assert_eq!(issue.submodule_name, "test");
        assert_eq!(issue.status, SubmoduleStatus::Dirty);
        assert_eq!(issue.description, "有未提交修改");
        assert_eq!(issue.suggested_action, "提交或 stash");
    }

    #[test]
    fn test_health_issue_clone() {
        let a = HealthIssue {
            submodule_name: "x".into(),
            status: SubmoduleStatus::Clean,
            description: "d".into(),
            suggested_action: "a".into(),
        };
        let b = a.clone();
        assert_eq!(a.submodule_name, b.submodule_name);
        assert_eq!(a.status, b.status);
    }
}

pub trait SubmoduleEditor {
    fn root(&self) -> &Path;
    fn add_submodule(
        &self,
        url: &str,
        path: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn init_all(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn update_single(
        &self,
        name: &str,
        strategy: UpdateStrategy,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn update_all(&self, strategy: UpdateStrategy) -> Result<(), Box<dyn std::error::Error>>;
    fn sync_to_parent(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn sync_all_to_parent(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn checkout_branch(&self, name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn checkout_all(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn create_branch(&self, name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn branch_all(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn retire_submodule(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn health_check(&self) -> Result<Vec<HealthIssue>, Box<dyn std::error::Error>>;
}
