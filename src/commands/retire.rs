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

    if attempt.status != ReleaseStatus::Published {
        return Err(Box::new(TransitionError::NotPublished(version.to_string())));
    }

    attempt.status = ReleaseStatus::Retired;
    attempt.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    attempt.reason = reason.to_string();
    storage.save(&attempt)?;

    let attempt_id = attempt.id.clone();
    println!("✓ 版本 {} 已退役 (发布尝试 ID: {})", version, attempt_id);
    Ok(attempt_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::release::{ReleaseAttempt, ReleaseStatus, Storage};

    #[test]
    fn test_retire_not_published() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let mut a = ReleaseAttempt::new("v1.0.0", "test");
        a.status = ReleaseStatus::Staged;
        storage.save(&a).unwrap();

        let result = run("v1.0.0", "", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_retire_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let result = run("v9.9.9", "", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_retire_from_published() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut storage = FileStorage::new(dir.path());
            let mut a = ReleaseAttempt::new("v1.0.0", "release");
            a.status = ReleaseStatus::Published;
            storage.save(&a).unwrap();
        }

        let result = run("v1.0.0", "EOL", dir.path());
        assert!(result.is_ok());

        let storage = FileStorage::new(dir.path());
        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Retired);
    }
}
