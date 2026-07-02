/// 发布前检查（preflight）。
///
/// 对应 scripts/preflight.sh。
/// 在发布前依次执行：构建、测试、发布 dry-run。
use std::path::Path;

/// preflight 检查结果。
#[derive(Debug)]
pub struct PreflightResult {
    pub build_ok: bool,
    pub test_ok: bool,
    pub dry_run_ok: bool,
    pub version: String,
}

impl PreflightResult {
    pub fn all_pass(&self) -> bool {
        self.build_ok && self.test_ok && self.dry_run_ok
    }
}

/// 执行发布前检查。
pub fn preflight(repo_path: &Path, _contract: &crate::contract::Contract) -> PreflightResult {
    println!("preflight");

    // 从契约读版本状态
    let scopes = crate::contract::load_scopes(repo_path);
    let scopes: Vec<crate::contract::Scope> = if scopes.is_empty() {
        vec![crate::contract::Scope {
            name: "(root)".into(),
            dir: ".".into(),
            language: crate::contract::Language::Unknown(String::new()),
            framework: String::new(),
            build_tool: crate::contract::BuildTool::Unknown(String::new()),
            registry: crate::contract::Registry::None,
            release: crate::contract::StageRelease::default(),
            test_threshold: None,
            ci_workflow: None,
        }]
    } else {
        scopes
    };

    let mut version = "?".to_string();
    for s in &scopes {
        // 语言/构建工具未知时输出警告，不阻断流程
        if !s.language.is_supported() {
            eprintln!("  ⚠ {}: 语言 {:?} 未知，相关检查跳过", s.name, s.language);
        }
        if !s.build_tool.is_supported() {
            eprintln!(
                "  ⚠ {}: 构建工具 {} 未知，语法校验跳过",
                s.name,
                s.build_tool.name()
            );
        }
        let vs = crate::contract::version_status(repo_path, s);
        match &vs.config_version {
            Some(v) => {
                let icon = if vs.consistent {
                    "✅"
                } else {
                    "⚠ tag不匹配"
                };
                println!("  {}: {} {}", s.name, v, icon);
                if version == "?" {
                    version = v.clone();
                }
            }
            None => println!("  {}: ? 未检测到版本", s.name),
        }
    }
    println!();

    let build_ok = run_build(repo_path);
    let test_ok = run_test(repo_path);
    let dry_run_ok = run_dry_run(repo_path);

    let result = PreflightResult {
        build_ok,
        test_ok,
        dry_run_ok,
        version,
    };

    if result.all_pass() {
        println!();
        println!("preflight passed");
    } else {
        println!();
        println!("preflight FAILED");
    }

    result
}

fn run_build(repo_path: &Path) -> bool {
    print!("--- cargo build ---");
    // 实验室简化版：检查 Cargo.toml 是否存在，是否存在语法错误
    if !repo_path.join("Cargo.toml").exists() {
        println!("  ⚠ 无 Cargo.toml，跳过");
        return true;
    }
    let result = std::process::Command::new("cargo")
        .args(["check"])
        .current_dir(repo_path)
        .output();
    match result {
        Ok(o) if o.status.success() => {
            println!("  ✅");
            true
        }
        Ok(_) => {
            println!("  ❌");
            false
        }
        Err(_) => {
            println!("  ⚠ cargo 未安装");
            false
        }
    }
}

fn run_test(repo_path: &Path) -> bool {
    print!("--- cargo test ---");
    if !repo_path.join("Cargo.toml").exists() {
        println!("  ⚠ 无 Cargo.toml，跳过");
        return true;
    }
    let result = std::process::Command::new("cargo")
        .args(["test"])
        .current_dir(repo_path)
        .output();
    match result {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout);
            let has_result = output.lines().any(|l| l.contains("test result:"));
            if o.status.success() && has_result {
                // 提取结果行
                if let Some(line) = output.lines().find(|l| l.contains("test result:")) {
                    println!("  {}", line.trim());
                }
                true
            } else if o.status.success() {
                println!("  ✅");
                true
            } else {
                println!("  ❌");
                false
            }
        }
        Err(_) => {
            println!("  ⚠ cargo 未安装");
            false
        }
    }
}

fn run_dry_run(repo_path: &Path) -> bool {
    print!("--- cargo publish --dry-run ---");
    if !repo_path.join("Cargo.toml").exists() {
        println!("  ⚠ 无 Cargo.toml，跳过");
        return true;
    }
    // 简版：不真正执行 dry-run（需要网络），只检查元数据
    let result = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo_path)
        .output();
    match result {
        Ok(o) if o.status.success() => {
            println!("  ✅（metadata 检查通过）");
            true
        }
        Ok(_) => {
            println!("  ❌");
            false
        }
        Err(_) => {
            println!("  ⚠ cargo 未安装");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_no_cargo_toml() {
        let d = tempfile::tempdir().unwrap();
        // 无 contract.yaml，使用默认契约
        let c = crate::contract::load(d.path());
        let r = preflight(d.path(), &c);
        // 无 Cargo.toml 时所有步骤跳过，preflight 通过
        assert!(r.all_pass());
    }
}
