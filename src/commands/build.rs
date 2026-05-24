use std::path::Path;
use std::process::Command;

pub fn run(repo_path: &Path, release: bool) -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml = repo_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err("当前目录不是 Rust 项目（未找到 Cargo.toml）".into());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.current_dir(repo_path);

    if release {
        cmd.arg("--release");
    }

    println!("构建中...");
    let start = std::time::Instant::now();

    let output = cmd.output()?;
    let elapsed = start.elapsed();

    if output.status.success() {
        let time_str = if elapsed.as_secs() > 0 {
            format!("{}.{:0>3}s", elapsed.as_secs(), elapsed.subsec_millis())
        } else {
            format!("{}ms", elapsed.subsec_millis())
        };
        println!("构建成功 ({})", time_str);
        Ok(time_str)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("构建失败:");
        for line in stderr.lines().take(10) {
            eprintln!("  {}", line);
        }
        Err("构建失败".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_build_no_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let result = run(dir.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
    }

    #[test]
    fn test_build_with_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        // Create a minimal Cargo project
        let cargo_content = r#"[package]
name = "test-build"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        std::fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut f = std::fs::File::create(dir.path().join("src/lib.rs")).unwrap();
        writeln!(f, "pub fn hello() -> &'static str {{ \"hello\" }}").unwrap();

        let result = run(dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_release_mode() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_content = r#"[package]
name = "test-build-rel"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        std::fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut f = std::fs::File::create(dir.path().join("src/lib.rs")).unwrap();
        writeln!(f, "pub fn hello() -> &'static str {{ \"hello\" }}").unwrap();

        let result = run(dir.path(), true);
        assert!(result.is_ok());
    }
}
