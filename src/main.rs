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
        #[arg(short = 'v', long)]
        version: String,
    },
    /// 将 Staged 版本正式发布上线
    Publish {
        #[arg(short = 'v', long)]
        version: String,
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// 取消 Staged 版本的发布
    Cancel {
        #[arg(short = 'v', long)]
        version: String,
    },
    /// 将已上线的版本标记为退役
    Retire {
        #[arg(short = 'v', long)]
        version: String,
    },
    /// 查看发布状态
    ReleaseStatus,
    /// 扫描项目管理文件，生成规划摘要
    Plan,
    /// 执行项目构建
    Build {
        #[arg(long)]
        release: bool,
    },
    /// 执行测试
    Test {
        #[arg(long)]
        name: Option<String>,
    },
}

fn repo_path() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), String> = match cli.command {
        Commands::Stage { version } => {
            qtcloud_devops_code::commands::stage::run(&version, &repo_path()).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Publish { version, yes } => {
            qtcloud_devops_code::commands::publish::run(&version, &repo_path(), yes).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Cancel { version } => {
            qtcloud_devops_code::commands::cancel::run(&version, &repo_path()).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Retire { version } => {
            qtcloud_devops_code::commands::retire::run(&version, &repo_path()).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::ReleaseStatus => {
            qtcloud_devops_code::commands::release_status::run(&repo_path()).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Plan => {
            qtcloud_devops_code::commands::plan::run(&repo_path()).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Build { release } => {
            qtcloud_devops_code::commands::build::run(&repo_path(), release).map(|_| ())
                .map_err(|e| format!("{}", e))
        }
        Commands::Test { name } => {
            qtcloud_devops_code::commands::test::run(&repo_path(), name.as_deref()).map(|_| ())
                .map_err(|e| format!("{}", e))
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
