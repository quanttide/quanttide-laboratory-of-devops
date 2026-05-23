use clap::{Parser, Subcommand};
use kse_core::commands::editor::GitSubmoduleEditor;
use kse_core::commands::{SubmoduleEditor, UpdateStrategy};
use kse_core::model;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "kse", about = "Git Submodule 专用编辑器 — 多仓库项目的子模块可视化工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 扫描并展示仓库所有子模块的状态
    HealthCheck {
        /// 仓库根目录路径，默认为当前目录
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 添加一个新的子模块
    Add {
        /// 子模块 URL
        url: String,
        /// 子模块路径
        path: String,
        /// 跟踪分支，默认为 main
        #[arg(default_value = "main", long = "branch", short = 'b')]
        branch: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 初始化所有未初始化的子模块
    Init {
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 更新单个子模块
    Update {
        /// 子模块名称
        name: String,
        /// 更新策略（fast-forward / rebase / merge）
        #[arg(default_value = "fast-forward", long = "strategy", short = 's')]
        strategy: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 更新所有子模块
    UpdateAll {
        /// 更新策略（fast-forward / rebase / merge）
        #[arg(default_value = "fast-forward", long = "strategy", short = 's')]
        strategy: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 同步子模块指针到父仓库
    Sync {
        /// 子模块名称
        name: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 同步所有子模块指针到父仓库
    SyncAll {
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 切换子模块分支
    Checkout {
        /// 子模块名称
        name: String,
        /// 目标分支
        branch: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 在子模块中创建并切换到新分支
    Branch {
        /// 子模块名称
        name: String,
        /// 新分支名称
        branch: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 退役（软删除）一个子模块
    Retire {
        /// 子模块名称
        name: String,
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 查看操作历史
    History {
        /// 仓库根目录路径
        #[arg(default_value = ".")]
        path: PathBuf,
        /// 显示条数
        #[arg(default_value = "20", long = "limit", short = 'n')]
        limit: usize,
        /// 按子模块名称筛选
        #[arg(long = "submodule", short = 'm')]
        submodule: Option<String>,
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
                        for sm in &state.submodules {
                            println!(
                                "  {:<20} {:<15} {}",
                                sm.name,
                                format!("{:?}", sm.status),
                                sm.tracked_branch,
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
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.add_submodule(&url, &path, &branch));
        }
        Commands::Init { path } => {
            let root = resolve_path(&path);
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
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.update_single(&name, strategy));
        }
        Commands::UpdateAll { strategy, path } => {
            let root = resolve_path(&path);
            let strategy = parse_strategy(&strategy).unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.update_all(strategy));
        }
        Commands::Sync { name, repo } => {
            let root = resolve_path(&repo);
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_to_parent(&name));
        }
        Commands::SyncAll { path } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_all_to_parent());
        }
        Commands::Checkout {
            name,
            branch,
            repo,
        } => {
            let root = resolve_path(&repo);
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.checkout_branch(&name, &branch));
        }
        Commands::Branch {
            name,
            branch,
            repo,
        } => {
            let root = resolve_path(&repo);
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.create_branch(&name, &branch));
        }
        Commands::Retire { name, repo } => {
            let root = resolve_path(&repo);
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.retire_submodule(&name));
        }
        Commands::History {
            path,
            limit,
            submodule,
        } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root.clone());
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
