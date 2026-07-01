/// 子模块状态三分法模型。
///
/// 对应 roadmap 中的状态重构方案：
/// - Synchronized：理想状态
/// - OutOfSync：可自动修复
/// - Anomaly：需要人工介入
use std::path::Path;

// ── 状态枚举 ──────────────────────────────────────────────────────────

/// 子模块顶层状态（三分法）。
#[derive(Debug, Clone, PartialEq)]
pub enum SubmoduleStatus {
    /// 已同步，理想状态
    Synchronized,
    /// 可自动修复
    OutOfSync(OutOfSyncKind),
    /// 需人工介入
    Anomaly(AnomalyKind),
}

/// 可自动修复的同步偏差。
#[derive(Debug, Clone, PartialEq)]
pub enum OutOfSyncKind {
    /// 有本地提交未推送
    AheadOfRemote,
    /// 远程有新提交未拉取
    BehindRemote,
    /// 工作区或暂存区有未提交变更
    Dirty,
    /// 本地和远程历史分叉
    Diverged,
}

/// 需人工介入的结构异常。
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyKind {
    /// HEAD 游离
    DetachedHead,
    /// 父仓库记录的 commit 在远程不存在
    Orphaned,
    /// 子模块目录不存在
    Missing,
    /// 子模块存在但未在 .gitmodules 注册
    Unregistered,
    /// 未知错误
    Unknown(String),
}

// ── 严重程度 ──────────────────────────────────────────────────────────

/// 问题严重程度。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "ℹ"),
            Severity::Warning => write!(f, "⚠"),
            Severity::Error => write!(f, "✗"),
        }
    }
}

// ── 健康诊断 ──────────────────────────────────────────────────────────

/// 健康问题诊断报告。
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub severity: Severity,
    pub submodule: String,
    pub description: String,
    pub suggested_action: String,
    pub auto_fixable: bool,
}

/// 根据状态生成诊断报告。
pub fn diagnose(submodule: &str, status: &SubmoduleStatus) -> HealthIssue {
    match status {
        SubmoduleStatus::Synchronized => HealthIssue {
            severity: Severity::Info,
            submodule: submodule.into(),
            description: "已同步".into(),
            suggested_action: "无需操作".into(),
            auto_fixable: true,
        },
        SubmoduleStatus::OutOfSync(kind) => match kind {
            OutOfSyncKind::AheadOfRemote => HealthIssue {
                severity: Severity::Info,
                submodule: submodule.into(),
                description: "有本地提交未推送".into(),
                suggested_action: "运行 code sync 自动推送".into(),
                auto_fixable: true,
            },
            OutOfSyncKind::BehindRemote => HealthIssue {
                severity: Severity::Info,
                submodule: submodule.into(),
                description: "远程有新提交未拉取".into(),
                suggested_action: "运行 code sync 自动拉取".into(),
                auto_fixable: true,
            },
            OutOfSyncKind::Dirty => HealthIssue {
                severity: Severity::Warning,
                submodule: submodule.into(),
                description: "有未提交的修改".into(),
                suggested_action: "请先提交或 stash，然后运行 sync".into(),
                auto_fixable: false,
            },
            OutOfSyncKind::Diverged => HealthIssue {
                severity: Severity::Error,
                submodule: submodule.into(),
                description: "本地与远程历史分叉".into(),
                suggested_action: "手动 merge 或 rebase".into(),
                auto_fixable: false,
            },
        },
        SubmoduleStatus::Anomaly(kind) => match kind {
            AnomalyKind::DetachedHead => HealthIssue {
                severity: Severity::Error,
                submodule: submodule.into(),
                description: "HEAD 游离".into(),
                suggested_action: "运行 git switch main".into(),
                auto_fixable: false,
            },
            AnomalyKind::Orphaned => HealthIssue {
                severity: Severity::Error,
                submodule: submodule.into(),
                description: "父仓库记录的 commit 在远程已不存在".into(),
                suggested_action: "手动检查子模块状态".into(),
                auto_fixable: false,
            },
            AnomalyKind::Missing => HealthIssue {
                severity: Severity::Error,
                submodule: submodule.into(),
                description: "子模块目录不存在".into(),
                suggested_action: "运行 git submodule update --init".into(),
                auto_fixable: true,
            },
            AnomalyKind::Unregistered => HealthIssue {
                severity: Severity::Warning,
                submodule: submodule.into(),
                description: "子模块存在但未在 .gitmodules 注册".into(),
                suggested_action: "手动清理或重新注册".into(),
                auto_fixable: false,
            },
            AnomalyKind::Unknown(msg) => HealthIssue {
                severity: Severity::Error,
                submodule: submodule.into(),
                description: format!("未知异常: {}", msg),
                suggested_action: "请手动检查仓库状态".into(),
                auto_fixable: false,
            },
        },
    }
}

/// 简化版的子模块状态检测：扫描工作树根部，检测常见状态。
///
/// 真实实现需要 git2 crate 和子模块遍历。此处仅演示状态模型用法。
pub fn scan_submodules(repo_path: &Path) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

    // 检查 .gitmodules 是否存在
    let gitmodules = repo_path.join(".gitmodules");
    if !gitmodules.exists() {
        return issues;
    }

    // 解析子模块列表（简版：读 .gitmodules）
    let content = std::fs::read_to_string(&gitmodules).unwrap_or_default();
    let mut current_name = String::new();
    let mut current_path = String::new();
    let mut in_submodule = false;

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("[submodule") {
            if in_submodule && !current_name.is_empty() {
                let status = check_submodule(repo_path, &current_name, &current_path);
                issues.push(diagnose(&current_name, &status));
            }
            current_name = t
                .trim_start_matches("[submodule \"")
                .trim_end_matches("\"]")
                .to_string();
            current_path.clear();
            in_submodule = true;
        } else if in_submodule {
            if let Some(p) = t.strip_prefix("path = ") {
                current_path = p.to_string();
            }
        }
    }
    if in_submodule && !current_name.is_empty() {
        let status = check_submodule(repo_path, &current_name, &current_path);
        issues.push(diagnose(&current_name, &status));
    }

    issues.sort_by_key(|i| i.severity.clone());
    issues
}

fn check_submodule(repo_path: &Path, _name: &str, path: &str) -> SubmoduleStatus {
    let sm_dir = repo_path.join(path);
    if !sm_dir.exists() {
        return SubmoduleStatus::Anomaly(AnomalyKind::Missing);
    }

    // 检查是否为 git 仓库
    let git_dir = sm_dir.join(".git");
    if !git_dir.exists() {
        return SubmoduleStatus::Anomaly(AnomalyKind::Unknown("不是 git 仓库".into()));
    }

    // 检查工作区是否干净
    let status_out = std::process::Command::new("git")
        .args(["-C", &sm_dir.to_string_lossy(), "status", "--porcelain"])
        .output();
    if let Ok(o) = status_out {
        if !o.stdout.is_empty() {
            return SubmoduleStatus::OutOfSync(OutOfSyncKind::Dirty);
        }
    }

    // 检查 ahead / behind
    let branch_out = std::process::Command::new("git")
        .args([
            "-C",
            &sm_dir.to_string_lossy(),
            "rev-list",
            "--left-right",
            "--count",
            "HEAD...origin/main",
        ])
        .output();
    if let Ok(o) = branch_out {
        let out = String::from_utf8_lossy(&o.stdout);
        if let Some((ahead, behind)) = out.trim().split_once('\t') {
            let a: usize = ahead.trim().parse().unwrap_or(0);
            let b: usize = behind.trim().parse().unwrap_or(0);
            if a > 0 && b > 0 {
                return SubmoduleStatus::OutOfSync(OutOfSyncKind::Diverged);
            }
            if a > 0 {
                return SubmoduleStatus::OutOfSync(OutOfSyncKind::AheadOfRemote);
            }
            if b > 0 {
                return SubmoduleStatus::OutOfSync(OutOfSyncKind::BehindRemote);
            }
        }
    }

    SubmoduleStatus::Synchronized
}

#[cfg(test)]
mod tests {
    use super::*;

    fn health_issue(name: &str, status: &SubmoduleStatus) -> HealthIssue {
        diagnose(name, status)
    }

    #[test]
    fn test_synchronized_is_info() {
        let h = health_issue("sm", &SubmoduleStatus::Synchronized);
        assert_eq!(h.severity, Severity::Info);
        assert!(h.auto_fixable);
    }

    #[test]
    fn test_ahead_is_info_and_fixable() {
        let h = health_issue(
            "sm",
            &SubmoduleStatus::OutOfSync(OutOfSyncKind::AheadOfRemote),
        );
        assert_eq!(h.severity, Severity::Info);
        assert!(h.auto_fixable);
    }

    #[test]
    fn test_behind_is_info_and_fixable() {
        let h = health_issue(
            "sm",
            &SubmoduleStatus::OutOfSync(OutOfSyncKind::BehindRemote),
        );
        assert_eq!(h.severity, Severity::Info);
        assert!(h.auto_fixable);
    }

    #[test]
    fn test_dirty_is_warning_not_fixable() {
        let h = health_issue("sm", &SubmoduleStatus::OutOfSync(OutOfSyncKind::Dirty));
        assert_eq!(h.severity, Severity::Warning);
        assert!(!h.auto_fixable);
    }

    #[test]
    fn test_diverged_is_error_not_fixable() {
        let h = health_issue("sm", &SubmoduleStatus::OutOfSync(OutOfSyncKind::Diverged));
        assert_eq!(h.severity, Severity::Error);
        assert!(!h.auto_fixable);
    }

    #[test]
    fn test_detached_is_error() {
        let h = health_issue("sm", &SubmoduleStatus::Anomaly(AnomalyKind::DetachedHead));
        assert_eq!(h.severity, Severity::Error);
    }

    #[test]
    fn test_missing_is_error_but_fixable() {
        let h = health_issue("sm", &SubmoduleStatus::Anomaly(AnomalyKind::Missing));
        assert_eq!(h.severity, Severity::Error);
        assert!(h.auto_fixable);
    }

    #[test]
    fn test_orphaned_is_error() {
        let h = health_issue("sm", &SubmoduleStatus::Anomaly(AnomalyKind::Orphaned));
        assert_eq!(h.severity, Severity::Error);
    }

    #[test]
    fn test_unknown_includes_message() {
        let h = health_issue(
            "sm",
            &SubmoduleStatus::Anomaly(AnomalyKind::Unknown("network error".into())),
        );
        assert!(h.description.contains("network error"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_scan_no_gitmodules() {
        let d = tempfile::tempdir().unwrap();
        let issues = scan_submodules(d.path());
        assert!(issues.is_empty());
    }
}
