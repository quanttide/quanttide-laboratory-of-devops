/// 示例：使用 qtcloud-devops-cli 库的 release 模块。
///
/// 演示：
/// 1. 校验版本号格式
/// 2. 从 CHANGELOG 提取 release notes
/// 3. 校验 CHANGELOG 是否包含某版本记录
use std::path::Path;

use qtcloud_devops_cli::release;

fn main() {
    let version = "v0.1.0";

    // 1. 校验版本号
    if !release::validate_version(version) {
        eprintln!("版本号格式错误: {}", version);
        std::process::exit(1);
    }
    println!("✓ 版本号格式正确: {}", version);

    // 2. 解析 GitHub 仓库地址
    let remote = release::get_remote_repo(Path::new("."));
    match remote {
        Some(repo) => println!("  远程仓库: {}", repo),
        None => println!("  未检测到远程仓库"),
    }

    // 3. 检查当前目录下 CHANGELOG.md
    let changelog = Path::new("CHANGELOG.md");
    let errors = release::precheck_version_changelog(version, changelog);
    if errors.is_empty() {
        println!("✓ CHANGELOG 包含版本 {}", version);
    } else {
        for e in &errors {
            println!("  ⚠ {}", e);
        }
    }

    // 4. 提取 release notes
    if let Some(notes) = release::extract_notes(version, changelog) {
        println!("  Release notes 长度: {} 字", notes.len());
    }
}
