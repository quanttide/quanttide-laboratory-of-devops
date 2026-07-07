/// git2 vs gix API 与性能对比实验
///
/// 对常见的 DevOps git 操作分别用两个库实现并计时。
/// 用法: cargo run -- git-exp [repo-path]
use std::path::Path;
use std::time::{Duration, Instant};

// ── 实验结果类型 ──────────────────────────────────────────────

struct Trial {
    label: &'static str,
    git2_ok: bool,
    gix_ok: bool,
    git2_dur: Duration,
    gix_dur: Duration,
}

impl Trial {
    fn winner(&self) -> &'static str {
        if !self.git2_ok && !self.gix_ok {
            "both failed"
        } else if !self.git2_ok {
            "gix only"
        } else if !self.gix_ok {
            "git2 only"
        } else {
            let a = self.git2_dur.as_nanos();
            let b = self.gix_dur.as_nanos();
            if a < b {
                "git2"
            } else if b < a {
                "gix"
            } else {
                "tie"
            }
        }
    }

    fn ratio(&self) -> f64 {
        let a = self.git2_dur.as_nanos().max(1);
        let b = self.gix_dur.as_nanos().max(1);
        if a > b {
            a as f64 / b as f64
        } else {
            b as f64 / a as f64
        }
    }
}

// ── 辅助: 计时包装 ────────────────────────────────────────────

fn time<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let r = f();
    let dur = start.elapsed();
    (r, dur)
}

// ── 各个实验 ──────────────────────────────────────────────────

/// 1. 打开仓库
fn exp_open(path: &Path) -> Trial {
    let (git2_ok, git2_dur) = time(|| git2::Repository::open(path).is_ok());
    let (gix_ok, gix_dur) = time(|| gix::open(path).is_ok());
    Trial {
        label: "open",
        git2_ok,
        gix_ok,
        git2_dur,
        gix_dur,
    }
}

/// 2. 解析 HEAD（当前 commit OID）
fn exp_head(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        let _oid = head.target()?;
        Some(())
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let head = repo.head().ok()?;
        let _oid = head.id()?;
        Some(())
    });
    Trial {
        label: "head",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 3. 遍历提交历史（取前 100 条）
fn exp_log(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let mut revwalk = repo.revwalk().ok()?;
        revwalk.push_head().ok()?;
        let count = revwalk.take(100).count();
        Some(count)
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let head = repo.head().ok()?;
        let oid = head.id()?;
        let walk = repo.rev_walk(vec![oid]).all().ok()?;
        let count = walk.take(100).count();
        Some(count)
    });
    Trial {
        label: "log(100)",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 4. 读取 Cargo.toml 在 HEAD 的内容
fn exp_blob(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        let tree = head.peel_to_tree().ok()?;
        let entry = tree.get_path(std::path::Path::new("Cargo.toml")).ok()?;
        let blob = repo.find_blob(entry.id()).ok()?;
        let _content = blob.content();
        Some(())
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let head = repo.head().ok()?;
        let oid = head.id()?;
        let commit = repo.find_commit(oid).ok()?;
        let tree = commit.tree().ok()?;
        let entry = tree.lookup_entry_by_path("Cargo.toml").ok()??;
        let obj = entry.object().ok()?;
        let _data = &obj.data;
        Some(())
    });
    Trial {
        label: "blob(Cargo.toml)",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 5. HEAD~3 与 HEAD 之间的 diff
fn exp_diff(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let mut revwalk = repo.revwalk().ok()?;
        revwalk.push_head().ok()?;
        let _ = revwalk.nth(3); // HEAD~3
        let old_oid = revwalk.next()?.ok()?;
        let head = repo.head().ok()?;
        let head_oid = head.target()?;
        let old_tree = repo.find_commit(old_oid).ok()?.tree().ok()?;
        let head_tree = repo.find_commit(head_oid).ok()?.tree().ok()?;
        let diff = repo
            .diff_tree_to_tree(Some(&old_tree), Some(&head_tree), None)
            .ok()?;
        let _stats = diff.stats().ok()?;
        Some(())
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let head = repo.head().ok()?;
        let head_oid = head.id()?;
        let head_commit = repo.find_commit(head_oid).ok()?;
        let head_tree = head_commit.tree().ok()?;

        let walk = repo.rev_walk(vec![head_oid]).all().ok()?;
        let old_info = walk.skip(3).next()?.ok()?;
        let old_tree = repo.find_commit(old_info.id()).ok()?.tree().ok()?;

        let _ = repo
            .diff_tree_to_tree(Some(&old_tree), Some(&head_tree), None)
            .ok()?;
        Some(())
    });
    Trial {
        label: "diff(HEAD~3..HEAD)",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 6. 工作区状态
fn exp_status(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).ok()?;
        let _count = statuses.len();
        Some(())
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let status = repo.status(gix::progress::Discard).ok()?;
        let iter = status.into_index_worktree_iter(Vec::new()).ok()?;
        let count = iter.count();
        Some(count)
    });
    Trial {
        label: "status",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 7. 列出本地分支
fn exp_branches(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let branches = repo.branches(Some(git2::BranchType::Local)).ok()?;
        let count = branches.count();
        Some(count)
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let refs = repo.references().ok()?;
        let iter = refs.prefixed("refs/heads").ok()?;
        let count = iter.count();
        Some(count)
    });
    Trial {
        label: "branches",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

/// 8. 列出 tag
fn exp_tags(path: &Path) -> Trial {
    let git2_ok = time(|| {
        let repo = git2::Repository::open(path).ok()?;
        let mut tags = Vec::new();
        repo.tag_foreach(|_oid, name| {
            tags.push(String::from_utf8_lossy(name).to_string());
            true
        })
        .ok()?;
        Some(tags.len())
    });
    let gix_ok = time(|| {
        let repo = gix::open(path).ok()?;
        let refs = repo.references().ok()?;
        let iter = refs.prefixed("refs/tags").ok()?;
        let count = iter.count();
        Some(count)
    });
    Trial {
        label: "tags",
        git2_ok: git2_ok.0.is_some(),
        gix_ok: gix_ok.0.is_some(),
        git2_dur: git2_ok.1,
        gix_dur: gix_ok.1,
    }
}

// ── 入口 ──────────────────────────────────────────────────────

pub fn run(repo_path: &Path) {
    let trials = [
        exp_open(repo_path),
        exp_head(repo_path),
        exp_log(repo_path),
        exp_blob(repo_path),
        exp_diff(repo_path),
        exp_status(repo_path),
        exp_branches(repo_path),
        exp_tags(repo_path),
    ];

    println!("━━━ git2 vs gix 性能对比 ━━━\n");
    println!(
        "  {:<24} {:>8} {:>8}  {:>8}  {:>6}  {:>5}",
        "操作", "git2", "gix", "胜者", "加速比", "结果"
    );
    println!("  ─────────────────────────────────────────────────────────────");

    let both_ok = |t: &Trial| t.git2_ok && t.gix_ok;

    for t in &trials {
        let g2 = if t.git2_ok {
            format!("{:>7}µs", t.git2_dur.as_micros())
        } else {
            "   FAIL".into()
        };
        let gx = if t.gix_ok {
            format!("{:>7}µs", t.gix_dur.as_micros())
        } else {
            "   FAIL".into()
        };
        let winner = t.winner();
        let ratio = if both_ok(t) {
            format!("{:>4.1}x", t.ratio())
        } else {
            "    -".into()
        };
        let result = match (t.git2_ok, t.gix_ok) {
            (true, true) => "✅",
            (false, false) => "❌",
            (true, false) => "⚠ git2",
            (false, true) => "⚠ gix",
        };
        println!(
            "  {:<24} {:>8} {:>8}  {:>8}  {:>6}  {:>5}",
            t.label, g2, gx, winner, ratio, result
        );
    }

    // 汇总
    let git2_wins = trials
        .iter()
        .filter(|t| t.winner() == "git2" && both_ok(t))
        .count();
    let gix_wins = trials
        .iter()
        .filter(|t| t.winner() == "gix" && both_ok(t))
        .count();
    let both_fail = trials.iter().filter(|t| !t.git2_ok && !t.gix_ok).count();
    println!(
        "\n  git2 胜: {}  |  gix 胜: {}  |  均失败: {}",
        git2_wins, gix_wins, both_fail
    );
    println!("  路径: {}", repo_path.display());
}
