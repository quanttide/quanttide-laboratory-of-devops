/// plan 命令实验原型：ROADMAP.md 管理。
///
/// 对应 `data/roadmap/platform/plan-command.md` 中的三个子命令。
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════
// plan status
// ═══════════════════════════════════════════════════════════════════════

/// 单个版本的规划进度。
#[derive(Debug)]
pub struct VersionProgress {
    pub version: String,
    pub done: usize,
    pub total: usize,
}

/// 解析 ROADMAP.md，返回各版本进度列表。
pub fn parse_roadmap(path: &Path) -> Vec<VersionProgress> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut versions: Vec<VersionProgress> = Vec::new();
    let mut current_version: Option<String> = None;
    let mut done = 0usize;
    let mut total = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## [") && trimmed.ends_with(']') {
            if let Some(ver) = current_version.take() {
                versions.push(VersionProgress {
                    version: ver,
                    done,
                    total,
                });
            }
            done = 0;
            total = 0;
            let ver = trimmed
                .trim_start_matches("## [")
                .trim_end_matches(']')
                .trim()
                .trim_start_matches('v')
                .to_string();
            current_version = Some(ver);
            continue;
        }

        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            total += 1;
            done += 1;
            continue;
        }

        if trimmed.starts_with("- [ ]") {
            total += 1;
            continue;
        }
    }

    if let Some(ver) = current_version {
        versions.push(VersionProgress {
            version: ver,
            done,
            total,
        });
    }
    versions
}

/// 格式化输出进度。
pub fn print_status(path: &Path) {
    let versions = parse_roadmap(path);
    if versions.is_empty() {
        println!("  📋 未找到规划条目");
        return;
    }

    let mut total_done = 0usize;
    let mut total_all = 0usize;

    for v in &versions {
        let rate = if v.total > 0 {
            v.done as f64 / v.total as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "  [{:<8}] {:>2}/{:>2} 完成 ({:.0}%)",
            v.version, v.done, v.total, rate
        );
        total_done += v.done;
        total_all += v.total;
    }

    let overall = if total_all > 0 {
        total_done as f64 / total_all as f64 * 100.0
    } else {
        0.0
    };
    println!("  {}", "-".repeat(40));
    println!(
        "  总计:  {}/{} 完成 ({:.0}%)",
        total_done, total_all, overall
    );
}

// ═══════════════════════════════════════════════════════════════════════
// plan clean
// ═══════════════════════════════════════════════════════════════════════

/// 标记一行是否为完成的 checkbox。
fn is_done_item(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("- [x]") || t.starts_with("- [X]")
}

/// 标记一行是否为分类标题（`### Added / Changed / Fixed / Removed / Deprecated / Security`）。
fn is_category_header(line: &str) -> bool {
    let t = line.trim();
    CATEGORIES
        .iter()
        .any(|c| t == *c || t.eq_ignore_ascii_case(c))
}

const CATEGORIES: &[&str] = &[
    "### Added",
    "### Changed",
    "### Fixed",
    "### Removed",
    "### Deprecated",
    "### Security",
];

/// 标记一行是否为版本标题。
fn is_version_header(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("## [") && t.ends_with(']')
}

/// 删除 ROADMAP.md 中所有已完成条目。
///
/// 只删 `- [x]` 行，保留版本标题和分类标题。
/// 空版本和空分类自动清理。
pub fn clean_roadmap(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut lines: Vec<&str> = content.lines().collect();

    // 第一遍：删除 done item 行
    lines.retain(|l| !is_done_item(l));

    // 第二遍：删除空的分类标题（后面紧跟另一个标题或文件尾）
    let mut i = 0;
    while i + 1 < lines.len() {
        if is_category_header(lines[i]) {
            let next = lines[i + 1].trim();
            // 如果下一行是标题、文件尾或空行，删除此分类
            if next.is_empty() || is_category_header(next) || is_version_header(next) {
                lines.remove(i);
                continue;
            }
        }
        i += 1;
    }
    // 处理最后一个分类标题
    if let Some(last) = lines.last() {
        if is_category_header(last) {
            lines.pop();
        }
    }

    // 第三遍：删除空的版本标题（后面紧跟另一个版本标题或文件尾）
    let mut i = 0;
    while i + 1 < lines.len() {
        if is_version_header(lines[i]) {
            let next = lines[i + 1].trim();
            if next.is_empty() || is_version_header(next) {
                lines.remove(i);
                continue;
            }
        }
        i += 1;
    }
    if let Some(last) = lines.last() {
        if is_version_header(last) {
            lines.pop();
        }
    }

    // 清理尾部空行
    while let Some(last) = lines.last() {
        if last.trim().is_empty() {
            lines.pop();
        } else {
            break;
        }
    }

    let result: Vec<String> = lines.into_iter().map(|s| s.to_string()).collect();
    if !result.is_empty() {
        if let Ok(mut f) = std::fs::File::create(path) {
            use std::io::Write;
            for line in &result {
                writeln!(f, "{}", line).ok();
            }
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════
// plan doctor
// ═══════════════════════════════════════════════════════════════════════

/// 修复报告中的问题项。
#[derive(Debug)]
pub struct FixNote {
    pub line: usize,
    pub issue: String,
}

/// 验证 ROADMAP.md 格式，返回规则检测到的问题清单。
///
/// 规则只做验证，不做修复。修复由 `doctor_llm`（LLM 驱动）或人工完成。
/// 当前仅实现规则验证层，LLM 修复留作扩展点。
pub fn doctor_roadmap(path: &Path) -> Vec<FixNote> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut fixes: Vec<FixNote> = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();

        // 1. 版本标题禁止 v 前缀
        if trimmed.starts_with("## [") && trimmed.ends_with(']') {
            let ver = trimmed
                .trim_start_matches("## [")
                .trim_end_matches(']')
                .trim();
            if ver.starts_with('v') {
                fixes.push(FixNote {
                    line: line_num,
                    issue: format!("版本号不应有 v 前缀: {}", ver),
                });
            }
        }

        // 2. 分类标题必须使用标准大小写
        if trimmed.starts_with("### ") {
            let lowered = trimmed.to_lowercase();
            if let Some(standard) = CATEGORIES.iter().find(|c| c.to_lowercase() == lowered) {
                if trimmed != *standard {
                    fixes.push(FixNote {
                        line: line_num,
                        issue: format!("分类标题大小写: 应为 '{}'，当前 '{}'", standard, trimmed),
                    });
                }
            }
        }

        // 3. checkbox 必须使用标准格式 `- [x] ` 或 `- [ ] `
        let has_any_box =
            trimmed.contains("[x]") || trimmed.contains("[X]") || trimmed.contains("[ ]");
        let is_standard = trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
            || trimmed.starts_with("- [ ] ");
        if has_any_box && !is_standard {
            fixes.push(FixNote {
                line: line_num,
                issue: format!("checkbox 格式异常: {}", trimmed),
            });
        }
    }

    fixes
}

/// LLM 修复扩展点。接收规则验证的问题清单和原始内容，返回修复后的内容。
///
/// 当前为桩函数，仅返回 None 表示需人工介入。
/// 接入 LLM 后：将内容 + 问题清单发给 LLM → LLM 返回修正后的完整 ROADMAP → 写入文件。
#[allow(dead_code)]
pub fn doctor_llm(content: &str, issues: &[FixNote]) -> Option<String> {
    // TODO: 调 LLM API 修复格式
    // Prompt: "ROADMAP.md 格式问题清单：... 请修复格式，不增删条目内容"
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_roadmap(content: &str) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(d.path().join("ROADMAP.md")).unwrap();
        write!(f, "{}", content).unwrap();
        d
    }

    fn read_roadmap(d: &Path) -> String {
        std::fs::read_to_string(d.join("ROADMAP.md")).unwrap_or_default()
    }

    // ── parse ──────────────────────────────────────────────────────

    #[test]
    fn test_parse_empty() {
        let d = write_roadmap("");
        let v = parse_roadmap(&d.path().join("ROADMAP.md"));
        assert!(v.is_empty());
    }

    #[test]
    fn test_parse_single_version() {
        let d = write_roadmap(
            "## [0.1.0]\n\
             \n\
             ### Added\n\
             - [x] feature a\n\
             - [ ] feature b\n\
             ### Fixed\n\
             - [x] bug c\n",
        );
        let v = parse_roadmap(&d.path().join("ROADMAP.md"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].version, "0.1.0");
        assert_eq!(v[0].done, 2);
        assert_eq!(v[0].total, 3);
    }

    #[test]
    fn test_parse_multi_version() {
        let d = write_roadmap(
            "## [0.2.0]\n\
             - [x] done\n\
             - [ ] todo\n\
             \n\
             ## [0.1.0]\n\
             - [x] a\n\
             - [x] b\n",
        );
        let v = parse_roadmap(&d.path().join("ROADMAP.md"));
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].version, "0.2.0");
        assert_eq!(v[0].done, 1);
        assert_eq!(v[0].total, 2);
        assert_eq!(v[1].version, "0.1.0");
        assert_eq!(v[1].done, 2);
        assert_eq!(v[1].total, 2);
    }

    #[test]
    fn test_parse_v_prefix() {
        let d = write_roadmap("## [v0.1.0]\n- [x] item\n");
        let v = parse_roadmap(&d.path().join("ROADMAP.md"));
        assert_eq!(v[0].version, "0.1.0");
    }

    #[test]
    fn test_parse_no_checkboxes() {
        let d = write_roadmap("## [0.1.0]\n\njust text\nno boxes\n");
        let v = parse_roadmap(&d.path().join("ROADMAP.md"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].done, 0);
        assert_eq!(v[0].total, 0);
    }

    #[test]
    fn test_file_not_found() {
        let d = tempfile::tempdir().unwrap();
        let v = parse_roadmap(&d.path().join("NONEXISTENT.md"));
        assert!(v.is_empty());
    }

    // ── clean ──────────────────────────────────────────────────────

    #[test]
    fn test_clean_removes_done_items() {
        let d = write_roadmap(
            "## [0.1.0]\n\
             ### Added\n\
             - [x] done item\n\
             - [ ] todo item\n\
             ### Fixed\n\
             - [x] fixed bug\n",
        );
        clean_roadmap(&d.path().join("ROADMAP.md"));
        let content = read_roadmap(d.path());
        assert!(!content.contains("done item"));
        assert!(!content.contains("fixed bug"));
        assert!(content.contains("todo item"));
    }

    #[test]
    fn test_clean_removes_empty_category() {
        // 某个分类下所有条目都做完 → 分类标题也要删
        let d = write_roadmap(
            "## [0.1.0]\n\
             ### Added\n\
             - [x] done\n\
             ### Fixed\n\
             - [ ] remaining\n",
        );
        clean_roadmap(&d.path().join("ROADMAP.md"));
        let content = read_roadmap(d.path());
        assert!(!content.contains("### Added"));
        assert!(content.contains("### Fixed"));
    }

    #[test]
    fn test_clean_removes_empty_version() {
        // 一个版本下所有条目都做完 → 版本标题也要删
        let d = write_roadmap(
            "## [0.2.0]\n\
             ### Added\n\
             - [x] done\n\
             \n\
             ## [0.1.0]\n\
             - [ ] remaining\n",
        );
        clean_roadmap(&d.path().join("ROADMAP.md"));
        let content = read_roadmap(d.path());
        assert!(!content.contains("0.2.0")); // 版本也删了
        assert!(content.contains("0.1.0"));
    }

    #[test]
    fn test_clean_no_done_items_no_change() {
        let d = write_roadmap("## [0.1.0]\n- [ ] todo\n");
        clean_roadmap(&d.path().join("ROADMAP.md"));
        let content = read_roadmap(d.path());
        assert!(content.contains("todo"));
        assert!(content.contains("0.1.0"));
    }

    #[test]
    fn test_clean_file_not_found() {
        let d = tempfile::tempdir().unwrap();
        let result = clean_roadmap(&d.path().join("NONEXISTENT.md"));
        assert!(result.is_empty());
    }

    // ── doctor ─────────────────────────────────────────────────────

    #[test]
    fn test_doctor_fixes_v_prefix() {
        let d = write_roadmap("## [v0.1.0]\n- [ ] item\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.iter().any(|f| f.issue.contains("v 前缀")));
    }

    #[test]
    fn test_doctor_fixes_category_case() {
        let d = write_roadmap("## [0.1.0]\n### added\n- [ ] item\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.iter().any(|f| f.issue.contains("大小写")));
    }

    #[test]
    fn test_doctor_clean_file_no_fixes() {
        let d = write_roadmap("## [0.1.0]\n### Added\n- [ ] item\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_doctor_unknown_category_not_touched() {
        let d = write_roadmap("## [0.1.0]\n### Custom\n- [ ] item\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_doctor_file_not_found() {
        let d = tempfile::tempdir().unwrap();
        let fixes = doctor_roadmap(&d.path().join("NONEXISTENT.md"));
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_doctor_fixes_malformed_checkbox() {
        let d = write_roadmap("## [0.1.0]\n-[x]no space after dash\n-  [ ] double space\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.iter().any(|f| f.issue.contains("checkbox")));
    }

    #[test]
    fn test_doctor_does_not_modify_file() {
        let original = "## [v0.1.0]\n### added\n-  [x] bad format\n";
        let d = write_roadmap(original);
        let _fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        // doctor 是只读的，不修改文件内容
        assert_eq!(read_roadmap(d.path()), original);
    }

    #[test]
    fn test_doctor_standard_checkbox_not_touched() {
        let d = write_roadmap("## [0.1.0]\n- [x] normal\n- [ ] normal\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        let checkbox_fixes: Vec<_> = fixes
            .iter()
            .filter(|f| f.issue.contains("checkbox"))
            .collect();
        assert!(checkbox_fixes.is_empty());
    }

    #[test]
    fn test_doctor_fixes_multiple_issues() {
        let d = write_roadmap("## [v0.1.0]\n### fixed\n- [ ] bug\n### ADDED\n- [ ] feature\n");
        let fixes = doctor_roadmap(&d.path().join("ROADMAP.md"));
        assert!(fixes.len() >= 2);
    }
}
