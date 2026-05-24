use std::path::Path;

use crate::model::release::{FileStorage, ReleaseStatus, Storage};

pub fn run(
    version: &str,
    repo_path: &Path,
    yes: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut storage = FileStorage::new(repo_path);
    let mut attempt = storage
        .load(version)
        .ok_or_else(|| format!("版本 {} 不存在，请先执行 stage", version))?;

    if attempt.status != ReleaseStatus::Staged {
        return Err(format!("版本 {} 不处于 Staged 状态 (当前: {:?})", version, attempt.status).into());
    }

    if !crate::commands::release::confirm_release(version, None, yes) {
        return Err("已取消发布".into());
    }

    let tag_ok = crate::commands::release::create_tag(version);
    if !tag_ok {
        return Err(format!("创建标签 {} 失败", version).into());
    }

    let push_ok = crate::commands::release::push_tag(version);
    if !push_ok {
        crate::commands::release::rollback_tag(version);
        return Err(format!("推送标签 {} 失败", version).into());
    }
    println!("✓ 标签 {} 已创建并推送", version);

    let changelog_path = repo_path.join("CHANGELOG.md");
    let notes = crate::commands::release::extract_notes(version, &changelog_path);

    if let Some(repo) = crate::commands::release::get_remote_repo() {
        let release_ok =
            crate::commands::release::create_release(version, notes.as_deref().unwrap_or(""), &repo);
        if !release_ok {
            crate::commands::release::rollback_tag(version);
            return Err("创建 GitHub Release 失败".into());
        }
        println!("✓ GitHub Release {} 已创建", version);
        println!("  https://github.com/{}/releases/tag/{}", repo, version);
    }

    attempt.status = ReleaseStatus::Published;
    attempt.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    storage.save(&attempt)?;

    let attempt_id = attempt.id.clone();
    println!("✓ 版本 {} 已发布 (发布尝试 ID: {})", version, attempt_id);
    Ok(attempt_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::release::{ReleaseRecord, ReleaseStatus, Storage};

    fn make_staged(version: &str) -> ReleaseRecord {
        ReleaseRecord::new_staged(version)
    }

    #[test]
    fn test_publish_not_staged() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("CHANGELOG.md"), "## [1.0.0]\n\ncontent").unwrap();

        {
            let mut storage = FileStorage::new(dir.path());
            let mut r = make_staged("v1.0.0");
            r.status = ReleaseStatus::Cancelled;
            storage.save(&r).unwrap();
        }

        let result = run("v1.0.0", dir.path(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_not_found() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("CHANGELOG.md"), "## [1.0.0]\n\ncontent").unwrap();
        let result = run("v1.0.0", dir.path(), true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("请先执行 stage"));
    }

    #[test]
    fn test_publish_user_cancels() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("CHANGELOG.md"), "## [1.0.0]\n\ncontent").unwrap();

        let mut storage = FileStorage::new(dir.path());
        storage.save(&make_staged("v1.0.0")).unwrap();

        let result = run("v1.0.0", dir.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("取消"));

        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Staged);
    }

}
