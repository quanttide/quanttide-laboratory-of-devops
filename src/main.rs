/// 量潮DevOps实验室
///
/// 演示 `build status` 命令和 `contract` 模块的用法。
/// 示例代码见 `apps/qtcloud-devops/src/cli/examples/release.rs`。
mod build;
mod contract;

fn main() {
    let repo_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    println!("🔬 量潮DevOps实验室 — build status\n");

    // 演示 contract 模块：scope 解析
    let scopes = contract::load_scopes(&repo_path);
    println!("📋 contract.yaml 解析:");
    if scopes.is_empty() {
        println!("   未找到 .quanttide/devops/contract.yaml");
    } else {
        for s in &scopes {
            let lang = contract::detect_language(&repo_path.join(&s.dir));
            println!(
                "   scope: {:<12} dir: {:<20} lang: {}",
                s.name,
                s.dir,
                lang.name()
            );
        }
    }
    println!();

    // 演示 build status
    build::status(&repo_path);
}
