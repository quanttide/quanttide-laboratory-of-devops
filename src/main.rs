/// 量潮DevOps实验室
///
/// 实现并演示 roadmap 中的各个命令模块和 CI 脚本：
/// - contract    scope 解析、语言检测、版本状态
/// - build       build status（CI、语法、版本一致性）
/// - code        code status（子模块三分法状态模型）
/// - test        test status（测试结果、覆盖率）
/// - validate    CI 验证（CHANGELOG、版本一致性）
/// - preflight   发布前检查（构建、测试、dry-run）
mod build;
mod code;
mod contract;
mod preflight;
mod test;
mod validate;

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
            let scope_dir = repo_path.join(&s.dir);
            let lang = contract::resolve_language(s, &scope_dir);
            println!("   scope: {:<12} dir: {:<20}", s.name, s.dir,);
            println!("    语言/框架:  {} / {}", lang.name(), s.framework);
            println!("    构建工具:   {}", s.build_tool.name());
            println!("    制品库:     {}", s.registry.name());
            println!("    CHANGELOG:  {}", s.release.changelog);
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

    // ── 5. validate ───────────────────────────────────────────────────
    println!("━━━ 5. CI 验证 ━━━");
    let changelog = repo_path.join("CHANGELOG.md");
    match validate::validate_changelog("v0.1.0", &changelog) {
        Ok(()) => println!("   CHANGELOG:     ✅ 包含版本 v0.1.0"),
        Err(e) => println!("   CHANGELOG:     ⚠ {}", e.join("; ")),
    }
    match validate::validate_version("v0.1.0", &repo_path) {
        Ok(v) => println!("   版本一致性:    ✅ {} 一致", v),
        Err(e) => println!("   版本一致性:    ⚠ {}", e),
    }
    println!();

    // ── 6. preflight ─────────────────────────────────────────────────
    println!("━━━ 6. preflight ━━━");
    let result = preflight::preflight(&repo_path);
    println!(
        "\n   构建: {}  测试: {}  dry-run: {}",
        if result.build_ok { "✅" } else { "❌" },
        if result.test_ok { "✅" } else { "❌" },
        if result.dry_run_ok { "✅" } else { "❌" },
    );
    println!();

    println!("🔬 演示结束。运行 cargo test 查看各模块单元测试。");
}
