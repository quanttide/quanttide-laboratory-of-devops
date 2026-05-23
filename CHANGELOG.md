# Changelog

## [2.0.0] — 2026-05-24

### 重命名

- CLI 从 `kse` 重命名为 `qtcloud-devops code`
- 所有命令从 `kse <cmd>` 变为 `qtcloud-devops code <cmd>`
- 二进制文件从 `kse` 改为 `qtcloud-devops`

### 破坏性变更

- **移除 8 个纯 git 包装命令**：`add`、`init`、`update`、`update-all`、`checkout`、`branch`、`checkout-all`、`branch-all`
  - 这些命令本质上是 shell 命令的 Rust 翻译版，没有建模层面的新贡献
  - 改用原生 git：`git submodule add`、`git submodule update --init`、`git checkout` 等
- **`kse health-check` 重命名为 `kse status`**：心理模型对齐 `git status`
- **`kse sync` 重设计为 `kse sync parent`**

### 保留的核心贡献

- `kse status`（原 health-check）— 三路 commit 比对 + 7 种状态分类
- `kse sync parent`（原 sync）— 子模块 → 父仓库指针同步的原子操作
- `kse sync platform` — 新增跨环境版本对齐（CI 场景）
- `kse retire` — 子模块自动反注册
- `kse history` — SQLite 操作历史
- `kse export-ci` — CI 脚本导出

## [1.0.0] — 2026-05-23

### 新增
- `kse health-check` — 扫描子模块状态（7 种状态判定）
- `kse add` — 添加子模块
- `kse init` — 初始化未初始化的子模块
- `kse update` / `kse update-all` — 更新子模块（支持 FastForward / Rebase / Merge）
- `kse sync` / `kse sync-all` — 同步子模块指针到父仓库
- `kse checkout` / `kse branch` — 切换/创建子模块分支
- `kse retire` — 退役子模块
- `kse history` — 查看操作历史（SQLite 持久化）
- `kse export-ci` — 导出 CI 脚本（shell / GitHub Actions / GitLab CI）
- `--dry-run` 全局预览模式
- Tauri 桌面应用：Web UI 仪表盘（子模块列表 + 详情面板 + 操作历史）
- 健康问题检测与建议引导

### 基础设施
- Rust + git2 + clap 命令行工具
- Tauri v1 跨平台桌面壳
- GitHub Actions CI（cargo check + test + clippy）
- SQLite 操作历史数据库（`.git/kse/history.db`）
- 模型层单元测试覆盖状态优先级与 CommitHash
