/// CI 验证工具。
///
/// 对应 scripts/validate-changelog.sh 和 scripts/validate-version.sh。
use std::path::Path;

/// 验证 CHANGELOG.md 包含指定版本的条目。
///
/// 对应 validate-changelog.sh。
pub fn validate_changelog(version: &str, changelog_path: &Path) -> Result<(), Vec<String>> {
    let errors = qtcloud_devops_cli::release::precheck_version_changelog(version, changelog_path);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// 验证 tag 版本号与配置文件（Cargo.toml / pyproject.toml）一致。
///
/// 对应 validate-version.sh。
/// `tag_ref` 格式如 `cli/v0.3.0` 或 `v0.3.0`。
pub fn validate_version(tag_ref: &str, repo_path: &Path) -> Result<String, String> {
    // 提取版本号：去掉 scope 前缀（如 cli/），保留 v0.3.0
    let tag_version = tag_ref
        .split('/')
        .last()
        .ok_or_else(|| format!("无法从 tag 提取版本号: {}", tag_ref))?;

    // 校验版本号格式
    if !qtcloud_devops_cli::release::validate_version(tag_version) {
        return Err(format!("版本号格式错误: {}", tag_version));
    }

    // 去掉 v 前缀用于比较
    let expected = tag_version.strip_prefix('v').unwrap_or(tag_version);

    // 校验 Cargo.toml
    if repo_path.join("Cargo.toml").exists() {
        let cargo_ver = read_cargo_version(repo_path)
            .ok_or_else(|| "无法读取 Cargo.toml 版本号".to_string())?;
        if cargo_ver != expected {
            return Err(format!(
                "版本不匹配: tag={} Cargo.toml={}",
                tag_version, cargo_ver
            ));
        }
    }

    // 校验 pyproject.toml（可选）
    let pyproject = repo_path.join("pyproject.toml");
    if pyproject.exists() {
        if let Some(py_ver) = read_pyproject_version(&pyproject) {
            if py_ver != expected {
                return Err(format!(
                    "版本不匹配: tag={} pyproject.toml={}",
                    tag_version, py_ver
                ));
            }
        }
    }

    Ok(expected.to_string())
}

fn read_cargo_version(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join("Cargo.toml")).ok()?;
    for line in content.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("version = \"") {
            if let Some(end) = v.find('"') {
                return Some(v[..end].to_string());
            }
        }
    }
    None
}

fn read_pyproject_version(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("version = \"") {
            if let Some(end) = v.find('"') {
                return Some(v[..end].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_changelog_found() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("CHANGELOG.md"), "## [1.0.0]\n\ncontent\n").unwrap();
        assert!(validate_changelog("v1.0.0", &d.path().join("CHANGELOG.md")).is_ok());
    }

    #[test]
    fn test_validate_changelog_missing() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("CHANGELOG.md"), "## [2.0.0]\n\ncontent\n").unwrap();
        assert!(validate_changelog("v1.0.0", &d.path().join("CHANGELOG.md")).is_err());
    }

    #[test]
    fn test_validate_changelog_file_not_found() {
        let d = tempfile::tempdir().unwrap();
        assert!(validate_changelog("v1.0.0", &d.path().join("NONE.md")).is_err());
    }

    #[test]
    fn test_validate_version_cargo_match() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let result = validate_version("v1.0.0", d.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.0.0");
    }

    #[test]
    fn test_validate_version_cargo_mismatch() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"2.0.0\"\n",
        )
        .unwrap();
        assert!(validate_version("v1.0.0", d.path()).is_err());
    }

    #[test]
    fn test_validate_version_scoped_tag() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let result = validate_version("cli/v0.1.0", d.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.1.0");
    }

    #[test]
    fn test_validate_version_invalid_format() {
        let d = tempfile::tempdir().unwrap();
        assert!(validate_version("bad", d.path()).is_err());
    }
}
