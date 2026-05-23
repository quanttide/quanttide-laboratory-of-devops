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

fn generate_shell_script(_all: &[Submodule], needs_update: &[&Submodule]) -> String {
    let mut s = String::new();
    s.push_str("#!/bin/bash\n");
    s.push_str("# KSE 自动生成的子模块更新脚本\n");
    s.push_str(&format!("# 生成时间: {}\n\n", timestamp()));
    s.push_str("set -e\n\n");
    if needs_update.is_empty() {
        s.push_str("# 所有子模块已是最新状态\n");
    } else {
        s.push_str("# 需要更新的子模块 (请使用 git submodule update):\n");
        for sm in needs_update {
            s.push_str(&format!(
                "#   git submodule update --remote --merge {}\n",
                sm.name
            ));
        }
        s.push_str("\n# 同步到父仓库\n");
        s.push_str("kse sync parent --all\n");
    }
    s
}

fn generate_github_actions(_all: &[Submodule], needs_update: &[&Submodule]) -> String {
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
                "        run: git -C {} submodule update --remote --merge\n",
                sm.name
            ));
        }
        s.push_str("      - name: Sync to parent\n");
        s.push_str("        run: ./target/release/kse sync parent --all\n");
    } else {
        s.push_str("      - name: Check status\n");
        s.push_str("        run: ./target/release/kse status\n");
    }
    s
}

fn generate_gitlab_ci(_all: &[Submodule], needs_update: &[&Submodule]) -> String {
    let mut s = String::new();
    s.push_str("stages:\n  - update\n\nupdate-submodules:\n");
    s.push_str("  stage: update\n");
    s.push_str("  script:\n");
    s.push_str("    - apt-get update && apt-get install -y libgit2-dev\n");
    s.push_str("    - cargo build --release\n");
    if !needs_update.is_empty() {
        for sm in needs_update {
            s.push_str(&format!(
                "    - git -C {} submodule update --remote --merge\n",
                sm.name
            ));
        }
        s.push_str("    - ./target/release/kse sync parent --all\n");
    }
    s.push_str("  only:\n    - schedules\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CommitHash;
    use std::path::PathBuf;

    fn make_state(input: Vec<(SubmoduleStatus, &str)>) -> RepoState {
        let total = input.len();
        let clean_count = input
            .iter()
            .filter(|(s, _)| *s == SubmoduleStatus::Clean)
            .count();
        let needs_attention: Vec<String> = input
            .iter()
            .filter(|(s, _)| *s != SubmoduleStatus::Clean)
            .map(|(_, n)| n.to_string())
            .collect();

        let submodules: Vec<Submodule> = input
            .into_iter()
            .map(|(status, name)| Submodule {
                name: name.into(),
                path: PathBuf::from(name),
                url: format!("https://example.com/{}.git", name),
                tracked_branch: "main".into(),
                parent_pointer: CommitHash::default(),
                local_head: CommitHash::default(),
                remote_head: CommitHash::default(),
                status,
                ahead_count: 0,
                behind_count: 0,
                remote_unreachable: false,
            })
            .collect();

        RepoState {
            root_path: "/tmp/repo".into(),
            submodules,
            total,
            clean_count,
            needs_attention,
        }
    }

    #[test]
    fn test_shell_script_with_updates() {
        let state = make_state(vec![
            (SubmoduleStatus::Clean, "lib-clean"),
            (SubmoduleStatus::BehindRemote, "lib-behind"),
            (SubmoduleStatus::Uninitialized, "lib-new"),
        ]);
        let script = generate_ci_script(&state, "shell");
        assert!(script.starts_with("#!/bin/bash"));
        assert!(script.contains("lib-behind"));
        assert!(script.contains("lib-new"));
        assert!(script.contains("sync parent"));
        assert!(!script.contains("lib-clean"));
    }

    #[test]
    fn test_shell_script_all_clean() {
        let state = make_state(vec![
            (SubmoduleStatus::Clean, "lib-a"),
            (SubmoduleStatus::Clean, "lib-b"),
        ]);
        let script = generate_ci_script(&state, "shell");
        assert!(script.starts_with("#!/bin/bash"));
        assert!(script.contains("已是最新"));
        assert!(!script.contains("kse sync-all"));
    }

    #[test]
    fn test_github_actions_format() {
        let state = make_state(vec![(SubmoduleStatus::BehindRemote, "lib-x")]);
        let script = generate_ci_script(&state, "github");
        assert!(script.contains("name: Update Submodules"));
        assert!(script.contains("actions/checkout@v4"));
        assert!(script.contains("lib-x"));
        assert!(script.contains("cargo build --release"));
    }

    #[test]
    fn test_github_actions_all_clean() {
        let state = make_state(vec![(SubmoduleStatus::Clean, "lib-a")]);
        let script = generate_ci_script(&state, "github");
        assert!(script.contains("name: Update Submodules"));
        assert!(script.contains("kse status"));
        assert!(!script.contains("submodule update"));
    }

    #[test]
    fn test_gitlab_ci_format() {
        let state = make_state(vec![(SubmoduleStatus::BehindRemote, "lib-y")]);
        let script = generate_ci_script(&state, "gitlab");
        assert!(script.contains("stages:"));
        assert!(script.contains("update-submodules:"));
        assert!(script.contains("lib-y"));
        assert!(script.contains("schedules"));
    }

    #[test]
    fn test_all_formats_produce_output() {
        let state = make_state(vec![(SubmoduleStatus::BehindRemote, "lib-z")]);
        for fmt in &["shell", "github", "gitlab"] {
            let script = generate_ci_script(&state, fmt);
            assert!(!script.is_empty(), "format {} should produce output", fmt);
        }
    }

    #[test]
    fn test_default_format_is_shell() {
        let state = make_state(vec![]);
        let script = generate_ci_script(&state, "unknown");
        assert!(script.starts_with("#!/bin/bash"));
    }
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
