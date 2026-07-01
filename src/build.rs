use std::path::Path;

use crate::contract;

/// 输出当前仓库的构建状态（按 scope）。
pub fn status(repo_path: &Path) {
    let scopes = contract::load_scopes(repo_path);

    println!("构建状态");
    println!("{}", "-".repeat(50));

    if scopes.is_empty() {
        // 无 contract.yaml，检测根目录
        let lang = contract::detect_language(repo_path);
        let tag = latest_tag(repo_path);
        let config_ver = contract::version_status(
            repo_path,
            &contract::Scope {
                name: "(root)".into(),
                dir: ".".into(),
            },
        );
        print_scope("(root)", repo_path, &lang, &tag, &config_ver);
    } else {
        for scope in &scopes {
            let scope_dir = repo_path.join(&scope.dir);
            if !scope_dir.exists() {
                println!("  [{}]     ⚠ 目录不存在: {}", scope.name, scope.dir);
                continue;
            }
            let lang = contract::detect_language(&scope_dir);
            let tag = latest_tag_for_scope(repo_path, &scope.name);
            let vs = contract::version_status(repo_path, scope);
            print_scope(&scope.name, &scope_dir, &lang, &tag, &vs);
        }
    }

    // 工作区状态
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
    _tag: &Option<String>,
    vs: &contract::VersionStatus,
) {
    println!("  [{:<12}] {}", name, lang.name());

    // CI 状态（简化版：只检查 gh CLI 是否可用）
    let ci_status = check_ci_status(name);
    println!("    CI:         {}", ci_status);

    // 语法检查（根据语言类型）
    let syntax_ok = check_syntax(name);
    println!("    syntax:     {}", syntax_ok);

    // 版本一致性
    match (&vs.tag_version, &vs.config_version) {
        (Some(t), Some(c)) if t == c => {
            println!("    version:    ✅ {}（一致）", t);
        }
        (Some(t), Some(c)) => {
            println!("    version:    ⚠ tag {} ≠ 配置 {}", t, c);
        }
        (Some(t), None) => {
            println!("    version:    tag {}（无配置文件）", t);
        }
        (None, Some(c)) => {
            println!("    version:    配置 {}（无 tag）", c);
        }
        (None, None) => {
            println!("    version:    暂无发布");
        }
    }
}

fn latest_tag(repo_path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["tag", "--sort=-version:refname"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    std::str::from_utf8(&output.stdout)
        .ok()?
        .lines()
        .next()
        .map(|s| s.to_string())
}

fn latest_tag_for_scope(repo_path: &Path, scope_name: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["tag", "--sort=-version:refname"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let prefix = format!("{}/v", scope_name);
    let alt_prefix = format!("{}/", scope_name);
    std::str::from_utf8(&output.stdout)
        .ok()?
        .lines()
        .find(|t| t.starts_with(&prefix) || t.starts_with(&alt_prefix))
        .map(|s| s.to_string())
}

fn check_ci_status(_scope: &str) -> String {
    // 简化：用 gh CLI 检查最近一次 workflow 运行状态
    // 实验室版本只检查 gh 是否可用
    let check = std::process::Command::new("gh")
        .args(["--version"])
        .output();
    match check {
        Ok(o) if o.status.success() => "gh 可用（需配置）".to_string(),
        _ => "⚠ gh CLI 未安装".to_string(),
    }
}

fn check_syntax(scope: &str) -> String {
    // 简易语法检查
    let dir = Path::new(".").join(scope);
    if dir.join("Cargo.toml").exists() {
        let result = std::process::Command::new("cargo")
            .args([
                "check",
                "--manifest-path",
                &dir.join("Cargo.toml").to_string_lossy(),
            ])
            .output();
        match result {
            Ok(o) if o.status.success() => "✅ cargo check 通过".to_string(),
            Ok(_) => "❌ cargo check 失败".to_string(),
            Err(_) => "⚠ cargo 未安装".to_string(),
        }
    } else {
        "—".to_string()
    }
}

fn is_working_tree_dirty(repo_path: &Path) -> bool {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output();
    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}
