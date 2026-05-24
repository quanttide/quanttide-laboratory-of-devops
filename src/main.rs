use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "qtcloud-devops",
    about = "量潮DevOps工具 — Release 发布管理",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 发布 Release
    ///
    /// 默认行为：创建 Git 标签并推送 + GitHub Release（仓库从 git remote 自动检测）。
    /// --tag-only：仅打标签，跳过 GitHub Release。
    /// --release-only：仅为已有标签创建 GitHub Release（跳过标签创建）。
    Release {
        #[arg(long, short = 'V')]
        version: String,

        #[arg(long, default_value = "CHANGELOG.md")]
        changelog: String,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        tag_only: bool,

        #[arg(long)]
        release_only: bool,

        #[arg(long, short = 'y')]
        yes: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Release {
            version,
            changelog,
            dry_run,
            tag_only,
            release_only,
            yes,
        } => {
            if tag_only && release_only {
                eprintln!("错误: --tag-only 和 --release-only 不能同时使用");
                std::process::exit(1);
            }

            let code = qtcloud_devops_code::commands::release::run(
                &version,
                &PathBuf::from(&changelog),
                dry_run,
                tag_only,
                release_only,
                yes,
            );
            std::process::exit(code);
        }
    }
}
