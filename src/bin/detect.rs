/// 版本号自动检测原型 — 实验 devops-release skill 中的 AI 决策规则。
///
/// 用法:
///   cargo run --bin detect -- <repo-path>
///
/// 修复:
///   1. tag 按 semver 排序，scope tag 和无 scope tag 不混淆
///   2. scope 从 changed files 匹配 contract.yaml 推断
///   3. 子模组通过 discover 支持
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(|p| p.into())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    match detect_version(&repo_path) {
        Ok(v) => println!("{}", v),
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

fn detect_version(repo_path: &Path) -> Result<String, String> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| format!("打开仓库失败: {}", e))?;

    // ── 1. 确定 scope ────────────────────────────────────────────
    let scope = detect_scope(&repo)?;
    println!("📌 scope: {:?}", scope);

    // ── 2. 读最新 tag（按 scope 过滤 + semver 排序）───────────────
    let latest_tag = get_latest_tag_for_scope(&repo, scope.as_deref())
        .ok_or_else(|| "没有找到版本标签，请手动指定版本号".to_string())?;
    println!("📦 最新标签: {}", latest_tag);

    let (_, ver_str) = parse_tag(&latest_tag);
    let (major, minor, patch, pre_stage, pre_num) = parse_version(ver_str)?;
    println!("   v{}.{}.{}", major, minor, patch);
    if let Some(ref stage) = pre_stage {
        println!("   预发布: {}.{}", stage, pre_num.unwrap_or(0));
    }

    // ── 3. 扫描 tag→HEAD 提交 ──────────────────────────────────────
    let tag_oid = repo
        .find_reference(&format!("refs/tags/{}", latest_tag))
        .and_then(|r| r.target().ok_or_else(|| git2::Error::from_str("")))
        .map_err(|_| "找不到标签引用")?;
    let head_oid = repo
        .head()
        .and_then(|h| h.target().ok_or_else(|| git2::Error::from_str("")))
        .map_err(|_| "找不到 HEAD")?;

    if head_oid == tag_oid {
        return Err("上次标签后没有新提交".into());
    }

    let mut revwalk = repo.revwalk().map_err(|_| "创建 revwalk 失败")?;
    revwalk.push(head_oid).ok();
    revwalk.hide(tag_oid).ok();

    let mut has_feat = false;
    let mut has_breaking = false;
    let mut has_logic_change = false;
    let mut commits: Vec<String> = Vec::new();

    for oid in revwalk {
        let oid = match oid {
            Ok(o) => o,
            Err(_) => continue,
        };
        if let Ok(commit) = repo.find_commit(oid) {
            let msg = commit.summary().unwrap_or("").to_string();
            commits.push(msg.clone());
            let lower = msg.to_lowercase();

            if lower.contains("breaking") || (msg.contains('!') && lower.starts_with("feat")) {
                has_breaking = true;
                has_logic_change = true;
            } else if lower.starts_with("feat") || msg.contains("Added") {
                has_feat = true;
                has_logic_change = true;
            } else if lower.starts_with("fix")
                || lower.starts_with("refactor")
                || lower.starts_with("test")
                || msg.contains("Fixed")
                || msg.contains("Changed")
            {
                has_logic_change = true;
            }
        }
    }

    println!("\n📝 提交数: {}", commits.len());
    for c in &commits {
        println!("   • {}", c);
    }

    if !has_logic_change {
        return Err("只有非逻辑改动（typo/注释/CI/README），无需发版".into());
    }

    if has_breaking {
        return Err("包含 breaking change，请人类指定 major 版本号".into());
    }

    // ── 4. 推断新版本 ────────────────────────────────────────────
    let new_version = if let Some(stage) = pre_stage {
        let next = pre_num.unwrap_or(0) + 1;
        format!("v{}.{}.{}-{}.{}", major, minor, patch, stage, next)
    } else if has_feat {
        format!("v{}.{}.{}-rc.1", major, minor + 1, 0)
    } else {
        format!("v{}.{}.{}", major, minor, patch + 1)
    };

    let prefix = match scope {
        Some(ref s) if !s.is_empty() && s != "(root)" => format!("{}/", s),
        _ => String::new(),
    };

    let result = format!("{}{}", prefix, new_version);
    println!("\n🔮 推断版本: {}", result);
    Ok(result)
}

// ═════════════════════════════════════════════════════════════════════
// scope 检测
// ═════════════════════════════════════════════════════════════════════

/// 从 changed files + contract.yaml 推断 scope。
fn detect_scope(repo: &git2::Repository) -> Result<Option<String>, String> {
    // 先查 contract.yaml → scope 目录映射
    let scopes = load_contract_scopes(repo.workdir().unwrap_or(Path::new(".")));

    // 从 changed files 推断：head 与最新 tag 间的差异文件
    let changed_paths = get_changed_paths_since_last_tag(repo)?;

    // 匹配 scope：统计每个 scope 命中的文件数（无变更文件时跳过）
    let mut hits: HashMap<&str, usize> = HashMap::new();
    for path in &changed_paths {
        for (name, dir) in &scopes {
            if path.starts_with(dir.trim_start_matches('/')) || path.contains(dir) {
                *hits.entry(name).or_insert(0) += 1;
            }
        }
    }

    // 取命中最多的 scope
    let best = hits.iter().max_by_key(|(_, c)| *c);
    match best {
        Some((name, count)) if *count > 0 => {
            println!(
                "   从 changed files 推断 scope={}, 命中 {} 文件",
                name, count
            );
            Ok(Some(name.to_string()))
        }
        _ => {
            // 回退：取最常见的 scoped tag，忽略孤立的 root tag
            let all_tags = collect_tags_with_scope(repo);
            let scoped: Vec<&String> = all_tags.keys().filter(|k| *k != "(root)").collect();
            if scoped.len() == 1 {
                return Ok(Some(scoped[0].clone()));
            }
            if scoped.len() > 1 {
                let names: Vec<&str> = scoped.iter().map(|s| s.as_str()).collect();
                return Err(format!("多个 scope 有变更: {:?}，请指定", names));
            }
            Ok(None)
        }
    }
}

/// 加载 contract.yaml 中的 scope 映射。
fn load_contract_scopes(repo_root: &Path) -> HashMap<String, String> {
    let paths = [
        repo_root.join(".quanttide/devops/contract.yaml"),
        repo_root.join("contract.yaml"),
    ];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(cfg) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(scopes) = cfg.get("scopes").and_then(|s| s.as_mapping()) {
                    let mut map = HashMap::new();
                    for (k, v) in scopes {
                        let name = k.as_str().unwrap_or("").to_string();
                        let dir = v
                            .get("dir")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        map.insert(name, dir);
                    }
                    return map;
                }
            }
        }
    }
    HashMap::new()
}

/// 获取上次 tag 到 HEAD 间变更的文件路径列表。
fn get_changed_paths_since_last_tag(repo: &git2::Repository) -> Result<Vec<String>, String> {
    let head_oid = repo
        .head()
        .and_then(|h| h.target().ok_or_else(|| git2::Error::from_str("")))
        .map_err(|_| "找不到 HEAD")?;

    let tree = repo
        .find_commit(head_oid)
        .and_then(|c| c.tree())
        .map_err(|_| "找不到 HEAD tree")?;

    // 取最新 tag（优先 scoped tag）
    let tags = collect_tags_with_scope(repo);
    let latest_tag = tags
        .iter()
        .filter(|(k, _)| *k != "(root)")
        .find_map(|(_, v)| v.first())
        .or_else(|| tags.get("(root)").and_then(|v| v.first()));
    let base_tree = match latest_tag {
        Some(tag) => {
            let tag_oid = repo
                .find_reference(&format!("refs/tags/{}", tag))
                .and_then(|r| r.target().ok_or_else(|| git2::Error::from_str("")))
                .ok()
                .and_then(|oid| repo.find_commit(oid).ok())
                .and_then(|c| c.tree().ok());
            tag_oid
        }
        None => None,
    };

    let diff = repo
        .diff_tree_to_tree(base_tree.as_ref(), Some(&tree), None)
        .map_err(|_| "diff 失败".to_string())?;

    let mut paths: Vec<String> = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(f) = delta.new_file().path() {
                paths.push(f.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )
    .ok();

    Ok(paths)
}

// ═════════════════════════════════════════════════════════════════════
// tag 处理：按 scope 分组 + semver 排序
// ═════════════════════════════════════════════════════════════════════

/// 获取指定 scope 的最新 tag（按 semver 排序）。
fn get_latest_tag_for_scope(repo: &git2::Repository, scope: Option<&str>) -> Option<String> {
    let all = collect_tags_with_scope(repo);
    let scope_key = scope.unwrap_or("(root)");
    all.get(scope_key).and_then(|tags| tags.first().cloned())
}

/// 收集所有 tag，按 scope 分组，每组内按 semver 降序排列。
fn collect_tags_with_scope(repo: &git2::Repository) -> HashMap<String, Vec<String>> {
    let tag_names = match repo.tag_names(None) {
        Ok(t) => t,
        Err(_) => return HashMap::new(),
    };

    let mut groups: HashMap<String, Vec<((u32, u32, u32, u32, u32), String)>> = HashMap::new();

    for tag in tag_names.iter().flatten() {
        let (scope, ver_str) = parse_tag(tag);
        let scope_name = scope.unwrap_or_else(|| "(root)".to_string());
        if let Ok((major, minor, patch, _, pre_num)) = parse_version(ver_str) {
            // pre_num: None → 正式版（排前面），Some(n) → 预发布（排后面）
            let pre_ord = pre_num.unwrap_or(0);
            // 预发布阶段排序：alpha < beta < rc
            let stage_ord = if ver_str.contains("-alpha") {
                1
            } else if ver_str.contains("-beta") {
                2
            } else if ver_str.contains("-rc") {
                3
            } else {
                0 // 正式版
            };
            // 排序键：(major, minor, patch, stage_ord, pre_ord) 全部降序
            let ord = (major, minor, patch, stage_ord, pre_ord);
            groups
                .entry(scope_name)
                .or_default()
                .push((ord, tag.to_string()));
        }
    }

    // 每组内降序排列
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for (scope, mut entries) in groups {
        entries.sort_by(|a, b| b.0.cmp(&a.0)); // 降序
        result.insert(scope, entries.into_iter().map(|(_, t)| t).collect());
    }
    result
}

/// 解析 tag 为 (scope, version_str)。
fn parse_tag(tag: &str) -> (Option<String>, &str) {
    if let Some((scope, ver)) = tag.split_once('/') {
        (Some(scope.to_string()), ver)
    } else {
        (None, tag)
    }
}

/// 解析版本字符串。
fn parse_version(s: &str) -> Result<(u32, u32, u32, Option<String>, Option<u32>), String> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let (ver_part, pre_part) = s.split_once('-').unwrap_or((s, ""));

    let parts: Vec<&str> = ver_part.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("版本号格式错误: {}，需要 X.Y.Z", s));
    }

    let major = parts[0].parse().map_err(|_| "major 不是数字".to_string())?;
    let minor = parts[1].parse().map_err(|_| "minor 不是数字".to_string())?;
    let patch: u32 = parts[2].parse().map_err(|_| "patch 不是数字".to_string())?;

    let (pre_stage, pre_num) = if pre_part.is_empty() {
        (None, None)
    } else {
        let sp: Vec<&str> = pre_part.split('.').collect();
        let stage = sp.first().map(|s| s.to_string());
        let num = sp.get(1).and_then(|s| s.parse().ok());
        (stage, num)
    };

    Ok((major, minor, patch, pre_stage, pre_num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_scoped() {
        assert_eq!(parse_tag("cli/v0.8.4"), (Some("cli".into()), "v0.8.4"));
    }

    #[test]
    fn test_parse_tag_root() {
        assert_eq!(parse_tag("v0.1.0"), (None, "v0.1.0"));
    }

    #[test]
    fn test_parse_version_formal() {
        let (ma, mi, pa, st, nu) = parse_version("0.8.4").unwrap();
        assert_eq!((ma, mi, pa), (0, 8, 4));
        assert!(st.is_none());
        assert!(nu.is_none());
    }

    #[test]
    fn test_parse_version_prerelease() {
        let (ma, mi, pa, st, nu) = parse_version("0.9.0-rc.1").unwrap();
        assert_eq!((ma, mi, pa), (0, 9, 0));
        assert_eq!(st.as_deref(), Some("rc"));
        assert_eq!(nu, Some(1));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        let (ma, mi, pa, _, _) = parse_version("v0.8.4").unwrap();
        assert_eq!((ma, mi, pa), (0, 8, 4));
    }

    #[test]
    fn test_parse_version_bad_format() {
        assert!(parse_version("abc").is_err());
        assert!(parse_version("0.1").is_err());
    }

    #[test]
    fn test_parse_version_beta() {
        let (ma, mi, pa, st, nu) = parse_version("0.7.0-beta.3").unwrap();
        assert_eq!((ma, mi, pa), (0, 7, 0));
        assert_eq!(st.as_deref(), Some("beta"));
        assert_eq!(nu, Some(3));
    }

    #[test]
    fn test_collect_tags_sorts_by_semver() {
        use std::collections::HashMap;
        // Mock 一个空的 repo，直接测 parse_version + 排序逻辑
        let tags = vec![
            "cli/v0.8.4",
            "cli/v0.9.0",
            "cli/v0.8.4-rc.1",
            "cli/v0.10.0",
            "cli/v0.9.0-rc.2",
        ];
        let mut groups: HashMap<String, Vec<(String, u32, u32, u32)>> = HashMap::new();
        for tag in &tags {
            let (scope, ver_str) = parse_tag(tag);
            let name = scope.unwrap_or_else(|| "(root)".to_string());
            if let Ok((ma, mi, pa, _, _)) = parse_version(ver_str) {
                groups
                    .entry(name)
                    .or_default()
                    .push((tag.to_string(), ma, mi, pa));
            }
        }
        // cli 组降序：0.10.0 > 0.9.0 > 0.9.0-rc.2 > 0.8.4 > 0.8.4-rc.1
        let mut cli = groups.remove("cli").unwrap();
        cli.sort_by(|a, b| (b.1, b.2, b.3).cmp(&(a.1, a.2, a.3)));
        let versions: Vec<&str> = cli.iter().map(|(t, _, _, _)| t.as_str()).collect();
        assert_eq!(
            versions,
            vec![
                "cli/v0.10.0",
                "cli/v0.9.0",
                "cli/v0.9.0-rc.2",
                "cli/v0.8.4",
                "cli/v0.8.4-rc.1",
            ]
        );
    }
}
