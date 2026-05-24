use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "qtcloud-devops",
    about = "量潮DevOps工具 — 软件发布生命周期管理",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 将版本部署至预发布/灰度环境，进入 Staged 状态
    Stage {
        #[arg(short = 'V', long)]
        version: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// 将 Staged 版本正式发布上线
    Publish {
        #[arg(short = 'V', long)]
        version: String,
        #[arg(long, default_value = "CHANGELOG.md")]
        changelog: String,
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// 取消 Staged 版本的发布
    Cancel {
        #[arg(short = 'V', long)]
        version: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// 将已上线的版本标记为退役
    Retire {
        #[arg(short = 'V', long)]
        version: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
}

fn repo_path() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Stage { version, reason } => {
            qtcloud_devops_code::commands::stage::run(&version, &reason, &repo_path())
        }
        Commands::Publish {
            version,
            changelog,
            yes,
        } => qtcloud_devops_code::commands::publish::run(
            &version,
            &PathBuf::from(&changelog),
            &repo_path(),
            yes,
        ),
        Commands::Cancel { version, reason } => {
            qtcloud_devops_code::commands::cancel::run(&version, &reason, &repo_path())
        }
        Commands::Retire { version, reason } => {
            qtcloud_devops_code::commands::retire::run(&version, &reason, &repo_path())
        }
    };

    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}
