#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kse_core::commands::editor::GitSubmoduleEditor;
use kse_core::commands::{SubmoduleEditor, UpdateStrategy};
use kse_core::model::RepoState;
use std::path::PathBuf;

#[tauri::command]
fn scan_repo(path: String) -> Result<Vec<SubmoduleInfo>, String> {
    let root = PathBuf::from(&path);
    let state =
        RepoState::scan(&root).map_err(|e| format!("扫描失败: {}", e))?;

    let infos: Vec<SubmoduleInfo> = state
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
        })
        .collect();

    Ok(infos)
}

#[tauri::command]
fn health_check(path: String) -> Result<Vec<IssueInfo>, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    let issues = editor.health_check().map_err(|e| format!("健康检查失败: {}", e))?;
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
fn init_all(path: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    editor.init_all().map_err(|e| format!("初始化失败: {}", e))?;
    Ok("已初始化所有子模块".into())
}

#[tauri::command]
fn update_single(repo: String, name: String, strategy: String) -> Result<String, String> {
    let root = PathBuf::from(&repo);
    let strategy = parse_strategy(&strategy)?;
    let editor = GitSubmoduleEditor::new(root);
    editor
        .update_single(&name, strategy)
        .map_err(|e| format!("更新失败: {}", e))?;
    Ok(format!("已更新子模块 '{}'", name))
}

#[tauri::command]
fn update_all(path: String, strategy: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    let strategy = parse_strategy(&strategy)?;
    let editor = GitSubmoduleEditor::new(root);
    editor
        .update_all(strategy)
        .map_err(|e| format!("批量更新失败: {}", e))?;
    Ok("已更新所有子模块".into())
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
fn list_history(path: String, limit: usize, submodule: Option<String>) -> Result<Vec<HistoryRecord>, String> {
    let root = PathBuf::from(&path);
    let editor = GitSubmoduleEditor::new(root);
    let records = editor
        .list_history(limit, submodule.as_deref())
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

fn parse_strategy(s: &str) -> Result<UpdateStrategy, String> {
    match s.to_lowercase().replace("-", "_").as_str() {
        "fastforward" | "ff" | "fast_forward" => Ok(UpdateStrategy::FastForward),
        "rebase" => Ok(UpdateStrategy::Rebase),
        "merge" => Ok(UpdateStrategy::Merge),
        _ => Err(format!(
            "未知策略 '{}'，可选: fast-forward, rebase, merge",
            s
        )),
    }
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
            health_check,
            init_all,
            update_single,
            update_all,
            sync_to_parent,
            sync_all_to_parent,
            retire_submodule,
            list_history,
        ])
        .run(tauri::generate_context!())
        .expect("启动 KSE 失败");
}
