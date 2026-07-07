/// 版本号自动检测 — 从 git 历史推断版本增量。
///
/// 收集 scope 下的 tag 和提交记录，按 devops-release skill 规则
/// 决定是否发版、minor/patch、预发布阶段。
///
/// 用法:
///   cargo run --bin detect -- <repo-path>
use std::path::Path;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    match detect_version(&repo_path) {
        Ok(Some(v)) => println!("🔮 建议版本: {}", v),
        Ok(None) => println!("无需发版"),
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

// ── 核心流程 ──────────────────────────────────────────────────

fn detect_version(repo_path: &Path) -> Result<Option<String>, String> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| format!("打开仓库失败: {}", e))?;
    let root = repo.workdir().unwrap_or(repo_path);

    let scopes = detect_scopes(root);
    if scopes.is_empty() {
        return Err("未检测到任何 scope".into());
    }

    for (scope_name, _dir) in &scopes {
        let result = detect_version_for_scope(root, scope_name)?;
        if result.is_some() {
            return Ok(result);
        }
    }
    Ok(None)
}

fn detect_version_for_scope(root: &Path, scope_name: &str) -> Result<Option<String>, String> {
    let tags = collect_tags_with_scope(root, scope_name);
    let latest_tag = tags.first().cloned();
    let commits = get_commits_since_tag(root, scope_name, &latest_tag)?;

    if commits.is_empty() {
        return Ok(None);
    }

    let decision = fallback_heuristic(&commits, &latest_tag)?;
    let version = build_version(&latest_tag, &decision, &commits)?;
    Ok(Some(version))
}

// ── 决策模型 ──────────────────────────────────────────────────

struct Decision {
    action: String,             // "release" | "skip"
    increment: Option<String>,  // "minor" | "patch"
    prerelease: Option<String>, // "rc" | null
}

fn fallback_heuristic(
    commits: &[String],
    _latest_tag: &Option<String>,
) -> Result<Decision, String> {
    let has_feat = commits.iter().any(|c| c.starts_with("feat:"));
    let has_fix = commits
        .iter()
        .any(|c| c.starts_with("fix:") || c.starts_with("refactor:"));
    let only_chore = commits
        .iter()
        .all(|c| c.starts_with("chore:") || c.starts_with("docs:") || c.starts_with("typo:"));

    if only_chore || commits.is_empty() {
        return Ok(Decision {
            action: "skip".into(),
            increment: None,
            prerelease: None,
        });
    }

    if has_feat {
        Ok(Decision {
            action: "release".into(),
            increment: Some("minor".into()),
            prerelease: Some("rc".into()),
        })
    } else if has_fix {
        Ok(Decision {
            action: "release".into(),
            increment: Some("patch".into()),
            prerelease: None,
        })
    } else {
        Ok(Decision {
            action: "skip".into(),
            increment: None,
            prerelease: None,
        })
    }
}

fn build_version(
    latest_tag: &Option<String>,
    decision: &Decision,
    _commits: &[String],
) -> Result<String, String> {
    if decision.action == "skip" {
        return Err("无需发版".into());
    }

    let (major, minor, patch, prerelease) = latest_tag
        .as_ref()
        .and_then(|t| parse_version(t))
        .unwrap_or((0, 0, 0, None));

    match decision.increment.as_deref() {
        Some("minor") => {
            let new_minor = minor + 1;
            match &decision.prerelease {
                Some(_) => Ok(format!("v{}.{}.{}-rc.1", major, new_minor, 0)),
                _ => Ok(format!("v{}.{}.{}", major, new_minor, 0)),
            }
        }
        Some("patch") => {
            let new_patch = patch + 1;
            match prerelease {
                Some(_) => {
                    // 已在预发布系列，递增序号
                    let (base, num) = prerelease
                        .as_ref()
                        .and_then(|p| p.rsplit_once('.'))
                        .unwrap_or(("rc", "0"));
                    let n: usize = num.parse().unwrap_or(0);
                    Ok(format!(
                        "v{}.{}.{}-{}.{}",
                        major,
                        minor,
                        new_patch,
                        base,
                        n + 1
                    ))
                }
                None => Ok(format!("v{}.{}.{}", major, minor, new_patch)),
            }
        }
        _ => Err("无效增量类型".into()),
    }
}

// ── scope 检测 ────────────────────────────────────────────────

fn detect_scopes(root: &Path) -> Vec<(String, String)> {
    let scopes = load_contract_scopes(root);
    if !scopes.is_empty() {
        return scopes;
    }
    vec![("root".into(), ".".into())]
}

fn load_contract_scopes(root: &Path) -> Vec<(String, String)> {
    for filename in &["contract.yaml", "contract.yml"] {
        let path = root.join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(scopes) = cfg.get("scopes").and_then(|s| s.as_mapping()) {
                    let mut result = Vec::new();
                    for (k, v) in scopes {
                        let name = k.as_str().unwrap_or("").to_string();
                        let dir = v
                            .get("dir")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        result.push((name, dir));
                    }
                    return result;
                }
            }
        }
    }
    vec![]
}

// ── Git 操作 ─────────────────────────────────────────────────

fn get_commits_since_tag(
    _root: &Path,
    _scope_name: &str,
    latest_tag: &Option<String>,
) -> Result<Vec<String>, String> {
    if latest_tag.is_none() {
        return Ok(vec!["feat: 初始版本".into()]);
    }
    // 简化：从 git log 获取提交
    let mut cmd = std::process::Command::new("git");
    cmd.args([
        "log",
        &format!("{}..HEAD", latest_tag.as_deref().unwrap()),
        "--oneline",
    ]);
    cmd.current_dir(_root);

    let output = cmd.output().map_err(|e| format!("git log 失败: {}", e))?;
    if !output.status.success() {
        return Ok(vec!["feat: 未知变更".into()]);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<String> = stdout
        .lines()
        .filter_map(|l| l.split_once(' ').map(|(_, msg)| msg.to_string()))
        .collect();
    Ok(commits)
}

fn collect_tags_with_scope(root: &Path, scope_name: &str) -> Vec<String> {
    let repo = match git2::Repository::open(root) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut tags = Vec::new();
    let _ = repo.tag_foreach(|_oid, name| {
        let s = String::from_utf8_lossy(name).to_string();
        tags.push(s);
        true
    });
    let prefix = format!("{}/", scope_name);
    let mut filtered: Vec<String> = tags
        .into_iter()
        .filter(|t| t.starts_with(&prefix) || !t.contains('/'))
        .collect();
    filtered.sort_by(|a, b| b.cmp(a));
    filtered
}

fn parse_version(version_str: &str) -> Option<(u64, u64, u64, Option<String>)> {
    let v = version_str.strip_prefix('v').unwrap_or(version_str);
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    if parts.len() < 2 {
        return None;
    }
    let major = parts.get(0)?.parse().ok()?;
    let minor = parts.get(1)?.parse().ok()?;
    let (patch, prerelease) = if let Some(p) = parts.get(2) {
        if let Some((num, pre)) = p.split_once('-') {
            (num.parse().ok()?, Some(pre.to_string()))
        } else {
            (p.parse().ok()?, None)
        }
    } else {
        (0, None)
    };
    Some((major, minor, patch, prerelease))
}

// ── 测试 ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_formal() {
        let (maj, min, pat, pre) = parse_version("v1.2.3").unwrap();
        assert_eq!((maj, min, pat), (1, 2, 3));
        assert!(pre.is_none());
    }

    #[test]
    fn test_parse_version_prerelease() {
        let (maj, min, pat, pre) = parse_version("v0.1.0-rc.1").unwrap();
        assert_eq!((maj, min, pat), (0, 1, 0));
        assert_eq!(pre, Some("rc.1".into()));
    }

    #[test]
    fn test_parse_version_bad_format() {
        assert!(parse_version("invalid").is_none());
    }

    #[test]
    fn test_fallback_heuristic_feat() {
        let d = fallback_heuristic(&["feat: new command".into()], &None).unwrap();
        assert_eq!(d.action, "release");
        assert_eq!(d.increment.as_deref(), Some("minor"));
    }

    #[test]
    fn test_fallback_heuristic_fix() {
        let d = fallback_heuristic(&["fix: bug fix".into()], &None).unwrap();
        assert_eq!(d.action, "release");
        assert_eq!(d.increment.as_deref(), Some("patch"));
    }

    #[test]
    fn test_fallback_heuristic_skip_chore() {
        let d = fallback_heuristic(&["chore: cleanup".into()], &None).unwrap();
        assert_eq!(d.action, "skip");
    }

    #[test]
    fn test_build_version_minor_rc() {
        let d = Decision {
            action: "release".into(),
            increment: Some("minor".into()),
            prerelease: Some("rc".into()),
        };
        let v = build_version(&None, &d, &[]).unwrap();
        assert_eq!(v, "v0.1.0-rc.1");
    }

    #[test]
    fn test_build_version_patch() {
        let d = Decision {
            action: "release".into(),
            increment: Some("patch".into()),
            prerelease: None,
        };
        let v = build_version(&Some("v0.2.0".into()), &d, &[]).unwrap();
        assert_eq!(v, "v0.2.1");
    }

    #[test]
    fn test_build_version_skip() {
        let d = Decision {
            action: "skip".into(),
            increment: None,
            prerelease: None,
        };
        assert!(build_version(&None, &d, &[]).is_err());
    }

    #[test]
    fn test_load_contract_scopes_empty() {
        let d = std::env::temp_dir().join("test_detect_empty");
        let _ = std::fs::create_dir_all(&d);
        assert!(load_contract_scopes(&d).is_empty());
    }
}
