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
