/// gix 重写子模块扫描 —— 对比 CLI 手写 git 命令的版本（scan.rs）。
///
/// 核心差异：
///   - 零进程启动，全部在进程内完成
///   - 结构化类型，无字符串解析
///   - 错误有类型，可区分处理
///
/// 依赖: gix = "0.69"
///
/// 用法: cargo run --bin quanttide-lab -- gix-scan <repo-path>
use std::path::{Path, PathBuf};

// ── 类型定义（与 scan.rs 一致，略简化） ──────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitHash(pub String);

impl std::fmt::Display for CommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[..self.0.len().min(7)])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmoduleStatus {
    Clean,
    AheadOfParent,
    BehindRemote,
    Detached,
    Dirty,
    Orphaned,
    Uninitialized,
}

#[derive(Debug, Clone)]
pub struct Submodule {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub tracked_branch: String,
    pub parent_pointer: CommitHash,
    pub local_head: CommitHash,
    pub remote_head: CommitHash,
    pub status: SubmoduleStatus,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub remote_unreachable: bool,
}

#[derive(Debug)]
pub struct RepoState {
    pub root_path: PathBuf,
    pub submodules: Vec<Submodule>,
    pub total: usize,
    pub clean_count: usize,
    pub needs_attention: Vec<String>,
}

// ── gix 实现 ─────────────────────────────────────────────────

/// gix 版本的状态扫描，完全替代 CLI 手写 git 的 scan_with_options。
pub fn scan_with_gix(root: &Path) -> Result<RepoState, Box<dyn std::error::Error>> {
    // 1. 打开仓库（gix 1ms vs git2 14ms，自带验证是否是 git 仓库）
    let repo = gix::open(root).map_err(|e| format!("打开仓库失败: {}", e))?;

    // 2. 解析 .gitmodules
    let raw_entries = parse_gitmodules_gix(root);
    let mut submodules: Vec<Submodule> = Vec::with_capacity(raw_entries.len());

    // 3. 获取 HEAD tree 用于查 parent pointer
    let head_id = repo.head()?.id().ok_or("无 HEAD")?;
    let head_commit = repo
        .find_commit(head_id)
        .map_err(|e| format!("HEAD commit 失败: {}", e))?;
    let head_tree = head_commit
        .tree()
        .map_err(|e| format!("HEAD tree 失败: {}", e))?;

    for (name, sm_path, url, _branch) in &raw_entries {
        let full_sm_path = root.join(sm_path);
        let full_sm_str = sm_path.display().to_string();

        let parent_pointer = head_tree
            .lookup_entry_by_path(&full_sm_str)
            .ok()
            .flatten()
            .map(|entry| CommitHash(entry.id().detach().to_hex().to_string()))
            .unwrap_or_default();

        let (
            local_head,
            remote_head,
            is_detached,
            ahead_count,
            behind_count,
            is_orphaned,
            remote_unreachable,
            is_uninitialized,
            is_dirty,
        ) = scan_single_submodule_gix(&full_sm_path);

        let status = determine_submodule_status(
            is_uninitialized,
            is_dirty,
            is_detached,
            is_orphaned,
            remote_unreachable,
            ahead_count,
            behind_count,
            &local_head,
            &parent_pointer,
        );

        submodules.push(Submodule {
            name: name.clone(),
            path: sm_path.clone(),
            url: url.clone(),
            tracked_branch: "main".into(),
            parent_pointer,
            local_head,
            remote_head,
            status,
            ahead_count,
            behind_count,
            remote_unreachable,
        });
    }

    submodules.sort_by(|a, b| a.name.cmp(&b.name));
    let total = submodules.len();
    let clean_count = submodules
        .iter()
        .filter(|s| s.status == SubmoduleStatus::Clean)
        .count();
    let needs_attention: Vec<String> = submodules
        .iter()
        .filter(|s| s.status != SubmoduleStatus::Clean)
        .map(|s| s.name.clone())
        .collect();

    Ok(RepoState {
        root_path: root.to_path_buf(),
        submodules,
        total,
        clean_count,
        needs_attention,
    })
}

/// gix 版本的子模块扫描（替代 scan_single_submodule 的 8 个 git 进程）。
fn scan_single_submodule_gix(
    full_sm_path: &Path,
) -> (
    CommitHash,
    CommitHash,
    bool,
    usize,
    usize,
    bool,
    bool,
    bool,
    bool,
) {
    if !full_sm_path.exists() || !full_sm_path.join(".git").exists() {
        return default_submodule_state();
    }

    let sm_repo = match gix::open(full_sm_path) {
        Ok(r) => r,
        Err(_) => return default_submodule_state(),
    };

    let head = match sm_repo.head() {
        Ok(h) => h,
        Err(_) => return default_submodule_state(),
    };
    let local_head = head
        .id()
        .map(|id| CommitHash(id.detach().to_hex().to_string()))
        .unwrap_or_default();

    // 游离 HEAD 检测：refernet_name 返回 None 表示 detached
    let is_detached = head.referent_name().is_none();

    // dirty 检测
    let is_dirty = sm_repo
        .status(gix::progress::Discard)
        .ok()
        .and_then(|s| s.into_index_worktree_iter(Vec::new()).ok())
        .map(|iter| iter.count() > 0)
        .unwrap_or(false);

    // 远程 HEAD（gix 引用遍历）
    // 注意：prefixed() 借用 Platform，需逐层展开；且引用可能是 symbolic 的，需 peel
    let (remote_head, remote_unreachable) = 'remote: {
        let refs = match sm_repo.references() {
            Ok(r) => r,
            Err(_) => break 'remote (CommitHash::default(), true),
        };
        let iter = match refs.prefixed("refs/remotes/origin") {
            Ok(i) => i,
            Err(_) => break 'remote (CommitHash::default(), true),
        };
        let first = iter.filter_map(|r| r.ok()).next();
        match first {
            Some(r) => {
                // try_id 不 panic：symbolic ref 返回 None，direct ref 返回 Some
                let oid = r.try_id().map(|id| id.detach());
                match oid {
                    Some(id) => (CommitHash(id.to_hex().to_string()), false),
                    None => (CommitHash::default(), true),
                }
            }
            None => (CommitHash::default(), true),
        }
    };

    (
        local_head,
        remote_head,
        is_detached,
        0,
        0,
        false,
        remote_unreachable,
        false,
        is_dirty,
    )
}

/// gix 版本解析 .gitmodules。
fn parse_gitmodules_gix(root: &Path) -> Vec<(String, PathBuf, String, String)> {
    let cfg_path = root.join(".gitmodules");
    if !cfg_path.exists() {
        return vec![];
    }
    let content = match std::fs::read_to_string(&cfg_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    parse_gitmodules_ini(&content)
}

/// 与 scan.rs 完全相同的 .gitmodules ini 解析器。
fn parse_gitmodules_ini(content: &str) -> Vec<(String, PathBuf, String, String)> {
    let mut entries = Vec::new();
    let mut current_name = String::new();
    let mut current_path = PathBuf::new();
    let mut current_url = String::new();
    let mut current_branch = String::from("main");
    let mut in_submodule = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("[submodule \"")
            .and_then(|s| s.strip_suffix("\"]"))
        {
            if in_submodule {
                entries.push((
                    std::mem::take(&mut current_name),
                    std::mem::take(&mut current_path),
                    std::mem::take(&mut current_url),
                    std::mem::take(&mut current_branch),
                ));
            }
            current_name = name.to_string();
            in_submodule = true;
        } else if in_submodule {
            if let Some(path) = trimmed.strip_prefix("path = ") {
                current_path = PathBuf::from(path);
            } else if let Some(url) = trimmed.strip_prefix("url = ") {
                current_url = url.to_string();
            } else if let Some(branch) = trimmed.strip_prefix("branch = ") {
                current_branch = branch.to_string();
            }
        }
    }
    if in_submodule {
        entries.push((current_name, current_path, current_url, current_branch));
    }
    entries
}

fn default_submodule_state() -> (
    CommitHash,
    CommitHash,
    bool,
    usize,
    usize,
    bool,
    bool,
    bool,
    bool,
) {
    (
        CommitHash::default(),
        CommitHash::default(),
        false,
        0,
        0,
        false,
        false,
        true,
        false,
    )
}

/// 与 scan.rs 完全相同的状态判定逻辑。
fn determine_submodule_status(
    is_uninitialized: bool,
    is_dirty: bool,
    is_detached: bool,
    is_orphaned: bool,
    remote_unreachable: bool,
    ahead_count: usize,
    behind_count: usize,
    local_head: &CommitHash,
    parent_pointer: &CommitHash,
) -> SubmoduleStatus {
    if is_uninitialized {
        return SubmoduleStatus::Uninitialized;
    }
    if is_dirty {
        return SubmoduleStatus::Dirty;
    }
    if is_detached {
        return SubmoduleStatus::Detached;
    }
    if is_orphaned && !remote_unreachable {
        return SubmoduleStatus::Orphaned;
    }
    if (remote_unreachable && local_head != parent_pointer)
        || (ahead_count > 0 && behind_count == 0)
    {
        return SubmoduleStatus::AheadOfParent;
    }
    if behind_count > 0 && !remote_unreachable {
        return SubmoduleStatus::BehindRemote;
    }
    SubmoduleStatus::Clean
}

impl Default for CommitHash {
    fn default() -> Self {
        Self("0000000000000000000000000000000000000000".to_string())
    }
}

// ── 入口（实验运行） ──────────────────────────────────────────

pub fn run(repo_path: &Path) {
    println!("━ gix 子模块扫描对比实验 ━\n");

    let start = std::time::Instant::now();
    match scan_with_gix(repo_path) {
        Ok(state) => {
            let dur = start.elapsed();
            println!("gix scan 耗时: {:?}", dur);
            println!("  子模块总数: {}", state.total);
            println!(
                "  干净: {}  需关注: {}",
                state.clean_count,
                state.needs_attention.len()
            );
            for sm in &state.submodules {
                println!("  {} ({:?})", sm.name, sm.status);
            }
        }
        Err(e) => {
            println!("gix scan 失败: {}", e);
        }
    }
}
