use crate::model::{RepoState, Submodule, SubmoduleStatus};

pub fn generate_ci_script(state: &RepoState, format: &str) -> String {
    let submodules = &state.submodules;
    let needs_update: Vec<&Submodule> = submodules
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                SubmoduleStatus::BehindRemote
                    | SubmoduleStatus::Uninitialized
                    | SubmoduleStatus::Detached
                    | SubmoduleStatus::Dirty
            )
        })
        .collect();

    match format {
        "github" => generate_github_actions(submodules, &needs_update),
        "gitlab" => generate_gitlab_ci(submodules, &needs_update),
        _ => generate_shell_script(submodules, &needs_update),
    }
}

fn generate_shell_script(
    _all: &[Submodule],
    needs_update: &[&Submodule],
) -> String {
    let mut s = String::new();
    s.push_str("#!/bin/bash\n");
    s.push_str("# KSE 自动生成的子模块更新脚本\n");
    s.push_str(&format!("# 生成时间: {}\n\n", timestamp()));
    s.push_str("set -e\n\n");
    if needs_update.is_empty() {
        s.push_str("# 所有子模块已是最新状态\n");
    } else {
        s.push_str("# 需要更新的子模块:\n");
        for sm in needs_update {
            s.push_str(&format!("kse update {} --strategy fast-forward\n", sm.name));
        }
        s.push_str("\n# 同步到父仓库\n");
        s.push_str("kse sync-all\n");
    }
    s
}

fn generate_github_actions(
    _all: &[Submodule],
    needs_update: &[&Submodule],
) -> String {
    let mut s = String::new();
    s.push_str("name: Update Submodules\n\n");
    s.push_str("on:\n  workflow_dispatch:\n  schedule:\n    - cron: '0 6 * * 1'\n\n");
    s.push_str("jobs:\n  update:\n    runs-on: ubuntu-latest\n");
    s.push_str("    steps:\n");
    s.push_str("      - uses: actions/checkout@v4\n");
    s.push_str("        with:\n          submodules: true\n");
    s.push_str("      - uses: actions-rust-lang/setup-rust-toolchain@v1\n");
    s.push_str("      - name: Build KSE\n");
    s.push_str("        run: cargo build --release\n");
    if !needs_update.is_empty() {
        s.push_str("      - name: Update submodules\n");
        for sm in needs_update {
            s.push_str(&format!(
                "        run: ./target/release/kse update {} --strategy fast-forward\n",
                sm.name
            ));
        }
        s.push_str("      - name: Sync to parent\n");
        s.push_str("        run: ./target/release/kse sync-all\n");
    } else {
        s.push_str("      - name: Check status\n");
        s.push_str("        run: ./target/release/kse health-check\n");
    }
    s
}

fn generate_gitlab_ci(
    _all: &[Submodule],
    needs_update: &[&Submodule],
) -> String {
    let mut s = String::new();
    s.push_str("stages:\n  - update\n\nupdate-submodules:\n");
    s.push_str("  stage: update\n");
    s.push_str("  script:\n");
    s.push_str("    - apt-get update && apt-get install -y libgit2-dev\n");
    s.push_str("    - cargo build --release\n");
    if !needs_update.is_empty() {
        for sm in needs_update {
            s.push_str(&format!(
                "    - ./target/release/kse update {} --strategy fast-forward\n",
                sm.name
            ));
        }
        s.push_str("    - ./target/release/kse sync-all\n");
    }
    s.push_str("  only:\n    - schedules\n");
    s
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    let year = 1970 + (days / 365) as u32;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        1 + ((days % 365) / 28),
        1 + ((days % 365) % 28),
        h,
        m,
        s
    )
}
