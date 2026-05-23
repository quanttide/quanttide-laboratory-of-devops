use std::path::{Path, PathBuf};

use crate::commands::history::{HistoryDb, OperationRecord};
use crate::commands::{HealthIssue, SubmoduleEditor, UpdateStrategy};
use crate::model::{RepoState, SubmoduleStatus};

pub struct GitSubmoduleEditor {
    root: PathBuf,
    history: HistoryDb,
}

impl GitSubmoduleEditor {
    pub fn new(root: PathBuf) -> Self {
        let history = HistoryDb::open(&root).unwrap_or_else(|e| {
            eprintln!("警告: 无法打开操作历史数据库: {}", e);
            // Create an in-memory fallback
            let db_dir = std::env::temp_dir().join("kse-history");
            std::fs::create_dir_all(&db_dir).ok();
            HistoryDb::open(&db_dir).expect("无法创建历史数据库")
        });
        Self { root, history }
    }

    fn log_ok(&self, action: &str, submodule: &str, detail: &str) {
        self.history.log_operation(action, submodule, detail, true).ok();
    }

    fn log_err(&self, action: &str, submodule: &str, detail: &str) {
        self.history.log_operation(action, submodule, detail, false).ok();
    }
}

impl SubmoduleEditor for GitSubmoduleEditor {
    fn root(&self) -> &Path {
        &self.root
    }

    fn add_submodule(
        &self,
        url: &str,
        path: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let full_path = self.root.join(path);
        if full_path.exists() {
            return Err(format!("路径已存在: {}", path).into());
        }
        let mut sm = repo.submodule(url, Path::new(path), false)?;
        sm.add_finalize()?;
        sm.set_branch(branch)?;
        let name = sm.name().unwrap_or(path);
        self.log_ok("add", name, &format!("url={}, path={}, branch={}", url, path, branch));
        println!("已添加子模块 '{}'", name);
        Ok(())
    }

    fn init_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let submodules = repo.submodules()?;
        let mut count = 0;
        for sm in &submodules {
            let status = sm.status(false)?;
            if status.contains(git2::SubmoduleStatus::WD_UNINITIALIZED) {
                sm.init(false)?;
                let name = sm.name().unwrap_or("unknown");
                self.log_ok("init", name, "初始化子模块");
                println!("已初始化子模块 '{}'", name);
                count += 1;
            }
        }
        if count == 0 {
            println!("没有未初始化的子模块");
        } else {
            println!("共初始化 {} 个子模块", count);
        }
        Ok(())
    }

    fn update_single(
        &self,
        name: &str,
        strategy: UpdateStrategy,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let mut sm = repo.find_submodule(name)?;
        let status = sm.status(false)?;
        if status.contains(git2::SubmoduleStatus::WD_DIRTY) {
            let msg = format!("子模块 '{}' 有未提交的修改，请先提交或 stash", name);
            self.log_err("update", name, &msg);
            return Err(msg.into());
        }
        sm.update(false, strategy.to_git2_update())?;
        self.log_ok("update", name, &format!("strategy={:?}", strategy));
        println!("已更新子模块 '{}'", name);
        Ok(())
    }

    fn update_all(&self, strategy: UpdateStrategy) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let submodules = repo.submodules()?;
        let mut count = 0;
        for sm in &submodules {
            let name = sm.name().unwrap_or("unknown").to_string();
            match self.update_single(&name, strategy) {
                Ok(()) => count += 1,
                Err(e) => eprintln!("警告: 更新子模块 '{}' 失败: {}", name, e),
            }
        }
        println!("共更新 {} 个子模块", count);
        Ok(())
    }

    fn sync_to_parent(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let sm = repo.find_submodule(name)?;
        let sm_path = sm.path();

        let mut index = repo.index()?;
        index.add_path(sm_path)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?;
        let parent = head.peel_to_commit()?;
        let signature = git2::Signature::now("kse", "kse@local")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("chore: 更新子模块 '{}' 指针", name),
            &tree,
            &[&parent],
        )?;
        self.log_ok("sync", name, "同步到父仓库");
        println!("已同步子模块 '{}' 到父仓库", name);
        Ok(())
    }

    fn sync_all_to_parent(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let submodules = repo.submodules()?;
        for sm in &submodules {
            let name = sm.name().unwrap_or("unknown").to_string();
            match self.sync_to_parent(&name) {
                Ok(()) => {}
                Err(e) => eprintln!("警告: 同步子模块 '{}' 失败: {}", name, e),
            }
        }
        Ok(())
    }

    fn checkout_branch(&self, name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let sm = repo.find_submodule(name)?;
        let sm_repo_path = self.root.join(sm.path());
        let sm_repo = git2::Repository::open(&sm_repo_path)?;

        let ref_name = format!("refs/heads/{}", branch);
        let obj = sm_repo.revparse_single(&ref_name)?;
        sm_repo.checkout_tree(&obj, None)?;
        sm_repo.set_head(&ref_name)?;
        self.log_ok("checkout", name, &format!("切换到分支 {}", branch));
        println!("子模块 '{}' 已切换到分支 '{}'", name, branch);
        Ok(())
    }

    fn create_branch(&self, name: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let sm = repo.find_submodule(name)?;
        let sm_repo_path = self.root.join(sm.path());
        let sm_repo = git2::Repository::open(&sm_repo_path)?;

        let head = sm_repo.head()?;
        let commit = head.peel_to_commit()?;
        sm_repo.branch(branch, &commit, false)?;

        let ref_name = format!("refs/heads/{}", branch);
        sm_repo.set_head(&ref_name)?;
        self.log_ok("branch", name, &format!("创建并切换到分支 {}", branch));
        println!("子模块 '{}' 已创建并切换到分支 '{}'", name, branch);
        Ok(())
    }

    fn retire_submodule(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(&self.root)?;
        let sm = repo.find_submodule(name)?;
        let url = sm.url().unwrap_or("").to_string();
        let sm_path = sm.path().to_path_buf();
        let mut sm = repo.find_submodule(name)?;
        sm.deinit(true)?;

        let gitmodules_path = self.root.join(".gitmodules");
        if gitmodules_path.exists() {
            let content = std::fs::read_to_string(&gitmodules_path)?;
            let mut new_content = String::new();
            let mut skip = false;
            let in_submodule = format!(r#"[submodule "{}"]"#, name);
            let in_submodule_alt = format!("[submodule \"{}\"]", name);
            for line in content.lines() {
                if line.trim() == in_submodule_alt || line.starts_with(&in_submodule) {
                    skip = true;
                    continue;
                }
                if skip && line.trim_start().starts_with('[') {
                    skip = false;
                }
                if !skip {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            std::fs::write(&gitmodules_path, new_content)?;
        }
        let mut index = repo.index()?;
        index.remove_path(&sm_path)?;
        index.write()?;

        self.history.log_retire(name, &url, &sm_path.display().to_string(), "用户手动退役")?;
        println!("已退役子模块 '{}'", name);
        Ok(())
    }

    pub fn list_history(
        &self,
        limit: usize,
        submodule: Option<&str>,
    ) -> Result<Vec<OperationRecord>, Box<dyn std::error::Error>> {
        self.history.list_operations(limit, submodule)
    }

    fn health_check(&self) -> Result<Vec<HealthIssue>, Box<dyn std::error::Error>> {
        let state = RepoState::scan(&self.root)?;
        let mut issues = Vec::new();
        for sm in &state.submodules {
            if sm.status != SubmoduleStatus::Clean {
                let (description, action) = describe_issue(&sm.status);
                issues.push(HealthIssue {
                    submodule_name: sm.name.clone(),
                    status: sm.status.clone(),
                    description,
                    suggested_action: action,
                });
            }
        }
        Ok(issues)
    }
}

fn describe_issue(status: &SubmoduleStatus) -> (String, String) {
    match status {
        SubmoduleStatus::AheadOfParent => (
            "本地领先于父仓库记录".into(),
            "运行 sync_to_parent 更新父仓库指针".into(),
        ),
        SubmoduleStatus::BehindRemote => (
            "远程有更新，本地落后".into(),
            "运行 update 获取最新代码".into(),
        ),
        SubmoduleStatus::Detached => (
            "处于游离 HEAD 状态".into(),
            "运行 checkout_branch 切换到跟踪分支".into(),
        ),
        SubmoduleStatus::Dirty => (
            "有未提交的修改".into(),
            "提交或 stash 当前修改".into(),
        ),
        SubmoduleStatus::Orphaned => (
            "父仓库记录的 commit 在远程已不存在".into(),
            "需手动干预".into(),
        ),
        SubmoduleStatus::Uninitialized => (
            "尚未初始化".into(),
            "运行 init 初始化子模块".into(),
        ),
        SubmoduleStatus::Clean => unreachable!(),
    }
}
