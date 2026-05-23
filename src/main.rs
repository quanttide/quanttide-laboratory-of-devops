use clap::{Parser, Subcommand};
use kse_core::commands::editor::GitSubmoduleEditor;
use kse_core::commands::export;
use kse_core::commands::{SubmoduleEditor, UpdateStrategy};
use kse_core::model;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "kse", about = "Git Submodule 专用编辑器 — 多仓库项目的子模块可视化工具")]
struct Cli {
    /// 预览模式：仅输出计划，不执行任何操作
    #[arg(global = true, long = "dry-run")]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 扫描并展示仓库所有子模块的状态
    HealthCheck {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 添加一个新的子模块
    Add {
        url: String,
        path: String,
        #[arg(default_value = "main", long = "branch", short = 'b')]
        branch: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 初始化所有未初始化的子模块
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 更新单个子模块
    Update {
        name: String,
        #[arg(default_value = "fast-forward", long = "strategy", short = 's')]
        strategy: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 更新所有子模块
    UpdateAll {
        #[arg(default_value = "fast-forward", long = "strategy", short = 's')]
        strategy: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 同步子模块指针到父仓库
    Sync {
        name: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 同步所有子模块指针到父仓库
    SyncAll {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 切换子模块分支
    Checkout {
        name: String,
        branch: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 在子模块中创建并切换到新分支
    Branch {
        name: String,
        branch: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 退役（软删除）一个子模块
    Retire {
        name: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 将所有子模块切换到指定分支
    CheckoutAll {
        branch: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 在所有子模块中创建并切换到新分支
    BranchAll {
        branch: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 查看操作历史
    History {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(default_value = "20", long = "limit", short = 'n')]
        limit: usize,
        #[arg(long = "submodule", short = 'm')]
        submodule: Option<String>,
    },
    /// 导出为可执行的 CI 脚本
    ExportCi {
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        path: PathBuf,
        /// 输出格式: shell / github / gitlab
        #[arg(default_value = "shell", long = "format", short = 'f')]
        format: String,
        /// 输出文件路径，默认输出到 stdout
        #[arg(long = "output", short = 'o')]
        output: Option<PathBuf>,
    },
}

fn parse_strategy(s: &str) -> Result<UpdateStrategy, String> {
    match s.to_lowercase().replace("-", "_").as_str() {
        "fastforward" | "ff" | "fast_forward" => Ok(UpdateStrategy::FastForward),
        "rebase" => Ok(UpdateStrategy::Rebase),
        "merge" => Ok(UpdateStrategy::Merge),
        _ => Err(format!(
            "未知策略 '{}'，可选值: fast-forward, rebase, merge",
            s
        )),
    }
}

fn resolve_path(path: &PathBuf) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|e| {
        eprintln!("错误: 无法解析路径 '{}': {}", path.display(), e);
        process::exit(1);
    })
}

fn main() {
    let cli = Cli::parse();
    let dry_run = cli.dry_run;

    match cli.command {
        Commands::HealthCheck { path } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root.clone());
            match editor.health_check() {
                Ok(issues) => {
                    let state = model::RepoState::scan(&root).unwrap_or_else(|e| {
                        eprintln!("错误: {}", e);
                        process::exit(1);
                    });
                    println!("仓库: {}", state.root_path.display());
                    println!("子模块总数: {}", state.total);
                    println!("干净: {}", state.clean_count);
                    if !state.needs_attention.is_empty() {
                        println!("需要关注: {}", state.needs_attention.join(", "));
                    }
                    println!();
                    if state.submodules.is_empty() && state.total == 0 {
                        println!("  没有子模块");
                    } else {
                        println!("  {:<20} {:<15} {:<10} {:<8} {}", "名称", "状态", "分支", "差异", "");
                        for sm in &state.submodules {
                            let diff = if sm.ahead_count > 0 && sm.behind_count > 0 {
                                format!("+{}/-{}", sm.ahead_count, sm.behind_count)
                            } else if sm.ahead_count > 0 {
                                format!("+{}", sm.ahead_count)
                            } else if sm.behind_count > 0 {
                                format!("-{}", sm.behind_count)
                            } else {
                                String::new()
                            };
                            println!(
                                "  {:<20} {:<15} {:<10} {:<8}",
                                sm.name,
                                format!("{:?}", sm.status),
                                sm.tracked_branch,
                                diff,
                            );
                        }
                    }
                    if !issues.is_empty() {
                        println!("\n健康问题:");
                        for issue in &issues {
                            println!("  [{}] {}", issue.submodule_name, issue.description);
                            println!("        建议: {}", issue.suggested_action);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Add {
            url,
            path,
            branch,
            repo,
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 添加子模块: url={}, path={}, branch={}", url, path, branch);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.add_submodule(&url, &path, &branch));
        }
        Commands::Init { path } => {
            let root = resolve_path(&path);
            if dry_run {
                println!("[预览] 初始化所有未初始化的子模块");
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.init_all());
        }
        Commands::Update {
            name,
            strategy,
            repo,
        } => {
            let root = resolve_path(&repo);
            let strategy = parse_strategy(&strategy).unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });
            if dry_run {
                println!("[预览] 更新子模块 '{}' (策略: {:?})", name, strategy);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.update_single(&name, strategy));
        }
        Commands::UpdateAll { strategy, path } => {
            let root = resolve_path(&path);
            let strategy = parse_strategy(&strategy).unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });
            if dry_run {
                println!("[预览] 更新所有子模块 (策略: {:?})", strategy);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.update_all(strategy));
        }
        Commands::Sync { name, repo } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 同步子模块 '{}' 到父仓库", name);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_to_parent(&name));
        }
        Commands::SyncAll { path } => {
            let root = resolve_path(&path);
            if dry_run {
                println!("[预览] 同步所有子模块到父仓库");
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_all_to_parent());
        }
        Commands::Checkout {
            name,
            branch,
            repo,
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 切换子模块 '{}' 到分支 '{}'", name, branch);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.checkout_branch(&name, &branch));
        }
        Commands::Branch {
            name,
            branch,
            repo,
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 在子模块 '{}' 创建并切换到分支 '{}'", name, branch);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.create_branch(&name, &branch));
        }
        Commands::Retire { name, repo } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 退役子模块 '{}'", name);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.retire_submodule(&name));
        }
        Commands::History {
            path,
            limit,
            submodule,
        } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root);
            match editor.list_history(limit, submodule.as_deref()) {
                Ok(records) => {
                    if records.is_empty() {
                        println!("没有操作历史记录");
                    } else {
                        println!("最近 {} 条操作记录:\n", records.len());
                        for r in &records {
                            let icon = if r.success { "✓" } else { "✗" };
                            println!(
                                "  {} [{}] {}: {} ({})",
                                icon, r.timestamp, r.action, r.submodule_name, r.detail
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::CheckoutAll { branch, path } => {
            let root = resolve_path(&path);
            if dry_run {
                println!("[预览] 将所有子模块切换到分支 '{}'", branch);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.checkout_all(&branch));
        }
        Commands::BranchAll { branch, path } => {
            let root = resolve_path(&path);
            if dry_run {
                println!("[预览] 在所有子模块中创建分支 '{}'", branch);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.branch_all(&branch));
        }
        Commands::ExportCi {
            path,
            format,
            output,
        } => {
            let root = resolve_path(&path);
            let state = model::RepoState::scan(&root).unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });
            let script = export::generate_ci_script(&state, &format);
            match output {
                Some(file) => {
                    std::fs::write(&file, &script).unwrap_or_else(|e| {
                        eprintln!("写入文件失败: {}", e);
                        process::exit(1);
                    });
                    println!("已导出到 {}", file.display());
                }
                None => {
                    println!("{}", script);
                }
            }
        }
    }
}

fn exec<T>(result: Result<T, Box<dyn std::error::Error>>) {
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            process::exit(1);
        }
    }
}
