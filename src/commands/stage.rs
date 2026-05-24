use std::path::Path;

use crate::model::release::{FileStorage, ReleaseAttempt, ReleaseStatus, Storage};

pub fn run(version: &str, reason: &str, repo_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if !crate::commands::release::validate_version(version) {
        return Err(format!("版本号格式错误: {}", version).into());
    }

    let mut storage = FileStorage::new(repo_path);

    if let Some(existing) = storage.load(version) {
        match existing.status {
            ReleaseStatus::Published => {
                return Err(format!("版本 {} 已发布，不可重复 stage", version).into());
            }
            ReleaseStatus::Staged => {
                let mut updated = existing.clone();
                updated.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string();
                updated.reason = reason.to_string();
                storage.save(&updated)?;
                return Ok(updated.id);
            }
            ReleaseStatus::Cancelled => {}
            ReleaseStatus::Retired => {
                return Err(format!("版本 {} 已退役，不可重复 stage", version).into());
            }
        }
    }

    let attempt = ReleaseAttempt::new(version, reason);
    storage.save(&attempt)?;
    println!("✓ 版本 {} 已进入 Staged 状态 (发布尝试 ID: {})", version, attempt.id);
    Ok(attempt.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::release::Storage;

    #[test]
    fn test_stage_new_version() {
        let dir = tempfile::tempdir().unwrap();
        let id = run("v1.0.0", "initial", dir.path()).unwrap();
        assert!(!id.is_empty());

        let storage = FileStorage::new(dir.path());
        let a = storage.load("v1.0.0").unwrap();
        assert_eq!(a.status, ReleaseStatus::Staged);
    }

    #[test]
    fn test_stage_invalid_version() {
        let dir = tempfile::tempdir().unwrap();
        let result = run("bad", "test", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_stage_already_published_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let mut a = ReleaseAttempt::new("v1.0.0", "test");
        a.status = ReleaseStatus::Published;
        storage.save(&a).unwrap();

        let result = run("v1.0.0", "", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已发布"));
    }

    #[test]
    fn test_stage_cancelled_restage() {
        let dir = tempfile::tempdir().unwrap();
        let old_id;
        {
            let mut storage = FileStorage::new(dir.path());
            let mut a = ReleaseAttempt::new("v1.0.0", "first");
            old_id = a.id.clone();
            a.status = ReleaseStatus::Cancelled;
            storage.save(&a).unwrap();
        }

        let id = run("v1.0.0", "retry", dir.path()).unwrap();
        assert!(!id.is_empty());

        let storage = FileStorage::new(dir.path());
        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Staged);
        assert_ne!(loaded.id, old_id);
    }

    #[test]
    fn test_stage_retired_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let mut a = ReleaseAttempt::new("v1.0.0", "test");
        a.status = ReleaseStatus::Retired;
        storage.save(&a).unwrap();

        let result = run("v1.0.0", "", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("退役"));
    }

    #[test]
    fn test_stage_idempotent_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let id1 = run("v1.0.0", "first", dir.path()).unwrap();

        let id2 = run("v1.0.0", "refresh", dir.path()).unwrap();
        assert_eq!(id1, id2);

        let storage = FileStorage::new(dir.path());
        let attempts = storage.list();
        assert_eq!(attempts.len(), 1);
    }
}
