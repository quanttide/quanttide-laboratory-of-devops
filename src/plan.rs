/// plan 命令实验原型：解析 ROADMAP.md 并显示 scope 规划进度。
///
/// 对应 `data/roadmap/platform/plan-command.md` 中的 `plan status` 设计。
use std::path::Path;

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

        // `## [X.Y.Z]` — 版本标题
        if trimmed.starts_with("## [") && trimmed.ends_with(']') {
            // 保存上一个版本的数据
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

        // `- [x]` — 已完成
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            total += 1;
            done += 1;
            continue;
        }

        // `- [ ]` — 未完成
        if trimmed.starts_with("- [ ]") {
            total += 1;
            continue;
        }
    }

    // 最后一个版本
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
        assert_eq!(v[0].version, "0.1.0"); // 'v' 被去掉
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
}
