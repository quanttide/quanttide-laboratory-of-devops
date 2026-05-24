use std::path::Path;

use crate::model::release::{FileStorage, ReleaseStatus, Storage, TransitionError};

pub fn run(
    version: &str,
    reason: &str,
    repo_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut storage = FileStorage::new(repo_path);
    let mut attempt = storage
        .load(version)
        .ok_or_else(|| format!("版本 {} 不存在", version))?;

    if attempt.status != ReleaseStatus::Staged {
        return Err(Box::new(TransitionError::NotStaged(version.to_string())));
    }

    crate::commands::release::rollback_tag(version);

    if let Some(repo) = crate::commands::release::get_remote_repo() {
        std::process::Command::new("gh")
            .args(["release", "delete", version, "--repo", &repo, "--yes"])
            .output()
            .ok();
        println!("✓ GitHub Release {} 已删除", version);
    }

    attempt.status = ReleaseStatus::Cancelled;
    attempt.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    attempt.reason = reason.to_string();
    storage.save(&attempt)?;

    let attempt_id = attempt.id.clone();
    println!("✓ 版本 {} 已取消 (发布尝试 ID: {})", version, attempt_id);
    Ok(attempt_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::release::{ReleaseAttempt, ReleaseStatus, Storage};

    #[test]
    fn test_cancel_not_staged() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let mut a = ReleaseAttempt::new("v1.0.0", "test");
        a.status = ReleaseStatus::Published;
        storage.save(&a).unwrap();

        let result = run("v1.0.0", "reason", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let result = run("v9.9.9", "", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_happy_path() {
        let dir = tempfile::tempdir().unwrap();
        let attempt_id;
        {
            let mut storage = FileStorage::new(dir.path());
            let a = ReleaseAttempt::new("v1.0.0", "test");
            attempt_id = a.id.clone();
            storage.save(&a).unwrap();
        }

        let result = run("v1.0.0", "no longer needed", dir.path());
        assert!(result.is_ok());

        let storage = FileStorage::new(dir.path());
        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Cancelled);
        assert_eq!(loaded.reason, "no longer needed");
        assert_eq!(loaded.id, attempt_id);
    }
}
