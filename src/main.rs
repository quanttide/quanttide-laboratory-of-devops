use clap::{Parser, Subcommand};
use kse_core::commands::editor::GitSubmoduleEditor;
use kse_core::commands::export;
use kse_core::commands::{HealthIssue, SubmoduleEditor};
use kse_core::model;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "kse",
    about = "Git Submodule 专用编辑器 — 多仓库项目的子模块可视化工具"
)]
struct Cli {
    #[arg(global = true, long = "dry-run")]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 扫描并展示仓库所有子模块的状态
    Status {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 同步子模块指针到父仓库（核心贡献）
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    /// 退役子模块
    Retire {
        name: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// 查看操作历史
    History {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(default_value = "20", long = "limit", short = 'n')]
        limit: usize,
        #[arg(long = "submodule", short = 'm')]
        submodule: Option<String>,
        #[arg(long = "start")]
        start: Option<String>,
        #[arg(long = "end")]
        end: Option<String>,
    },
    /// 导出为可执行的 CI 脚本
    ExportCi {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(default_value = "shell", long = "format", short = 'f')]
        format: String,
        #[arg(long = "output", short = 'o')]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SyncAction {
    /// 子模块 → 父仓库：更新父仓库指针
    Parent {
        #[arg(default_value = ".")]
        repo: PathBuf,
        name: Option<String>,
    },
    /// 跨环境子模块版本对齐（CI 场景）
    Platform {
        name: String,
        #[arg(long = "env", short = 'e', default_value = "production")]
        env: String,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
}

fn resolve_path(path: &PathBuf) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|e| {
        eprintln!("错误: 无法解析路径 '{}': {}", path.display(), e);
        process::exit(1);
    })
}

fn print_issues(issues: &[HealthIssue]) {
    if !issues.is_empty() {
        println!("\n需要关注的子模块:");
        for issue in issues {
            println!("  [{}] {}", issue.submodule_name, issue.description);
            println!("        建议: {}", issue.suggested_action);
        }
    }
}

fn print_aggregate(state: &model::RepoState) {
    if let Ok((_, agg)) = model::RepoState::scan_all(&state.root_path) {
        println!("\n聚合统计:");
        println!("  总数: {}", agg.total);
        println!("  ✅ Clean: {}", agg.clean);
        if agg.ahead_of_parent > 0 {
            println!("  ⬆ AheadOfParent: {}", agg.ahead_of_parent);
        }
        if agg.behind_remote > 0 {
            println!("  ⬇ BehindRemote: {}", agg.behind_remote);
        }
        if agg.detached > 0 {
            println!("  ⚠ Detached: {}", agg.detached);
        }
        if agg.dirty > 0 {
            println!("  🔴 Dirty: {}", agg.dirty);
        }
        if agg.orphaned > 0 {
            println!("  💀 Orphaned: {}", agg.orphaned);
        }
        if agg.uninitialized > 0 {
            println!("  ⚪ Uninitialized: {}", agg.uninitialized);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let dry_run = cli.dry_run;

    match cli.command {
        Commands::Status { path } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root.clone());
            let state = model::RepoState::scan(&root).unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });
            let issues = editor.status().unwrap_or_else(|e| {
                eprintln!("错误: {}", e);
                process::exit(1);
            });

            println!("仓库: {}", state.root_path.display());
            println!("子模块总数: {}", state.total);
            println!("干净: {}", state.clean_count);
            if !state.needs_attention.is_empty() {
                println!("需要关注: {}", state.needs_attention.join(", "));
            }

            print_aggregate(&state);
            println!();

            if state.submodules.is_empty() && state.total == 0 {
                println!("  没有子模块");
            } else {
                println!(
                    "  {:<20} {:<15} {:<10} {:<8}",
                    "名称", "状态", "分支", "差异"
                );
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
            print_issues(&issues);
        }

        Commands::Sync {
            action: SyncAction::Platform { repo, name, env },
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 检查子模块 '{}' 在 {} 环境的版本", name, env);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_platform(&name, &env));
        }
        Commands::Sync {
            action:
                SyncAction::Parent {
                    repo,
                    name: Some(n),
                    ..
                },
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 同步子模块 '{}' 到父仓库", n);
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_to_parent(&n));
        }
        Commands::Sync {
            action: SyncAction::Parent { repo, .. },
        } => {
            let root = resolve_path(&repo);
            if dry_run {
                println!("[预览] 同步所有子模块到父仓库");
                return;
            }
            let editor = GitSubmoduleEditor::new(root);
            exec(editor.sync_all_to_parent());
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
            start,
            end,
        } => {
            let root = resolve_path(&path);
            let editor = GitSubmoduleEditor::new(root);
            match editor.list_history(
                limit,
                submodule.as_deref(),
                start.as_deref(),
                end.as_deref(),
            ) {
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
