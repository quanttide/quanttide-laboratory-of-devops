use crate::model::SubmoduleStatus;
use std::path::Path;

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

pub trait SubmoduleEditor {
    fn add_submodule(
        url: &str,
        path: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn init_all() -> Result<(), Box<dyn std::error::Error>>;
    fn update_single(
        name: &str,
        strategy: UpdateStrategy,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn update_all() -> Result<(), Box<dyn std::error::Error>>;
    fn sync_to_parent(name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn sync_all_to_parent() -> Result<(), Box<dyn std::error::Error>>;
    fn checkout_branch(name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn create_branch(name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn retire_submodule(name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn health_check(root: &Path) -> Result<Vec<HealthIssue>, Box<dyn std::error::Error>>;
}
