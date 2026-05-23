use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitHash(pub String);

impl std::fmt::Display for CommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[..self.0.len().min(7)])
    }
}

impl Default for CommitHash {
    fn default() -> Self {
        Self(String::from("0000000000000000000000000000000000000000"))
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

impl SubmoduleStatus {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Dirty => 0,
            Self::Orphaned => 1,
            Self::Detached => 2,
            Self::Uninitialized => 3,
            Self::BehindRemote => 4,
            Self::AheadOfParent => 5,
            Self::Clean => 6,
        }
    }
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
}

#[derive(Debug, Clone)]
pub struct RepoState {
    pub root_path: PathBuf,
    pub submodules: Vec<Submodule>,
    pub total: usize,
    pub clean_count: usize,
    pub needs_attention: Vec<String>,
}

impl RepoState {
    pub fn scan(root: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let gitmodules_path = root.join(".gitmodules");
        if !gitmodules_path.exists() {
            return Ok(RepoState {
                root_path: root.to_path_buf(),
                submodules: vec![],
                total: 0,
                clean_count: 0,
                needs_attention: vec![],
            });
        }

        let repo = match git2::Repository::open(root) {
            Ok(r) => r,
            Err(e) => return Err(format!("无法打开 Git 仓库 '{}': {}", root.display(), e).into()),
        };
        let mut submodules = Vec::new();

        let mut git_submodules = repo.submodules()?;
        git_submodules.sort_by(|a, b| a.name().cmp(b.name()));

        for sm in &git_submodules {
            let name = sm.name().unwrap_or("unknown").to_string();
            let sm_path = sm.path();
            let full_sm_path = root.join(sm_path);
            let url = sm.url().unwrap_or("").to_string();
            let branch = sm.branch().unwrap_or("main").to_string();

            let raw_status = sm.status(false)?;
            let is_uninitialized =
                raw_status.contains(git2::SubmoduleStatus::WD_UNINITIALIZED);
            let is_dirty = raw_status.contains(git2::SubmoduleStatus::WD_DIRTY);

            // 父仓库记录的 commit
            let parent_pointer = CommitHash(sm.head_id().to_string());

            // 子模块本地 HEAD、远程 HEAD、commit 差异、Orphaned 检测（一次 open）
            let (local_head, remote_head, is_detached, ahead_count, behind_count, is_orphaned) = if is_uninitialized {
                (CommitHash::default(), CommitHash::default(), false, 0, 0, false)
            } else {
                match git2::Repository::open(&full_sm_path) {
                    Ok(sub_repo) => {
                        let local = sub_repo
                            .head()
                            .ok()
                            .and_then(|r| r.target())
                            .map(|o| CommitHash(o.to_string()))
                            .unwrap_or_default();

                        let detached = sub_repo
                            .head()
                            .ok()
                            .map(|r| !r.is_branch())
                            .unwrap_or(false);

                        let remote = sub_repo
                            .find_reference(&format!("refs/remotes/origin/{}", branch))
                            .ok()
                            .and_then(|r| r.target())
                            .map(|o| CommitHash(o.to_string()))
                            .unwrap_or_default();

                        let (ahead, behind) = count_commits(&sub_repo, &parent_pointer, &remote, &local);

                        // Orphaned: parent_pointer != remote_head and not reachable
                        let orphaned = if &remote != &CommitHash::default() && &parent_pointer != &remote {
                            let p = git2::Oid::from_str(&parent_pointer.0).ok();
                            let r = git2::Oid::from_str(&remote.0).ok();
                            match (p, r) {
                                (Some(p_oid), Some(r_oid)) => {
                                    sub_repo.merge_base(r_oid, p_oid).map(|base| base != p_oid).unwrap_or(false)
                                }
                                _ => false,
                            }
                        } else {
                            false
                        };

                        (local, remote, detached, ahead, behind, orphaned)
                    }
                    Err(_) => (CommitHash::default(), CommitHash::default(), false, 0, 0, false),
                }
            };

            let status = if is_uninitialized {
                SubmoduleStatus::Uninitialized
            } else if is_dirty {
                SubmoduleStatus::Dirty
            } else if is_detached {
                SubmoduleStatus::Detached
            } else if is_orphaned {
                SubmoduleStatus::Orphaned
            } else if ahead_count > 0 && behind_count == 0 {
                SubmoduleStatus::AheadOfParent
            } else if behind_count > 0 {
                SubmoduleStatus::BehindRemote
            } else if local_head == parent_pointer && local_head == remote_head {
                SubmoduleStatus::Clean
            } else {
                SubmoduleStatus::Clean
            };

            submodules.push(Submodule {
                name,
                path: sm_path.to_path_buf(),
                url,
                tracked_branch: branch,
                parent_pointer,
                local_head,
                remote_head,
                status,
                ahead_count,
                behind_count,
            });
        }

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
}

fn count_commits(
    sub_repo: &git2::Repository,
    parent_pointer: &CommitHash,
    remote_head: &CommitHash,
    local_head: &CommitHash,
) -> (usize, usize) {
    let parse = |h: &CommitHash| git2::Oid::from_str(&h.0).ok();

    let parent = parse(parent_pointer);
    let remote = parse(remote_head);
    let local = parse(local_head);

    let ahead = count_between(sub_repo, parent, local);
    let behind = count_between(sub_repo, local, remote);

    (ahead, behind)
}

fn count_between(
    repo: &git2::Repository,
    from: Option<git2::Oid>,
    to: Option<git2::Oid>,
) -> usize {
    let (Some(from), Some(to)) = (from, to) else {
        return 0;
    };
    if from == to {
        return 0;
    }
    let mut walk = match repo.revwalk() {
        Ok(w) => w,
        Err(_) => return 0,
    };
    if walk.push(to).is_err() || walk.hide(from).is_err() {
        return 0;
    }
    walk.count()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- SubmoduleStatus ----

    #[test]
    fn test_status_priority_ordering() {
        assert!(SubmoduleStatus::Dirty.priority() < SubmoduleStatus::Clean.priority());
        assert!(SubmoduleStatus::Orphaned.priority() < SubmoduleStatus::BehindRemote.priority());
        assert!(SubmoduleStatus::Detached.priority() < SubmoduleStatus::AheadOfParent.priority());
        assert!(SubmoduleStatus::Uninitialized.priority() < SubmoduleStatus::Clean.priority());
    }

    #[test]
    fn test_clean_is_lowest_priority() {
        let statuses = [
            SubmoduleStatus::Dirty,
            SubmoduleStatus::Orphaned,
            SubmoduleStatus::Detached,
            SubmoduleStatus::Uninitialized,
            SubmoduleStatus::BehindRemote,
            SubmoduleStatus::AheadOfParent,
        ];
        for s in &statuses {
            assert!(s.priority() < SubmoduleStatus::Clean.priority());
        }
    }

    #[test]
    fn test_all_priorities_are_unique() {
        let priorities: Vec<u8> = [
            SubmoduleStatus::Dirty,
            SubmoduleStatus::Orphaned,
            SubmoduleStatus::Detached,
            SubmoduleStatus::Uninitialized,
            SubmoduleStatus::BehindRemote,
            SubmoduleStatus::AheadOfParent,
            SubmoduleStatus::Clean,
        ]
        .iter()
        .map(|s| s.priority())
        .collect();
        let mut sorted = priorities.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(priorities.len(), sorted.len(), "priorities must be unique");
    }

    #[test]
    fn test_status_debug_output() {
        assert_eq!(format!("{:?}", SubmoduleStatus::Clean), "Clean");
        assert_eq!(format!("{:?}", SubmoduleStatus::Dirty), "Dirty");
        assert_eq!(format!("{:?}", SubmoduleStatus::Detached), "Detached");
        assert_eq!(format!("{:?}", SubmoduleStatus::Orphaned), "Orphaned");
        assert_eq!(format!("{:?}", SubmoduleStatus::Uninitialized), "Uninitialized");
        assert_eq!(format!("{:?}", SubmoduleStatus::AheadOfParent), "AheadOfParent");
        assert_eq!(format!("{:?}", SubmoduleStatus::BehindRemote), "BehindRemote");
    }

    #[test]
    fn test_status_clone_eq() {
        let a = SubmoduleStatus::Dirty;
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ---- CommitHash ----

    #[test]
    fn test_commit_hash_display_truncates() {
        let hash = CommitHash("abcdef1234567890".to_string());
        let display = format!("{}", hash);
        assert_eq!(display.len(), 7);
        assert_eq!(display, "abcdef1");
    }

    #[test]
    fn test_commit_hash_display_short() {
        let hash = CommitHash("abc".to_string());
        let display = format!("{}", hash);
        assert_eq!(display.len(), 3);
        assert_eq!(display, "abc");
    }

    #[test]
    fn test_commit_hash_display_empty() {
        let hash = CommitHash(String::new());
        let display = format!("{}", hash);
        assert_eq!(display, "");
    }

    #[test]
    fn test_commit_hash_equality() {
        let a = CommitHash("abc".to_string());
        let b = CommitHash("abc".to_string());
        let c = CommitHash("def".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_commit_hash_default() {
        let d = CommitHash::default();
        assert_eq!(d.0, "0000000000000000000000000000000000000000");
        assert_eq!(d.to_string(), "0000000");
    }

    #[test]
    fn test_commit_hash_clone() {
        let a = CommitHash("deadbeef".to_string());
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ---- Submodule ----

    #[test]
    fn test_submodule_builder() {
        let sm = Submodule {
            name: "test".into(),
            path: PathBuf::from("libs/test"),
            url: "https://example.com/test.git".into(),
            tracked_branch: "main".into(),
            parent_pointer: CommitHash("aaa".into()),
            local_head: CommitHash("bbb".into()),
            remote_head: CommitHash("ccc".into()),
            status: SubmoduleStatus::BehindRemote,
            ahead_count: 0,
            behind_count: 3,
        };
        assert_eq!(sm.name, "test");
        assert_eq!(sm.behind_count, 3);
        assert_eq!(format!("{:?}", sm.status), "BehindRemote");
    }

    // ---- RepoState ----

    #[test]
    fn test_repo_state_empty() {
        let state = RepoState {
            root_path: PathBuf::from("/tmp"),
            submodules: vec![],
            total: 0,
            clean_count: 0,
            needs_attention: vec![],
        };
        assert_eq!(state.total, 0);
        assert!(state.needs_attention.is_empty());
    }

    #[test]
    fn test_repo_state_with_mixed_status() {
        let submodules = vec![
            Submodule {
                name: "clean-one".into(),
                path: PathBuf::from("a"),
                url: String::new(),
                tracked_branch: "main".into(),
                parent_pointer: CommitHash::default(),
                local_head: CommitHash::default(),
                remote_head: CommitHash::default(),
                status: SubmoduleStatus::Clean,
                ahead_count: 0,
                behind_count: 0,
            },
            Submodule {
                name: "dirty-one".into(),
                path: PathBuf::from("b"),
                url: String::new(),
                tracked_branch: "main".into(),
                parent_pointer: CommitHash::default(),
                local_head: CommitHash::default(),
                remote_head: CommitHash::default(),
                status: SubmoduleStatus::Dirty,
                ahead_count: 0,
                behind_count: 0,
            },
        ];

        let total = submodules.len();
        let clean_count = submodules.iter().filter(|s| s.status == SubmoduleStatus::Clean).count();
        let needs_attention: Vec<String> = submodules
            .iter()
            .filter(|s| s.status != SubmoduleStatus::Clean)
            .map(|s| s.name.clone())
            .collect();

        assert_eq!(total, 2);
        assert_eq!(clean_count, 1);
        assert_eq!(needs_attention, vec!["dirty-one"]);
    }
}
