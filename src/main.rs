/// 量潮DevOps实验室
///
/// 实现并演示 roadmap 中的各个命令模块和 CI 脚本：
/// - contract    四维架构（Stages / Platforms / Sources / Scopes）
/// - build       build status（CI、语法、版本一致性）
/// - code        code status（子模块三分法状态模型）
/// - test        test status（测试结果、覆盖率）
/// - preflight   发布前检查（构建、测试、dry-run）
mod build;
mod code;
mod contract;
mod plan;
mod preflight;
mod test;

fn main() {
    let repo_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    println!("🔬 量潮DevOps实验室\n");

    // ── 1. 契约解析（四维架构） ───────────────────────────────────────
    println!("━━━ 1. 契约解析（四维架构） ━━━");
    let c = contract::load(&repo_path);
    println!("   Stages:");
    println!("     build:   {:?}", c.stages.build.command);
    println!("     test:    阈值 {}%", c.stages.test.threshold);
    println!(
        "     release: CHANGELOG={}, pre_publish={:?}",
        c.stages.release.changelog, c.stages.release.pre_publish
    );
    println!("   Platforms:");
    println!("     source_control:  {}", c.platforms.source_control);
    println!("     ci:             {}", c.platforms.ci);
    println!(
        "     artifact_reg:   {}",
        c.platforms.artifact_registry.name()
    );
    println!("   Sources:");
    println!(
        "     version: {:?} {:?}",
        c.sources.version.source_type, c.sources.version.path
    );
    if c.scopes.is_empty() {
        println!("   Scopes:   未定义");
    } else {
        for s in &c.scopes {
            let scope_dir = repo_path.join(&s.dir);
            let lang = contract::resolve_language(s, &scope_dir);
            println!("   Scopes:   {:<12} dir={}", s.name, s.dir);
            println!("     语言/框架:  {} / {}", lang.name(), s.framework);
            println!("     构建工具:   {}", s.build_tool.name());
            println!("     制品库:     {}", s.registry.name());
            println!("     CHANGELOG:  {}", s.release.changelog);
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
    test::status(&repo_path, &c);
    println!();

    // ── 5. preflight ─────────────────────────────────────────────────
    println!("━━━ 5. preflight ━━━");
    let result = preflight::preflight(&repo_path, &c);
    println!(
        "\n   构建: {}  测试: {}  dry-run: {}",
        if result.build_ok { "✅" } else { "❌" },
        if result.test_ok { "✅" } else { "❌" },
        if result.dry_run_ok { "✅" } else { "❌" },
    );
    println!();

    // ── 6. plan status（实验原型） ────────────────────────────────────
    println!("━━━ 6. plan status ━━━");
    let roadmap_path = repo_path.join("ROADMAP.md");
    if roadmap_path.exists() {
        plan::print_status(&roadmap_path);
    } else {
        println!("   未找到 ROADMAP.md");
    }
    println!();

    // ── 7. plan doctor（规则验证，不修复） ──────────────────────────
    println!("━━━ 7. plan doctor ━━━");
    if roadmap_path.exists() {
        let issues = plan::doctor_roadmap(&roadmap_path);
        if issues.is_empty() {
            println!("   ✅ 格式无误");
        } else {
            for f in &issues {
                println!("   ⚠ L{}: {}", f.line, f.issue);
            }
            println!("   规则仅做验证，修复由 LLM 完成（当前未接入）");
        }
    }
    println!();

    println!("🔬 演示结束。运行 cargo test 查看各模块单元测试。");
}
