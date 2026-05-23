#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kse_core::commands::editor::GitSubmoduleEditor;
use kse_core::commands::export;
use kse_core::commands::SubmoduleEditor;
use kse_core::model::{AggregateStatus, RepoState};
use std::path::PathBuf;

#[tauri::command]
fn scan_repo(path: String) -> Result<ScanResult, String> {
    let root = PathBuf::from(&path);
    let state = RepoState::scan(&root).map_err(|e| format!("扫描失败: {}", e))?;

    let submodules: Vec<SubmoduleInfo> = state
        .submodules
        .into_iter()
        .map(|sm| SubmoduleInfo {
            name: sm.name,
            path: sm.path.display().to_string(),
            url: sm.url,
            tracked_branch: sm.tracked_branch,
            parent_pointer: sm.parent_pointer.to_string(),
            local_head: sm.local_head.to_string(),
            remote_head: sm.remote_head.to_string(),
            status: format!("{:?}", sm.status),
            ahead_count: sm.ahead_count,
            behind_count: sm.behind_count,
            remote_unreachable: sm.remote_unreachable,
        })
        .collect();

    let agg = AggregateStatus::from_submodules(&state.submodules);
    Ok(ScanResult {
        submodules,
        aggregate: AggregateInfo {
            total: agg.total,
            clean: agg.clean,
            ahead_of_parent: agg.ahead_of_parent,
            behind_remote: agg.behind_remote,
            detached: agg.detached,
            dirty: agg.dirty,
            orphaned: agg.orphaned,
            uninitialized: agg.uninitialized,
        },
    })
}

#[tauri::command]
fn status(path: String) -> Result<Vec<IssueInfo>, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    let issues = editor.status().map_err(|e| format!("状态检查失败: {}", e))?;
    Ok(issues
        .into_iter()
        .map(|i| IssueInfo {
            submodule_name: i.submodule_name,
            status: format!("{:?}", i.status),
            description: i.description,
            suggested_action: i.suggested_action,
        })
        .collect())
}

#[tauri::command]
fn sync_to_parent(repo: String, name: String) -> Result<String, String> {
    let root = PathBuf::from(&repo);
    let editor = GitSubmoduleEditor::new(root);
    editor
        .sync_to_parent(&name)
        .map_err(|e| format!("同步失败: {}", e))?;
    Ok(format!("已同步子模块 '{}' 到父仓库", name))
}

#[tauri::command]
fn sync_all_to_parent(path: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    editor
        .sync_all_to_parent()
        .map_err(|e| format!("批量同步失败: {}", e))?;
    Ok("已同步所有子模块到父仓库".into())
}

#[tauri::command]
fn retire_submodule(repo: String, name: String) -> Result<String, String> {
    let root = PathBuf::from(&repo);
    let editor = GitSubmoduleEditor::new(root);
    editor
        .retire_submodule(&name)
        .map_err(|e| format!("退役失败: {}", e))?;
    Ok(format!("已退役子模块 '{}'", name))
}

#[tauri::command]
fn export_ci(path: String, format: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    let state = RepoState::scan(&root).map_err(|e| format!("扫描失败: {}", e))?;
    Ok(export::generate_ci_script(&state, &format))
}

#[tauri::command]
fn list_history(
    path: String,
    limit: usize,
    submodule: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<HistoryRecord>, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    let records = editor
        .list_history(limit, submodule.as_deref(), start_date.as_deref(), end_date.as_deref())
        .map_err(|e| format!("查询历史失败: {}", e))?;
    Ok(records
        .into_iter()
        .map(|r| HistoryRecord {
            id: r.id,
            timestamp: r.timestamp,
            action: r.action,
            submodule_name: r.submodule_name,
            detail: r.detail,
            success: r.success,
        })
        .collect())
}

#[derive(serde::Serialize)]
struct ScanResult {
    submodules: Vec<SubmoduleInfo>,
    aggregate: AggregateInfo,
}

#[derive(serde::Serialize)]
struct AggregateInfo {
    total: usize,
    clean: usize,
    ahead_of_parent: usize,
    behind_remote: usize,
    detached: usize,
    dirty: usize,
    orphaned: usize,
    uninitialized: usize,
}

#[derive(serde::Serialize)]
struct SubmoduleInfo {
    name: String,
    path: String,
    url: String,
    tracked_branch: String,
    parent_pointer: String,
    local_head: String,
    remote_head: String,
    status: String,
    ahead_count: usize,
    behind_count: usize,
    remote_unreachable: bool,
}

#[derive(serde::Serialize)]
struct HistoryRecord {
    id: i64,
    timestamp: String,
    action: String,
    submodule_name: String,
    detail: String,
    success: bool,
}

#[derive(serde::Serialize)]
struct IssueInfo {
    submodule_name: String,
    status: String,
    description: String,
    suggested_action: String,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_repo,
            status,
            sync_to_parent,
            sync_all_to_parent,
            retire_submodule,
            list_history,
            export_ci,
        ])
        .run(tauri::generate_context!())
        .expect("启动 KSE 失败");
}
