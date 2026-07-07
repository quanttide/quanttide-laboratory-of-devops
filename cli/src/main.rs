/// 量潮DevOps实验室
///
/// 实验室是 CLI 功能的原型验证场。已推进到平台的功能不再保留副本。
///
/// 当前保留的实验性模块：
/// - git_experiment  git2 vs gix API 与性能对比（纯研究，无平台等价物）
/// - detect         版本号自动检测（bin）—— 尚未完全集成到 CLI
mod git_experiment;
mod gix_scan;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 子命令后可选跟仓库路径，默认当前目录
    let repo_path = args
        .get(2)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "git-exp" => git_experiment::run(&repo_path),
            "gix-scan" => gix_scan::run(&repo_path),
            _ => {
                eprintln!("未知子命令: {}", args[1]);
                eprintln!("可用: git-exp, gix-scan");
                eprintln!();
                eprintln!("发布/检查等操作请使用平台 CLI:");
                eprintln!("  qtcloud-devops status");
                eprintln!("  qtcloud-devops release publish -v <version> -y");
                std::process::exit(1);
            }
        }
        return;
    }

    println!("🔬 量潮DevOps实验室\n");
    println!("实验模块:");
    println!("  cargo run --bin quanttide-lab -- gix-scan  — gix 子模块扫描");
    println!("  cargo run --bin detect                      — 版本号自动检测");
    println!();
    println!("平台命令（已覆盖实验室原型）:");
    println!("  qtcloud-devops status                      — 统一状态");
    println!("  qtcloud-devops release publish -v <ver> -y — 发布");
}
