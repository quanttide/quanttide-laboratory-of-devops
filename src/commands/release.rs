use std::path::Path;
use std::process::Command;

pub fn precheck(version: &str, changelog_path: &Path, release_only: bool) -> Vec<String> {
    let mut errors = Vec::new();

    if !validate_version(version) {
        errors.push(format!("版本号格式错误: {}", version));
    }

    if changelog_path.exists() {
        let content = std::fs::read_to_string(changelog_path).unwrap_or_default();
        let ver = normalize_version(version);
        let marker = format!("## [{}]", ver);
        if !content.contains(&marker) {
            errors.push(format!("CHANGELOG.md 未找到 {} 版本记录", ver));
        }
    } else {
        errors.push(format!("CHANGELOG.md 不存在: {}", changelog_path.display()));
    }

    if release_only {
        let output = Command::new("git").args(["tag", "-l"]).output();
        if let Ok(out) = output {
            let tags = String::from_utf8_lossy(&out.stdout);
            if !tags.lines().any(|t| t.trim() == version) {
                errors.push(format!(
                    "标签不存在: {}（--release-only 需要标签已存在）",
                    version
                ));
            }
        }
    }

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output();
    if let Ok(out) = output {
        if !out.stdout.is_empty() {
            errors.push("工作区有未提交的变更".to_string());
        }
    }

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
    if let Ok(out) = output {
        let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !branch.is_empty()
            && !branch.starts_with("main")
            && !branch.starts_with("master")
            && !branch.starts_with("release/")
        {
            errors.push(format!(
                "不在可发布分支上 (当前: {}), 请切换到 main/master/release/*",
                branch
            ));
        }
    }

    errors
}

pub fn validate_version(version: &str) -> bool {
    let re = regex::Regex::new(
        r"^(v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?|[a-zA-Z0-9_.-]+/v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?)$",
    )
    .unwrap();
    re.is_match(version)
}

fn normalize_version(version: &str) -> String {
    let s = version.strip_prefix('v').unwrap_or(version);
    s.split("/v").last().unwrap_or(s).to_string()
}

pub fn extract_notes(version: &str, changelog_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(changelog_path).ok()?;
    let ver = normalize_version(version);
    let start_marker = format!("## [{}]", ver);

    let mut capture = false;
    let mut notes: Vec<&str> = Vec::new();

    for line in content.lines() {
        if line.trim().starts_with(&start_marker) {
            capture = true;
            continue;
        }
        if capture {
            if line.starts_with("## [") {
                break;
            }
            notes.push(line);
        }
    }

    let text = notes.join("\n").trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn confirm_release(version: &str, notes: Option<&str>, yes: bool) -> bool {
    println!("\n发布版本: {}", version);
    println!();
    println!("检查结果:");
    println!("  ✓ 预检查全部通过");
    println!();
    println!("Release Notes 预览:");
    println!("{}", notes.unwrap_or("(空)"));
    println!();

    if yes {
        return true;
    }

    use std::io::Write;
    print!("确认发布? (y/N): ");
    std::io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}

pub fn create_tag(version: &str) -> bool {
    let result = Command::new("git").args(["tag", version]).output();
    match result {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            eprintln!("创建标签失败: {}", String::from_utf8_lossy(&out.stderr).trim());
            false
        }
        Err(e) => {
            eprintln!("创建标签失败: {}", e);
            false
        }
    }
}

pub fn push_tag(version: &str) -> bool {
    let result = Command::new("git")
        .args(["push", "origin", version])
        .output();
    match result {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            eprintln!("推送标签失败: {}", String::from_utf8_lossy(&out.stderr).trim());
            false
        }
        Err(e) => {
            eprintln!("推送标签失败: {}", e);
            false
        }
    }
}

pub fn get_remote_repo() -> Option<String> {
    let result = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !result.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&result.stdout).trim().to_string();
    parse_github_repo(&url)
}

pub fn parse_github_repo(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"github\.com[/:]([^/]+/[^/]+?)(?:\.git)?$")
        .ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}

pub fn create_release(version: &str, notes: &str, repo: &str) -> bool {
    let result = Command::new("gh")
        .args([
            "release",
            "create",
            version,
            "--title",
            version,
            "--notes",
            notes,
            "--repo",
            repo,
        ])
        .output();
    match result {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            eprintln!(
                "创建 Release 失败: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            false
        }
        Err(e) => {
            eprintln!("创建 Release 失败: {}", e);
            false
        }
    }
}

pub fn rollback_tag(version: &str) {
    Command::new("git")
        .args(["tag", "-d", version])
        .output()
        .ok();
    Command::new("git")
        .args(["push", "origin", "--delete", version])
        .output()
        .ok();
    println!("↻ 标签 {} 已回滚", version);
}

pub fn run(
    version: &str,
    changelog_path: &Path,
    dry_run: bool,
    tag_only: bool,
    release_only: bool,
    yes: bool,
) -> i32 {
    let errors = precheck(version, changelog_path, release_only);
    if !errors.is_empty() {
        println!("预检查失败:");
        for err in &errors {
            println!("  ✗ {}", err);
        }
        return 1;
    }

    let notes = extract_notes(version, changelog_path);
    println!("\n=== Release Notes 预览 ===");
    println!("{}", notes.as_deref().unwrap_or("(空)"));
    println!("=========================\n");

    if dry_run {
        println!("✓ 预检查通过 (dry-run 模式，不执行)");
        return 0;
    }

    if !confirm_release(version, notes.as_deref(), yes) {
        println!("已取消发布");
        return 0;
    }

    let mut tag_created = false;

    if !release_only {
        let output = Command::new("git").args(["tag", "-l"]).output();
        let tag_exists = output.ok().is_some_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|t| t.trim() == version)
        });

        if tag_exists {
            println!("→ 标签 {} 已存在，跳过 tag 创建", version);
        } else {
            if !create_tag(version) {
                return 1;
            }
            if !push_tag(version) {
                rollback_tag(version);
                return 1;
            }
            tag_created = true;
            println!("✓ 标签 {} 已创建并推送", version);
        }
    }

    if !tag_only {
        let repo = get_remote_repo();
        match repo {
            Some(r) => {
                if !create_release(version, notes.as_deref().unwrap_or(""), &r) {
                    if tag_created {
                        rollback_tag(version);
                    }
                    return 1;
                }
                println!("✓ GitHub Release {} 已创建", version);
                println!("  https://github.com/{}/releases/tag/{}", r, version);
            }
            None => {
                println!("错误: 无法从 git remote 解析 GitHub 仓库");
                if tag_created {
                    rollback_tag(version);
                }
                return 1;
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_validate_version_v_prefix() {
        assert!(validate_version("v1.2.3"));
    }

    #[test]
    fn test_validate_version_with_suffix() {
        assert!(validate_version("v1.2.3-alpha.1"));
        assert!(validate_version("v1.2.3-rc1"));
    }

    #[test]
    fn test_validate_version_pkg_prefix() {
        assert!(validate_version("pkg/v1.2.3"));
    }

    #[test]
    fn test_validate_version_invalid() {
        assert!(!validate_version("1.2.3"));
        assert!(!validate_version("v1.2"));
        assert!(!validate_version("v1.2.3.4"));
        assert!(!validate_version("abc"));
        assert!(!validate_version(""));
        assert!(!validate_version("vabc.def.ghi"));
    }

    #[test]
    fn test_normalize_version_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_normalize_version_pkg() {
        assert_eq!(normalize_version("pkg/v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_normalize_version_with_suffix() {
        assert_eq!(normalize_version("v1.2.3-rc1"), "1.2.3-rc1");
    }

    #[test]
    fn test_extract_notes_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "# Changelog\n\n## [1.0.0] — 2026-01-01\n\n### Added\n\n- feature 1\n- feature 2\n\n## [0.9.0]\n\nold"
        )
        .unwrap();

        let notes = extract_notes("v1.0.0", &path);
        assert!(notes.is_some());
        let text = notes.unwrap();
        assert!(text.contains("feature 1"));
        assert!(text.contains("feature 2"));
        assert!(!text.contains("old"));
    }

    #[test]
    fn test_extract_notes_pkg_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "# Changelog\n\n## [2.0.0] — 2026-05-24\n\n### Breaking\n\nrelease notes here").unwrap();

        let notes = extract_notes("pkg/v2.0.0", &path);
        assert!(notes.is_some());
        assert!(notes.unwrap().contains("Breaking"));
    }

    #[test]
    fn test_extract_notes_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "# Changelog\n\n## [1.0.0]\n\ncontent").unwrap();
        assert!(extract_notes("v2.0.0", &path).is_none());
    }

    #[test]
    fn test_confirm_release_yes_flag() {
        assert!(confirm_release("v1.0.0", None, true));
    }

    #[test]
    fn test_parse_github_repo_https() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(parse_github_repo(url), Some("owner/repo".into()));
    }

    #[test]
    fn test_parse_github_repo_ssh() {
        let url = "git@github.com:owner/repo.git";
        assert_eq!(parse_github_repo(url), Some("owner/repo".into()));
    }

    #[test]
    fn test_parse_github_repo_no_git_suffix() {
        let url = "https://github.com/owner/repo";
        assert_eq!(parse_github_repo(url), Some("owner/repo".into()));
    }

    #[test]
    fn test_parse_github_repo_not_github() {
        let url = "https://gitlab.com/owner/repo.git";
        assert_eq!(parse_github_repo(url), None);
    }


}
