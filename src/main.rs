/// 量潮DevOps实验室
///
/// 实现并演示 roadmap 中的各个命令模块：
/// - contract    scope 解析、语言检测、版本状态
/// - build       build status（CI、语法、版本一致性）
/// - code        code status（子模块三分法状态模型）
/// - test        test status（测试结果、覆盖率）
mod build;
mod code;
mod contract;
mod test;

fn main() {
    let repo_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    println!("🔬 量潮DevOps实验室\n");

    // ── 1. contract 演示 ──────────────────────────────────────────────
    println!("━━━ 1. contract.yaml 解析 ━━━");
    let scopes = contract::load_scopes(&repo_path);
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

    // ── 2. build status ───────────────────────────────────────────────
    println!("━━━ 2. build status ━━━");
    build::status(&repo_path);
    println!();

    // ── 3. code status（三分法模型演示） ──────────────────────────────
    println!("━━━ 3. code status ━━━");
    let issues = code::scan_submodules(&repo_path);
    if issues.is_empty() {
        println!("   未检测到子模块配置（无 .gitmodules）");
        println!("   演示：构造测试数据运行 cargo test");
    } else {
        for issue in &issues {
            println!(
                "   {} [{:<12}] {} — {}",
                issue.severity, issue.submodule, issue.description, issue.suggested_action
            );
        }
    }
    println!();

    // ── 4. test status ───────────────────────────────────────────────
    println!("━━━ 4. test status ━━━");
    test::status(&repo_path);
    println!();

    println!("🔬 演示结束。运行 cargo test 查看各模块单元测试。");
}
