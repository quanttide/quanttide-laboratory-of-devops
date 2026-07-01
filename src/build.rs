use std::path::Path;

use crate::contract;

/// 输出当前仓库的构建状态（按 scope）。
pub fn status(repo_path: &Path) {
    let c = contract::load(repo_path);

    println!("构建状态");
    println!("{}", "-".repeat(50));

    if c.scopes.is_empty() {
        let lang = contract::detect_by_files(repo_path);
        let tag = latest_tag(repo_path);
        let vs = version_status_root(repo_path);
        print_scope("(root)", repo_path, &lang, &tag, &vs, &c);
    } else {
        for scope in &c.scopes {
            let scope_dir = repo_path.join(&scope.dir);
            if !scope_dir.exists() {
                println!("  [{}]     ⚠ 目录不存在: {}", scope.name, scope.dir);
                continue;
            }
            let lang = contract::resolve_language(scope, &scope_dir);
            let tag = latest_tag_for_scope(repo_path, &scope.name);
            let vs = contract::version_status(repo_path, scope);
            print_scope(&scope.name, &scope_dir, &lang, &tag, &vs, &c);
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
    _tag: &Option<String>,
    vs: &contract::VersionStatus,
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
    println!("    threshold:  {}%", c.stages.test.threshold);
}

fn version_status_root(repo_path: &Path) -> contract::VersionStatus {
    let tag = latest_tag(repo_path);
    let config = contract::detect_by_files(repo_path);
    let dir = repo_path;
    let config_ver = match config {
        contract::Language::Rust => read_simple_version(dir, "Cargo.toml"),
        _ => None,
    };
    let consistent = match (&tag, &config_ver) {
        (Some(t), Some(c)) => t == c,
        (None, None) => true,
        _ => false,
    };
    contract::VersionStatus {
        tag_version: tag,
        config_version: config_ver,
        consistent,
    }
}

fn read_simple_version(dir: &Path, filename: &str) -> Option<String> {
    let content = std::fs::read_to_string(dir.join(filename)).ok()?;
    for line in content.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("version = \"") {
            if let Some(end) = v.find('"') {
                return Some(v[..end].to_string());
            }
        }
    }
    None
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
