/// 发布模块 — 对应 `.agents/skills/devops-release/` 技能。
///
/// 封装完整的发布流程：
/// 1. 前置检查（构建 → 测试 → 发布状态）
/// 2. 预发布版本（可选）
/// 3. 正式发布
/// 4. 发布后检查
use std::path::Path;

/// 步骤 1: 前置检查 — 构建 + 测试 + 发布状态。
///
/// 对应 devops-release skill 的 "1. 前置检查"。
pub fn precheck(repo_path: &Path) {
    println!("━━━ 前置检查 ━━━");

    // 1a. 构建状态
    println!("--- 构建 ---");
    qtcloud_devops_cli::build::status(repo_path);

    // 1b. 测试状态
    println!("--- 测试 ---");
    let contract = qtcloud_devops_cli::contract::load(repo_path);
    qtcloud_devops_cli::test::status(repo_path, &contract);

    // 1c. 发布状态（版本一致性、CHANGELOG、tag）
    println!("--- 发布状态 ---");
    qtcloud_devops_cli::release::status(repo_path);

    println!("✅ 前置检查完成");
}

/// 步骤 2: 发布版本。
///
/// 对应 devops-release skill 的 "2. 预发布版本（可选）" + "3. 正式发布"。
///
/// `version` 格式: `vX.Y.Z` 或 `scope/vX.Y.Z`（如 `cli/v0.8.3`）。
/// `prerelease` 为 true 时跳过用户确认（相当于 rc 版）。
pub fn publish(version: &str, repo_path: &Path, prerelease: bool) -> Result<(), String> {
    // 1. 前置检查
    precheck(repo_path);

    // 2. 校验版本号
    if !qtcloud_devops_cli::contract::validate_version(version) {
        return Err(format!("版本号格式错误: {}", version));
    }
    let ver = qtcloud_devops_cli::contract::normalize_version(version);
    println!("\n━━━ 发布 {} ━━━", version);

    // 3. 执行发布
    qtcloud_devops_cli::release::publish(version, repo_path, prerelease, None)
        .map_err(|e| format!("发布失败: {}", e))?;

    // 4. 发布后检查
    println!("\n━━━ 发布后检查 ━━━");
    qtcloud_devops_cli::release::status(repo_path);

    println!("✅ 版本 {} 已发布", ver);
    Ok(())
}

/// 查看发布状态。
///
/// 对应 devops-release skill 的 "4. 发布后检查"。
pub fn status(repo_path: &Path) {
    println!("━━━ 发布状态 ━━━");
    qtcloud_devops_cli::release::status(repo_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precheck_no_cargo_toml() {
        let d = tempfile::tempdir().unwrap();
        // 无 Cargo.toml 的目录，所有检查应跳过而非崩溃
        precheck(d.path());
    }

    #[test]
    fn test_publish_rejects_invalid_version() {
        let d = tempfile::tempdir().unwrap();
        let result = publish("bad-version", d.path(), true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("格式错误"));
    }

    #[test]
    fn test_status_no_git_repo() {
        let d = tempfile::tempdir().unwrap();
        // 非 git 目录不应崩溃
        status(d.path());
    }

    #[test]
    fn test_publish_empty_dir_as_prerelease() {
        let d = tempfile::tempdir().unwrap();
        // 空目录 publish 应因无 git 仓库而失败
        let result = publish("v0.1.0-rc.1", d.path(), true);
        assert!(result.is_err());
    }
}
