# TODO

## 已完成

### Iteration 0：项目脚手架 ✅

- [x] **0.1** 初始化 Rust 项目，配置依赖
  - [x] `cargo init`（项目根目录）
  - [x] 添加 `git2`（vendored）、`clap`（derive）、`serde`（derive）依赖
  - [ ] ~~验证 `cargo build` 通过~~（需本地 Rust 工具链）
- [x] **0.2** 基础设施文件
  - [x] 添加 `.gitignore`（含 `/target`、`.DS_Store`、`*.swp`、`.vscode/`、`.idea/`）
  - [x] 添加 `rustfmt.toml`（max_width=100, tab_spaces=4, edition=2021）
  - [ ] ~~验证 `cargo fmt --check` 通过~~（需本地 Rust 工具链）
- [x] **0.3** 搭建目录结构
  - [x] 创建 `src/model/mod.rs`
  - [x] 创建 `src/commands/mod.rs`
- [x] **0.4** 配置 CI
  - [x] 创建 `.github/workflows/ci.yml`
  - [x] CI 步骤：checkout → setup Rust → cargo check → cargo test → cargo clippy
  - [ ] ~~验证 CI 通过~~（需 push 触发）
- [x] **0.5** 空 CLI → 后续扩展为完整 CLI
  - [x] clap 定义 `kse` CLI
  - [ ] ~~验证 `cargo run -- --help`~~（需本地 Rust 工具链）

### Iteration 1：核心模型与 CLI 原型 ✅

- [x] 定义 `CommitHash(String)` 新类型，实现 `Display`（截断 7 位）+ `Default`
- [x] 定义 `SubmoduleStatus` 枚举（7 种变体 + `priority()`）
- [x] 定义 `Submodule` 结构体
- [x] 定义 `HealthIssue` 结构体
- [x] 定义 `RepoState` 结构体 + `RepoState::scan(&Path)` 三路 commit 比对
- [x] 实现 `SubmoduleEditor` trait
- [x] 实现 `UpdateStrategy` 枚举（FastForward / Rebase / Merge）
- [x] `kse health-check [path]` — 扫描并表格输出
- [x] 错误处理：路径不存在 / 非 Git 仓库
- [x] 6 个单元测试（模型层 + 集成测试 fixture）

### Iteration 2：原子操作命令集 ✅

- [x] `add_submodule` — `git submodule add` 封装 + 路径冲突检测
- [x] `init_all` — 批量初始化
- [x] `update_single` — 支持 FastForward / Rebase / Merge
- [x] `update_all` — 批量更新（容错：单个失败继续）
- [x] `sync_to_parent` — 更新父仓库 commit 指针
- [x] `sync_all_to_parent` — 批量同步
- [x] `checkout_branch` — 切换到指定分支
- [x] `create_branch` — 创建并切换到新分支
- [x] `retire_submodule` — `git submodule deinit` + 移除 `.gitmodules` 条目 + 记录退役信息
- [x] 错误处理：本地有未提交修改时阻止更新
- [x] 重复添加检测：同名/同路径校验
- [x] 集成测试：7 个 `#[ignore]` 测试（临时仓库 + 子模块 fixture）
- [x] `checkout_all` / `branch_all` — 批量切换/创建分支（trait + CLI + Tauri）
- [x] UI 多选（复选框 + Select All）+ 选中执行 + 进度显示
- [x] UI dry-run 预览弹窗（模态框，执行前展示操作计划）
- [x] 撤销指引（reflog 文档 → README）

### Iteration 3：Tauri 外壳与状态驱动 UI ✅

- [x] `src-tauri/Cargo.toml` + `tauri.conf.json` + `build.rs`
- [x] 7 个 Tauri command 绑定（scan_repo / health_check / init_all / update_single / update_all / sync_to_parent / sync_all_to_parent / retire_submodule / list_history / export_ci）
- [x] `src/lib.rs` 共享库 + CLI/Tauri 共用 `kse_core`
- [x] Web UI：侧边栏（仓库路径 + 统计 + 批量操作 + 导出 CI + 历史）
- [x] Web UI：子模块列表表格（状态颜色圆点 + 操作按钮）
- [x] Web UI：详情面板（三列 commit 对比 + diff 差异数 + 状态引导 + 建议操作）
- [x] Web UI：健康问题横幅
- [x] Web UI：`--dry-run` 导出 CI 按钮（复制到剪贴板）
- [x] 响应式 flex 布局

### Iteration 4：操作历史与异常处理 ✅

- [x] SQLite schema：`operations` 表 + `retired_submodules` 表
- [x] `rusqlite`（bundled）数据库初始化（`.git/kse/history.db`）
- [x] 每次原子操作自动写入 `operations` 表
- [x] `kse history` CLI 命令（`--limit` / `--submodule`）
- [x] Web UI 侧边栏历史面板
- [x] Detached / Dirty / Orphaned 健康检测 + 修复引导
- [x] retire 自动记录退役信息到 `retired_submodules` 表

### Iteration 5：分批灰度与打包分发 ✅

- [x] 全局 `--dry-run` 参数，所有命令支持预览模式
- [x] `commands/export.rs` 模块：shell / GitHub Actions / GitLab CI 脚本生成
- [x] `kse export-ci` CLI 命令（`--format` / `--output`）
- [x] `export_ci` Tauri command + Web UI 按钮
- [x] Tauri 跨平台打包配置（macOS .dmg / Linux .AppImage / Windows .msi）
- [x] README 用户手册
- [x] CHANGELOG.md
- [x] 版本号更新到 v1.0.0

---

## 待完成（需本地环境）

| 任务 | 迭代 | 命令 |
|------|------|------|
| 本地编译验证 | 全部 | `cargo build && cargo test && cargo clippy -- -D warnings` |
| CI 触发验证 | 0.4 | `git push origin main` |
| Tauri 桌面应用启动 | 3.0 | `cargo tauri dev` |
| Tauri 跨平台打包 | 5.4 | `cargo tauri build` |
| GitHub Release | 5.6 | 创建 GitHub Release + 上传安装包 |

## 待实现

| 任务 | 原因 | 优先级 |
|------|------|--------|
| 4.5 按时间范围筛选历史 | 需要前端日期选择器 | 低 |
| 2.1 URL 可达性验证 | 需要网络请求 | 低 |
| 4.4 完整 Orphaned 远程检测 | 需要 fetch 操作 | 低 |
