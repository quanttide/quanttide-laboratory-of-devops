# TODO

## 已完成

### Iteration 0-5

全部完成，详见 git log `6d388be..47a0dd2`。

关键交付：
- `kse` CLI — 14 个子命令，`--dry-run` 全局预览
- Tauri 桌面应用 — 12 个后端命令 + 完整 Web UI
- SQLite 操作历史 — `.git/kse/history.db`
- CI 导出 — shell / GitHub Actions / GitLab CI
- 76 个测试（44 unit + 32 integration）

---

## Iteration 6：规范合规补齐

### 6.1 Orphaned 检测逻辑

- [x] **6.1.1** 实现 `is_orphaned()` — merge_base 判定（inline in `scan()`）
- [x] **6.1.2** 插入判定分支 — Dirty > Orphaned > Detached
- [x] **6.1.3** 集成测试 — `test_scan_remote_unreachable` + 优先级已覆盖

### 6.2 离线场景处理

- [x] **6.2.1** `Submodule.remote_unreachable: bool`
  - [x] 更新结构体定义 + `SubmoduleInfo` Tauri 结构体
  - [x] `RepoState::scan()` 中 `find_reference` 失败时标记 `true`
- [x] **6.2.2** 远程不可达时判定降级
  - [x] 跳过 Orphaned 判定分支（`!remote_unreachable`）
  - [x] 跳过 BehindRemote 判定分支
  - [x] `ahead_count` / `behind_count` 置 0
- [x] **6.2.3** UI 层展示
  - [x] 状态列显示 🛰 标记（`statusIcon`）
  - [x] 详情面板显示"远程仓库不可达"横幅

### 6.3 AggregateStatus + health_check

- [x] **6.3.1** `AggregateStatus` 结构体 — `total` + 7 种状态计数 + `Default` + `from_submodules()`
- [x] **6.3.2** `scan_all()` — 委托 `RepoState::scan()` + 聚合
- [x] **6.3.3** `health_check()` — 过滤非 Clean + `describe_issue()`（已有）
- [x] **6.3.4** CLI/Tauri/UI 聚合输出
  - [x] `kse health-check` 输出聚合统计
  - [x] Tauri `scan_repo` 返回 `ScanResult { submodules, aggregate }`
  - [x] Web UI 概览区域 + 聚合计数
- [x] 集成测试 — `test_aggregate_status_from_scan`

---

## 待完成（需本地环境）

| 任务 | 命令 | 状态 |
|------|------|:----:|
| 本地编译验证 | `cargo build && cargo test && cargo clippy -- -D warnings` | ✅ build+test 通过 |
| Tauri 系统依赖安装 | `sudo apt install libsoup2.4-dev libwebkit2gtk-4.0-dev` | ❌ 需 root |
| Tauri 命令桥接依赖修复 | `package = "kse"` 添加到 src-tauri/Cargo.toml | ✅ 已提交 |
| CI 触发验证 | `git push origin main` | ⏳ |
| Tauri 桌面应用启动 | `cargo tauri dev` | ❌ 卡系统依赖 |
| Tauri 跨平台打包 | `cargo tauri build` | ❌ |
| GitHub Release | 创建 GitHub Release + 上传安装包 | ❌ |

## Iteration 8（已完成）

### 8.1 本地编译验证

- [x] `cargo build --release` — 通过
- [x] `cargo test` — 47 个单元测试全部通过
- [x] `cargo test -- --include-ignored` — 6 个集成测试通过，30 个需要裸仓库 fixture

### 8.2 Tauri 命令桥接

- [x] 确认 `src-tauri/src/main.rs` 已有全部 12 个 Tauri 命令（`scan_repo`, `health_check`, `init_all`, `update_single`, `update_all`, `sync_to_parent`, `sync_all_to_parent`, `retire_submodule`, `checkout_all`, `branch_all`, `export_ci`, `list_history`）
- [x] 修复依赖声明：`kse_core = { path = "..", package = "kse" }`
- [x] 系统依赖问题：需 `sudo apt install libsoup2.4-dev libwebkit2gtk-4.0-dev libgtk-3-dev`
- [x] 提交 `c84ac97 fix: add package = "kse" to kse_core dependency in src-tauri Cargo.toml`

## Iteration 8：PyO3 集成（已完成）

### 8.1 crate 结构调整

- [x] `[lib] crate-type = ["lib", "cdylib"]` — lib 同时输出 `.rlib` 和 `.so`
- [x] `pyo3` 依赖（optional feature `python`）
- [x] `serde_json = "1"` 依赖

### 8.2 Python 绑定

- [x] `src/python.rs` — 新建 `#[pymodule] kse_core`，导出 `scan_repo(path) -> dict`
- [x] 条件编译 `#[cfg(feature = "python")]`
- [x] 模型类型加 `#[derive(serde::Serialize)]`（CommitHash, SubmoduleStatus, Submodule, RepoState, AggregateStatus）
- [x] `cargo build --features python` 编译通过

### 8.3 待完成

- [ ] PyO3 端到端调用验证（Python import kse_core 并调用 scan_repo）
- [ ] maturin 包管理（创建 `pyproject.toml`）
- [ ] 导出更多函数（checkout_all, sync_all, init_all, health_check）
- [ ] 创建 `qtcloud-devops-cli` Python 包入口

### 7.1 用户指南文档

- [x] 撰写完整用户指南 `docs/user-guide.md`（555 行）
- [x] 覆盖所有 14 个 CLI 命令详解
- [x] Web UI 使用指南（ASCII 布局图 + 功能说明）
- [x] CI 集成示例（GitHub Actions）
- [x] 常见场景工作流（日常开发/批量操作/清理）
- [x] 故障排除表

### 7.2 文档评审与修复

- [x] 修复 8 处 `--repo`/`--path` CLI 示例错误（应改为位置参数）
- [x] 新增项目简介和 Table of Contents
- [x] 新增系统要求表格
- [x] 新增卸载指南
- [x] `git reset --hard` 撤销操作风险提示

### 7.3 实际验证

- [x] 在 quanttide 主仓库（17 个子模块）上全流程验证
- [x] `health-check` → 发现 7 个 Detached、2 个 Dirty、2 个 Orphaned
- [x] `checkout-all main` → 修复 7 个游离 HEAD
- [x] `sync-all` → 修复 2 个 Dirty
- [x] `git push` → 修复 2 个 Orphaned（本地 commit 未推送到远程）
- [x] 最终状态：17/17 Clean ✅

## 未来规划

### Tauri 桌面应用

当前已存在 `src-tauri/` 外壳和 `web-ui/` 骨架，但尚未接入后端命令。需完成：

| 阶段 | 内容 |
|------|------|
| Tauri 命令桥接 | 将 lib.rs 中的 12 个原子操作逐一注册为 Tauri 命令 |
| Web UI 渲染 | 实现 dev.md 中的布局蓝图 |
| 状态联动 | 勾选 → 批量操作 → 预览 → 执行 → 自动刷新 |
| 离线降级 UI | 远程不可达时显示 🛰 标记 |

### CLI 增强

| 功能 | 说明 |
|------|------|
| 勾选批量操作 | 支持 `kse update lib-a,lib-b` 逗号分隔 |
| 子模块 push 提醒 | sync 后提示用户 push 子模块 |
| `kse push-all` | 自动 push 所有 AheadOfParent 子模块 |
