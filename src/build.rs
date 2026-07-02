use std::path::Path;

use crate::contract;

/// 输出当前仓库的构建状态（按 scope）。
pub fn status(repo_path: &Path) {
    let c = contract::load(repo_path);

    println!("构建状态");
    println!("{}", "-".repeat(50));

    if c.scopes.is_empty() {
        let lang = contract::detect_by_files(repo_path);
        let root_scope = contract::Scope {
            name: "(root)".into(),
            dir: ".".into(),
            language: lang.clone(),
            framework: String::new(),
            build_tool: contract::BuildTool::Unknown(String::new()),
            registry: contract::Registry::None,
            release: contract::StageRelease::default(),
            test_threshold: None,
        };
        let vs = contract::version_status(repo_path, &root_scope);
        let release = contract::scope_release(&c, &root_scope);
        print_scope("(root)", repo_path, &lang, &vs, release, &c);
    } else {
        for scope in &c.scopes {
            let scope_dir = repo_path.join(&scope.dir);
            if !scope_dir.exists() {
                println!("  [{}]     ⚠ 目录不存在: {}", scope.name, scope.dir);
                continue;
            }
            let lang = contract::resolve_language(scope, &scope_dir);
            let vs = contract::version_status(repo_path, scope);
            let release = contract::scope_release(&c, scope);
            print_scope(&scope.name, &scope_dir, &lang, &vs, release, &c);
        }
    }

    let dirty = is_working_tree_dirty(repo_path);
    println!(
        "  {}         {}",
        "工作区".to_string(),
        if dirty {
            "⚠ 有未提交变更"
        } else {
            "✅ 干净"
        }
    );
}

fn print_scope(
    name: &str,
    _dir: &Path,
    lang: &contract::Language,
    vs: &contract::VersionStatus,
    release: &contract::StageRelease,
    c: &contract::Contract,
) {
    println!("  [{:<12}] {}", name, lang.name());
    println!("    CI:         {}", check_ci(name));
    println!("    syntax:     {}", check_syntax(name));
    match (&vs.tag_version, &vs.config_version) {
        (Some(t), Some(cv)) if t == cv => println!("    version:    ✅ {}（一致）", t),
        (Some(t), Some(cv)) => println!("    version:    ⚠ tag {} ≠ 配置 {}", t, cv),
        (Some(t), None) => println!("    version:    tag {}（无配置文件）", t),
        (None, Some(cv)) => println!("    version:    配置 {}（无 tag）", cv),
        (None, None) => println!("    version:    暂无发布"),
    }
    println!("    registry:   {}", c.platforms.artifact_registry.name());
    println!("    changelog:  {}", release.changelog);
}

fn check_ci(_scope: &str) -> String {
    match std::process::Command::new("gh").arg("--version").output() {
        Ok(o) if o.status.success() => "gh 可用（需配置）".to_string(),
        _ => "⚠ gh CLI 未安装".to_string(),
    }
}

fn check_syntax(scope: &str) -> String {
    let dir = Path::new(".").join(scope);
    if dir.join("Cargo.toml").exists() {
        match std::process::Command::new("cargo")
            .args([
                "check",
                "--manifest-path",
                &dir.join("Cargo.toml").to_string_lossy(),
            ])
            .output()
        {
            Ok(o) if o.status.success() => "✅ cargo check 通过".to_string(),
            Ok(_) => "❌ cargo check 失败".to_string(),
            Err(_) => "⚠ cargo 未安装".to_string(),
        }
    } else {
        "—".to_string()
    }
}

fn is_working_tree_dirty(repo_path: &Path) -> bool {
    match std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
    {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_scope_all_ok() {
        let d = tempfile::tempdir().unwrap();
        let c = contract::load(d.path());
        let vs = contract::VersionStatus {
            tag_version: Some("0.1.0".into()),
            config_version: Some("0.1.0".into()),
            consistent: true,
        };
        let release = contract::StageRelease::default();
        // 只是格式验证，不写文件
        print_scope(
            "test",
            d.path(),
            &contract::Language::Rust,
            &vs,
            &release,
            &c,
        );
    }

    #[test]
    fn test_is_working_tree_dirty_empty_repo() {
        let d = tempfile::tempdir().unwrap();
        // 不是 git 仓库时返回 false
        assert!(!is_working_tree_dirty(d.path()));
    }

    #[test]
    fn test_detect_no_contract_yaml() {
        let d = tempfile::tempdir().unwrap();
        // 默认契约的 scopes 为空
        let c = contract::load(d.path());
        assert!(c.scopes.is_empty());
    }
}
