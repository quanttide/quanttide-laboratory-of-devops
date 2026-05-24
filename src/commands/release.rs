use std::path::Path;
use std::process::Command;

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

pub fn precheck_version_changelog(version: &str, changelog_path: &Path) -> Vec<String> {
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

    errors
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ---- validate_version ----

    #[test]
    fn test_validate_version_v_prefix() {
        assert!(validate_version("v1.2.3"));
    }

    #[test]
    fn test_validate_version_with_suffix() {
        assert!(validate_version("v1.2.3-alpha.1"));
        assert!(validate_version("v1.2.3-rc1"));
        assert!(validate_version("v1.2.3-beta"));
        assert!(validate_version("v1.2.3-0"));
    }

    #[test]
    fn test_validate_version_pkg_prefix() {
        assert!(validate_version("pkg/v1.2.3"));
        assert!(validate_version("my-crate/v1.2.3"));
        assert!(validate_version("a.b_c/v1.2.3"));
    }

    #[test]
    fn test_validate_version_pkg_with_suffix() {
        assert!(validate_version("pkg/v1.2.3-rc.1"));
    }

    #[test]
    fn test_validate_version_invalid() {
        assert!(!validate_version("1.2.3"));
        assert!(!validate_version("v1.2"));
        assert!(!validate_version("v1.2.3.4"));
        assert!(!validate_version("abc"));
        assert!(!validate_version(""));
        assert!(!validate_version("vabc.def.ghi"));
        assert!(!validate_version("V1.2.3"));
        assert!(!validate_version(" v1.2.3"));
        assert!(!validate_version("v1.2.3 "));
        assert!(!validate_version("v1.0.0-"));
        assert!(!validate_version("/v1.0.0"));
    }

    // ---- normalize_version ----

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
    fn test_normalize_version_complex_pkg() {
        assert_eq!(normalize_version("my-crate/v1.2.3-beta.2"), "1.2.3-beta.2");
    }

    #[test]
    fn test_normalize_version_no_prefix() {
        assert_eq!(normalize_version("1.2.3"), "1.2.3");
    }

    // ---- precheck_version_changelog ----

    #[test]
    fn test_precheck_changelog_version_format_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "## [1.0.0]\n\ncontent").unwrap();
        let errors = precheck_version_changelog("v1.x", &path);
        assert!(errors.iter().any(|e| e.contains("版本号格式错误")));
    }

    #[test]
    fn test_precheck_changelog_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let errors = precheck_version_changelog("v1.0.0", &path);
        assert!(errors.iter().any(|e| e.contains("不存在")));
    }

    #[test]
    fn test_precheck_changelog_missing_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "## [1.0.0]\n\ncontent").unwrap();
        let errors = precheck_version_changelog("v2.0.0", &path);
        assert!(errors.iter().any(|e| e.contains("未找到")));
    }

    #[test]
    fn test_precheck_changelog_no_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "## [1.0.0]\n\ncontent").unwrap();
        let errors = precheck_version_changelog("v1.0.0", &path);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_precheck_changelog_pkg_version_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "## [2.0.0]\n\ncontent").unwrap();
        let errors = precheck_version_changelog("pkg/v2.0.0", &path);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_precheck_changelog_both_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "## [1.0.0]\n\ncontent").unwrap();
        let errors = precheck_version_changelog("bad", &path);
        assert_eq!(errors.len(), 2);
    }

    // ---- extract_notes ----

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
    fn test_extract_notes_empty_notes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "# Changelog\n\n## [1.0.0]\n\n").unwrap();
        assert!(extract_notes("v1.0.0", &path).is_none());
    }

    #[test]
    fn test_extract_notes_suffixed_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "# Changelog\n\n## [1.0.0-rc.1]\n\npre-release notes\n\n## [1.0.0]\n\nstable notes").unwrap();
        let notes = extract_notes("v1.0.0-rc.1", &path);
        assert!(notes.is_some());
        assert!(notes.unwrap().contains("pre-release"));
    }

    #[test]
    fn test_extract_notes_multiple_versions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "# Changelog\n\n## [3.0.0]\n\nthird\n\n## [2.0.0]\n\nsecond\n\n## [1.0.0]\n\nfirst"
        )
        .unwrap();
        let notes = extract_notes("v2.0.0", &path);
        assert!(notes.is_some());
        assert_eq!(notes.unwrap(), "second");
    }

    #[test]
    fn test_extract_notes_no_changelog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        assert!(extract_notes("v1.0.0", &path).is_none());
    }

    #[test]
    fn test_extract_notes_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CHANGELOG.md");
        std::fs::write(&path, "").unwrap();
        assert!(extract_notes("v1.0.0", &path).is_none());
    }

    // ---- confirm_release ----

    #[test]
    fn test_confirm_release_yes_flag() {
        assert!(confirm_release("v1.0.0", None, true));
    }

    #[test]
    fn test_confirm_release_yes_flag_with_notes() {
        assert!(confirm_release("v1.0.0", Some("release notes here"), true));
    }

    // ---- parse_github_repo ----

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
    fn test_parse_github_repo_org_subpath() {
        let url = "https://github.com/org-name/sub-team/repo.git";
        assert_eq!(parse_github_repo(url), None);
    }

    #[test]
    fn test_parse_github_repo_not_github() {
        let url = "https://gitlab.com/owner/repo.git";
        assert_eq!(parse_github_repo(url), None);
    }

    #[test]
    fn test_parse_github_repo_invalid_url() {
        assert_eq!(parse_github_repo(""), None);
        assert_eq!(parse_github_repo("not-a-url"), None);
        assert_eq!(parse_github_repo("github.com"), None);
        assert_eq!(parse_github_repo("https://example.com"), None);
    }

    // ---- get_remote_repo (parse_github_repo integration) ----

    #[test]
    fn test_parse_github_repo_ssh_no_git_suffix() {
        let url = "git@github.com:owner/repo";
        assert_eq!(parse_github_repo(url), Some("owner/repo".into()));
    }

    #[test]
    fn test_parse_github_repo_trailing_slash() {
        let url = "https://github.com/owner/repo/";
        assert_eq!(parse_github_repo(url), None);
    }

    // ---- confirm_release edge ----

    #[test]
    fn test_confirm_release_with_empty_notes() {
        assert!(confirm_release("v1.0.0", Some(""), true));
    }

    // ---- create_tag + rollback_tag in temp repo (single test to avoid CWD races) ----

    #[test]
    fn test_create_and_rollback_tag_in_temp_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir(&repo).unwrap();

        Command::new("git").args(["init"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["config", "user.name", "t"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["config", "user.email", "t@t"]).current_dir(&repo).output().unwrap();
        std::fs::write(repo.join("f"), "").unwrap();
        Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "x"]).current_dir(&repo).output().unwrap();

        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo).unwrap();

        assert!(create_tag("v0.1.0-single"));
        let out = Command::new("git").args(["tag", "-l"]).current_dir(&repo).output().unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("v0.1.0-single"));

        rollback_tag("v0.1.0-single");
        let out = Command::new("git").args(["tag", "-l"]).current_dir(&repo).output().unwrap();
        assert!(!String::from_utf8_lossy(&out.stdout).contains("v0.1.0-single"));

        std::env::set_current_dir(orig).unwrap();
    }
}
