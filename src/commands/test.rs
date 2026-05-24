use std::path::Path;
use std::process::Command;

pub fn run(repo_path: &Path, name_filter: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml = repo_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err("当前目录不是 Rust 项目（未找到 Cargo.toml）".into());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.current_dir(repo_path);

    if let Some(filter) = name_filter {
        cmd.arg("--");
        cmd.arg(filter);
    }

    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let all_output = format!("{}\n{}", stdout, stderr);

    // Parse test results from output
    let passed = all_output.lines().filter(|l| l.contains("... ok")).count();
    let failed: Vec<&str> = all_output.lines().filter(|l| l.contains("... FAILED")).collect();
    let total = passed + failed.len();

    println!("测试结果");
    println!("{}", "-".repeat(40));
    println!("  总数: {}", total);
    println!("  通过: {}", passed);
    println!("  失败: {}", failed.len());

    if !failed.is_empty() {
        println!();
        println!("失败用例:");
        for f in &failed {
            let test_name = f.split("...").next().unwrap_or(f).trim();
            println!("  {}", test_name);
        }
    }

    if output.status.success() {
        Ok(format!("{}/{}", passed, total))
    } else {
        Err(format!("{} 个测试失败", failed.len()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_test_no_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let result = run(dir.path(), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
    }

    #[test]
    fn test_test_runs_with_project() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_content = r#"[package]
name = "test-test"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
tempfile = "3"
"#;
        std::fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut f = std::fs::File::create(dir.path().join("src/lib.rs")).unwrap();
        writeln!(f, r#"
#[test]
fn test_passes() {{ assert_eq!(2 + 2, 4); }}

#[test]
fn test_also_passes() {{ assert!(true); }}
"#).unwrap();

        let result = run(dir.path(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2/2");
    }

    #[test]
    fn test_test_with_name_filter() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_content = r#"[package]
name = "test-filter"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
tempfile = "3"
"#;
        std::fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut f = std::fs::File::create(dir.path().join("src/lib.rs")).unwrap();
        writeln!(f, r#"
#[test]
fn test_foo() {{ assert_eq!(1, 1); }}

#[test]
fn test_bar() {{ assert_eq!(2, 2); }}
"#).unwrap();

        let result = run(dir.path(), Some("test_foo"));
        assert!(result.is_ok());
    }
}
