/// 量潮DevOps实验室
///
/// 当前保留的实验性模块：
/// - preflight   发布前检查（构建、测试、dry-run）—— 尚未集成到 CLI
/// - release     发布流程编排 —— 原型，对应 devops-release skill
/// - detect     版本号自动检测（bin） —— 实验 devops-release 规则
mod preflight;
mod release;

fn main() {
    let repo_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "release" => run_release(&args[2..], &repo_path),
            _ => {
                eprintln!("未知子命令: {}", args[1]);
                eprintln!("可用: release");
                std::process::exit(1);
            }
        }
        return;
    }

    println!("🔬 量潮DevOps实验室\n");

    // ── preflight ─────────────────────────────────────────────────
    println!("━━━ preflight ━━━");
    let c = qtcloud_devops_cli::contract::load(&repo_path);
    let result = preflight::preflight(&repo_path, &c);
    println!(
        "\n   构建: {}  测试: {}  dry-run: {}",
        if result.build_ok { "✅" } else { "❌" },
        if result.test_ok { "✅" } else { "❌" },
        if result.dry_run_ok { "✅" } else { "❌" },
    );
    println!();

    // ── release status ────────────────────────────────────────────
    println!("━━━ release status ━━━");
    release::status(&repo_path);
    println!();

    // ── release precheck ──────────────────────────────────────────
    println!("━━━ release precheck ━━━");
    release::precheck(&repo_path);
    println!();

    println!("🔬 实验模块: preflight / release");
    println!("   实验 binary: cargo run --bin detect");
    println!("   发布演示: cargo run -- release publish v0.1.0-rc.1");
}

/// release 子命令路由。
fn run_release(args: &[String], repo_path: &std::path::Path) {
    if args.is_empty() {
        release::status(repo_path);
        return;
    }
    match args[0].as_str() {
        "status" => release::status(repo_path),
        "precheck" => {
            release::precheck(repo_path);
        }
        "publish" => {
            let version = if args.len() > 1 {
                &args[1]
            } else {
                eprintln!("用法: cargo run -- release publish <version>");
                eprintln!("示例: cargo run -- release publish v0.1.0-rc.1");
                std::process::exit(1);
            };
            let prerelease =
                version.contains("rc") || version.contains("alpha") || version.contains("beta");
            match release::publish(version, repo_path, prerelease) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("用法: cargo run -- release <status|precheck|publish>");
            std::process::exit(1);
        }
    }
}
