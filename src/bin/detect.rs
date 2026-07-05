/// 版本号自动检测 — LLM 驱动的版本号推断。
///
/// 保留基础设施函数（tag 读取、提交扫描、scope 检测），
/// 将版本增量决策交给 LLM，覆盖 devops-release skill 中
/// 硬编码规则无法处理的场景（变更规模、预发布阶段、阶段晋级等）。
///
/// 多个 scope 有变更时各自独立判断。
///
/// 用法:
///   cargo run --bin detect -- <repo-path>
///
/// 依赖环境变量:
///   LLM_API_KEY  — DeepSeek API Key（可选，未设置时回退到启发式规则）
///   LLM_MODEL    — 模型名（默认 deepseek-chat）
///   LLM_BASE_URL — API 地址（默认 https://api.deepseek.com）
use quanttide_agent::llm::{CompleteOptions, LLM};
use quanttide_agent::message::Message;
use quanttide_agent::Settings;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(|p| p.into())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    match detect_version(&repo_path) {
        Ok(true) => {} // 结果已在内部打印
        Ok(false) => println!("无需发版"),
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

/// LLM 决策输出。
#[derive(serde::Deserialize)]
struct LlmDecision {
    action: String,             // "release" | "skip" | "human"
    increment: Option<String>,  // "minor" | "patch" | null
    prerelease: Option<String>, // "alpha" | "beta" | "rc" | null
    reason: String,
}

/// 调度层：检测所有有变更的 scope，各自独立运行完整版本推断管道。
fn detect_version(repo_path: &Path) -> Result<bool, String> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| format!("打开仓库失败: {}", e))?;

    let project_type = detect_project_type(&repo);
    println!("📌 项目类型: {}", project_type);

    let scopes = detect_scopes(&repo)?;
    if scopes.is_empty() {
        return Err("没有找到匹配的 scope".into());
    }

    println!("📌 检测到 scope: {:?}", scopes);

    let mut results: Vec<(String, String)> = Vec::new();

    for scope in &scopes {
        match detect_version_for_scope(&repo, scope, project_type) {
            Ok(Some(v)) => {
                results.push((scope.clone(), v));
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("   ⚠ {}: {}", scope, e);
            }
        }
    }

    if results.is_empty() {
        return Ok(false);
    }

    if results.len() == 1 {
        let (s, v) = &results[0];
        let prefixed = prefix_version(s, v);
        println!("\n🔮 推断版本: {}", prefixed);
    } else {
        println!("\n🔮 各 scope 推断结果:");
        for (s, v) in &results {
            println!("   {}/{}", s, v);
        }
    }

    Ok(true)
}

/// 单个 scope 的版本推断管道。
fn detect_version_for_scope(
    repo: &git2::Repository,
    scope: &str,
    project_type: &str,
) -> Result<Option<String>, String> {
    println!("\n--- {} ---", scope);

    // ── 1. 读最新 tag（可能没有）───────────────────────────────────
    let head_oid = repo
        .head()
        .and_then(|h| h.target().ok_or_else(|| git2::Error::from_str("")))
        .map_err(|_| "找不到 HEAD")?;

    let (latest_tag, major, minor, patch, pre_stage, pre_num, is_first) =
        match get_latest_tag_for_scope(repo, Some(scope)) {
            Some(ref tag) => {
                let (_, ver_str) = parse_tag(tag);
                let (ma, mi, pa, st, nu) = parse_version(ver_str)?;
                println!("📦 最新标签: {}", tag);
                println!("   v{}.{}.{}", ma, mi, pa);
                if let Some(ref stage) = st {
                    println!("   预发布: {}.{}", stage, nu.unwrap_or(0));
                }
                (Some(tag.clone()), ma, mi, pa, st, nu, false)
            }
            None => {
                println!("📦 没有版本标签（新项目）");
                (None, 0, 1, 0, None, None, true)
            }
        };

    // ── 2. 扫描提交 ──────────────────────────────────────────────────
    let mut revwalk = repo.revwalk().map_err(|_| "创建 revwalk 失败")?;
    revwalk.push(head_oid).ok();
    if let Some(ref tag) = latest_tag {
        let tag_oid = repo
            .find_reference(&format!("refs/tags/{}", tag))
            .and_then(|r| r.target().ok_or_else(|| git2::Error::from_str("")))
            .map_err(|_| "找不到标签引用")?;
        if head_oid == tag_oid {
            return Err("上次标签后没有新提交".into());
        }
        revwalk.hide(tag_oid).ok();
    }

    let mut commits: Vec<String> = Vec::new();
    for oid in revwalk {
        let oid = match oid {
            Ok(o) => o,
            Err(_) => continue,
        };
        if let Ok(commit) = repo.find_commit(oid) {
            let msg = commit.summary().unwrap_or("").to_string();
            commits.push(msg);
        }
    }

    println!("📝 提交数: {}", commits.len());
    for c in &commits {
        println!("   • {}", c);
    }

    if commits.is_empty() {
        return Err("没有提交记录".into());
    }

    // ── 3. LLM 推断版本（回退到启发式规则）───────────────────────────
    let llm_tag = latest_tag.as_deref().unwrap_or("(新项目，无版本标签)");
    let decision = llm_decide(&commits, llm_tag, project_type, scope)?;

    println!("🧠 LLM 决策: {}", decision.reason);
    match decision.action.as_str() {
        "skip" => {
            if is_first {
                // 新项目：首个版本始终发 v0.1.0，不论提交类型
            } else {
                return Ok(None);
            }
        }
        "human" => return Err(format!("需要人类判断: {}", decision.reason)),
        _ => {}
    }

    let new_version = if is_first {
        // 新项目：首个版本固定为 v0.1.0，预发布阶段由 LLM 决定
        match decision.prerelease.as_deref() {
            Some(pr) => format!("v0.1.0-{}.1", pr),
            None => "v0.1.0".to_string(),
        }
    } else {
        let increment = decision.increment.as_deref().unwrap_or("patch");
        build_version(
            major,
            minor,
            patch,
            pre_stage.as_deref(),
            pre_num,
            increment,
            decision.prerelease.as_deref(),
        )
    };

    println!("🔮 推断版本: {}", prefix_version(scope, &new_version));
    Ok(Some(new_version))
}

/// 调用 LLM 决定版本策略。未配置 LLM 时回退到启发式规则。
fn llm_decide(
    commits: &[String],
    latest_tag: &str,
    project_type: &str,
    scope: &str,
) -> Result<LlmDecision, String> {
    let settings = Settings::from_env();
    if settings.llm_api_key.is_empty() {
        return Ok(fallback_heuristic(commits));
    }

    let llm = LLM::new(
        &settings.llm_model,
        &settings.llm_base_url,
        &settings.llm_api_key,
    );

    let commits_text = commits
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}. {}", i + 1, c))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"你是一个版本号推断专家。根据以下信息，决定下一个版本号策略。

## 约束
- 不做 major bump（breaking change 交给人类）
- 仅 chore/typo/CI 配置 → skip
- `docs:` 是内容变更（文档项目的交付物），不是非逻辑改动
- patch 级别修复 → 直发正式版
- minor 级别新功能 → 代码项目走预发布（rc），文档项目直发正式
- 大版本早期未完成功能 → alpha
- 功能基本完成 → beta
- 功能冻结只修 bug → rc
- 已在预发布系列 → 同阶段递增序号（除非有理由晋级下一阶段）

### 如何判断 minor vs patch

**代码项目：**
- `feat:` → minor（追加新能力）
- `fix: / refactor: / test:` → patch（修问题）

**内容/文档项目：**
- **绝大多数变更都是 patch**。新增文档、更新内容、格式规范化、目录结构调整都是日常工作。
- minor 仅限全新内容品类上线的程度（例如从零搭建了一整套新手册），极少发生。
- 不确定时就 patch。

## 当前版本
项目类型: {project_type}
最新 tag: {tag}
scope: {scope}

## 提交记录（tag→HEAD）
{commits}

## 输出格式（仅 JSON）
{{"action": "release"|"skip"|"human", "increment": "minor"|"patch"|null, "prerelease": "alpha"|"beta"|"rc"|null, "reason": "判断理由"}}
"#,
        tag = latest_tag,
        scope = scope,
        project_type = project_type,
        commits = commits_text,
    );

    let messages = vec![
        Message::new(
            "system",
            "你是一个严格的版本号推断工具。只输出 JSON，不要额外内容。",
        ),
        Message::new("user", &prompt),
    ];

    let options = CompleteOptions {
        response_format: Some(serde_json::json!({"type": "json_object"})),
        ..Default::default()
    };

    let resp = llm
        .complete(&messages, options)
        .map_err(|e| format!("LLM 调用失败: {}", e.0))?;

    let decision: LlmDecision = serde_json::from_str(&resp.content)
        .map_err(|e| format!("LLM 输出解析失败: {} — 原始输出: {}", e, resp.content))?;

    Ok(decision)
}

/// 启发式回退规则。
///
/// 区分 feat（minor）和非 feat（patch），但统一直发正式无预发布。
/// 预发布仅由 LLM 在有项目类型上下文时决定。
///
/// 规则：
/// - breaking → 交给人类
/// - `feat:` → minor，直发正式
/// - `fix:/docs:/refactor:/test:` → patch，直发正式
/// - 仅 chore/typo/CI → 跳过
fn fallback_heuristic(commits: &[String]) -> LlmDecision {
    let mut has_feat = false;
    let mut has_breaking = false;
    let mut has_logic_change = false;

    for msg in commits {
        let lower = msg.to_lowercase();
        if lower.contains("breaking") || (msg.contains('!') && lower.starts_with("feat")) {
            has_breaking = true;
            has_logic_change = true;
        } else if lower.starts_with("feat") || msg.contains("Added") {
            has_feat = true;
            has_logic_change = true;
        } else if lower.starts_with("fix")
            || lower.starts_with("docs")
            || lower.starts_with("refactor")
            || lower.starts_with("test")
            || msg.contains("Fixed")
            || msg.contains("Changed")
        {
            has_logic_change = true;
        }
    }

    if !has_logic_change {
        return LlmDecision {
            action: "skip".into(),
            increment: None,
            prerelease: None,
            reason: "仅有 chore/typo/CI 改动，无需发版".into(),
        };
    }

    if has_breaking {
        return LlmDecision {
            action: "human".into(),
            increment: None,
            prerelease: None,
            reason: "包含 breaking change，请人类指定 major 版本号".into(),
        };
    }

    if has_feat {
        LlmDecision {
            action: "release".into(),
            increment: Some("minor".into()),
            prerelease: None,
            reason: "包含 feat，minor 增量直发正式".into(),
        }
    } else {
        LlmDecision {
            action: "release".into(),
            increment: Some("patch".into()),
            prerelease: None,
            reason: "包含 docs/fix/refactor，patch 增量直发正式".into(),
        }
    }
}

/// 根据决策构建版本字符串（不含 scope 前缀）。
fn build_version(
    major: u32,
    minor: u32,
    patch: u32,
    pre_stage: Option<&str>,
    pre_num: Option<u32>,
    increment: &str,
    prerelease: Option<&str>,
) -> String {
    if let Some(stage) = pre_stage {
        // 已在预发布系列 → 同阶段递增序号
        let next = pre_num.unwrap_or(0) + 1;
        return format!("v{}.{}.{}-{}.{}", major, minor, patch, stage, next);
    }

    match (increment, prerelease) {
        ("minor", Some(pr)) => format!("v{}.{}.{}-{}.1", major, minor + 1, 0, pr),
        ("minor", None) => format!("v{}.{}.{}", major, minor + 1, 0),
        _ => format!("v{}.{}.{}", major, minor, patch + 1),
    }
}

/// 为版本字符串添加 scope 前缀。
fn prefix_version(scope: &str, version: &str) -> String {
    if scope.is_empty() || scope == "(root)" {
        version.to_string()
    } else {
        format!("{}/{}", scope, version)
    }
}

// ═════════════════════════════════════════════════════════════════════
// 项目类型检测
// ═════════════════════════════════════════════════════════════════════

/// 检测项目类型：code（代码项目）或 docs（文档项目）。
///
/// 简单规则：仓库根目录存在 src/、Cargo.toml、package.json 等
/// 代码指示物 → code，否则 → docs。
fn detect_project_type(repo: &git2::Repository) -> &'static str {
    let workdir = match repo.workdir() {
        Some(d) => d,
        None => return "unknown",
    };

    let indicators = [
        workdir.join("src").is_dir(),
        workdir.join("Cargo.toml").exists(),
        workdir.join("package.json").exists(),
        workdir.join("pyproject.toml").exists(),
        workdir.join("setup.py").exists(),
        workdir.join("go.mod").exists(),
        workdir.join("packages").is_dir(), // monorepo
        workdir.join("apps").is_dir(),     // monorepo
    ];

    if indicators.iter().any(|&x| x) {
        "code"
    } else {
        "docs"
    }
}

// ═════════════════════════════════════════════════════════════════════
// scope 检测
// ═════════════════════════════════════════════════════════════════════

/// 从 changed files + contract.yaml 检测所有有变更的 scope。
fn detect_scopes(repo: &git2::Repository) -> Result<Vec<String>, String> {
    let scopes = load_contract_scopes(repo.workdir().unwrap_or(Path::new(".")));
    let changed_paths = get_changed_paths_since_last_tag(repo)?;

    // 从 changed files 匹配 scope
    let mut hits: HashMap<String, usize> = HashMap::new();
    for path in &changed_paths {
        for (name, dir) in &scopes {
            if path.starts_with(dir.trim_start_matches('/')) || path.contains(dir) {
                *hits.entry(name.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut matched: Vec<String> = hits.into_keys().collect();
    if !matched.is_empty() {
        // 按被匹配的次数降序排列
        matched.sort();
        return Ok(matched);
    }

    // 回退：从已有 tag 收集 scope
    let all_tags = collect_tags_with_scope(repo);
    let scoped: Vec<&String> = all_tags.keys().filter(|k| *k != "(root)").collect();
    if !scoped.is_empty() {
        return Ok(scoped.into_iter().cloned().collect());
    }

    // 最后回退：(root) — 让后续流程给出具体错误
    Ok(vec!["(root)".into()])
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
            let pre_ord = pre_num.unwrap_or(0);
            let stage_ord = if ver_str.contains("-alpha") {
                1
            } else if ver_str.contains("-beta") {
                2
            } else if ver_str.contains("-rc") {
                3
            } else {
                0
            };
            let ord = (major, minor, patch, stage_ord, pre_ord);
            groups
                .entry(scope_name)
                .or_default()
                .push((ord, tag.to_string()));
        }
    }

    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for (scope, mut entries) in groups {
        entries.sort_by(|a, b| b.0.cmp(&a.0));
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

    #[test]
    fn test_fallback_heuristic_feat() {
        let commits = vec!["feat: add new command".into()];
        let d = fallback_heuristic(&commits);
        assert_eq!(d.action, "release");
        assert_eq!(d.increment.as_deref(), Some("minor"));
        assert!(d.prerelease.is_none());
    }

    #[test]
    fn test_fallback_heuristic_fix() {
        let commits = vec!["fix: resolve crash".into()];
        let d = fallback_heuristic(&commits);
        assert_eq!(d.action, "release");
        assert_eq!(d.increment.as_deref(), Some("patch"));
        assert!(d.prerelease.is_none());
    }

    #[test]
    fn test_fallback_heuristic_docs() {
        let commits = vec!["docs: update readme".into()];
        let d = fallback_heuristic(&commits);
        assert_eq!(d.action, "release");
        assert_eq!(d.increment.as_deref(), Some("patch"));
        assert!(d.prerelease.is_none());
    }

    #[test]
    fn test_fallback_heuristic_skip() {
        let commits = vec!["chore: bump version".into()];
        let d = fallback_heuristic(&commits);
        assert_eq!(d.action, "skip");
    }

    #[test]
    fn test_fallback_heuristic_breaking() {
        let commits = vec!["feat!: breaking change".into()];
        let d = fallback_heuristic(&commits);
        assert_eq!(d.action, "human");
    }

    #[test]
    fn test_build_version_patch() {
        let v = build_version(0, 8, 4, None, None, "patch", None);
        assert_eq!(v, "v0.8.5");
    }

    #[test]
    fn test_build_version_minor_rc() {
        let v = build_version(0, 8, 4, None, None, "minor", Some("rc"));
        assert_eq!(v, "v0.9.0-rc.1");
    }

    #[test]
    fn test_build_version_prerelease_increment() {
        let v = build_version(0, 9, 0, Some("rc"), Some(1), "patch", None);
        assert_eq!(v, "v0.9.0-rc.2");
    }

    #[test]
    fn test_build_version_minor_formal() {
        let v = build_version(0, 8, 4, None, None, "minor", None);
        assert_eq!(v, "v0.9.0");
    }

    #[test]
    fn test_prefix_version_scoped() {
        assert_eq!(prefix_version("cli", "v0.9.0"), "cli/v0.9.0");
    }

    #[test]
    fn test_prefix_version_root() {
        assert_eq!(prefix_version("(root)", "v0.9.0"), "v0.9.0");
    }

    #[test]
    fn test_prefix_version_empty() {
        assert_eq!(prefix_version("", "v0.9.0"), "v0.9.0");
    }
}
