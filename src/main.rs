use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod commands;
mod model;

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::HealthCheck { path } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|e| {
                eprintln!("错误: 无法解析路径 '{}': {}", path.display(), e);
                process::exit(1);
            });

            match model::RepoState::scan(&root) {
                Ok(state) => {
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
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
