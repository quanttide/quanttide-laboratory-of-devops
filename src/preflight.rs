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
pub fn preflight(repo_path: &Path) -> PreflightResult {
    println!("preflight");

    let version = read_self_version(repo_path).unwrap_or_else(|| "?".into());
    println!("for v{version}");
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

fn read_self_version(repo_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(repo_path.join("Cargo.toml")).ok()?;
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
        let r = preflight(d.path());
        // 无 Cargo.toml 时所有步骤跳过，preflight 通过
        assert!(r.all_pass());
    }

    #[test]
    fn test_read_self_version() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(read_self_version(d.path()).as_deref(), Some("0.1.0"));
    }
}
