# TODO

## Iteration 0：项目脚手架

- [x] **0.1** 初始化 Rust 项目，配置依赖
  - [x] `cargo init`（项目根目录）
  - [x] 添加 `git2`（vendored）、`clap`（derive）、`serde`（derive）依赖
  - [ ] 验证 `cargo build` 通过（需本地 Rust 工具链）
- [x] **0.2** 基础设施文件
  - [x] 添加 `.gitignore`（含 `/target`、`.DS_Store`、`*.swp`、`.vscode/`、`.idea/`）
  - [x] 添加 `rustfmt.toml`（max_width=100, tab_spaces=4, edition=2021）
  - [ ] 验证 `cargo fmt --check` 通过（需本地 Rust 工具链）
- [x] **0.3** 搭建目录结构
  - [x] 创建 `src/model/mod.rs`
  - [x] 创建 `src/commands/mod.rs`
- [x] **0.4** 配置 CI
  - [x] 创建 `.github/workflows/ci.yml`
  - [x] CI 步骤：checkout → setup Rust → cargo check → cargo test → cargo clippy
  - [ ] 验证 CI 通过（触发一次 push）
- [x] **0.5** 空 CLI
  - [x] clap 定义 `kse` CLI，至少包含 `health-check` 子命令占位
  - [ ] 验证 `cargo run -- --help` 输出帮助信息（需本地 Rust 工具链）

**验证**：`cargo build && cargo test && cargo clippy -- -D warnings` 全部通过

---

## Iteration 1：核心模型与 CLI 原型

### 1.1 模型定义

- [x] 定义 `CommitHash(String)` 新类型，实现 `Display`（截断 7 位）
- [x] 定义 `SubmoduleStatus` 枚举（7 种变体）
  - [x] Clean / AheadOfParent / BehindRemote / Detached / Dirty / Orphaned / Uninitialized
  - [x] 实现 `priority()` 方法（Dirty=0 > Orphaned=1 > Detached=2 > Uninitialized=3 > BehindRemote=4 > AheadOfParent=5 > Clean=6）
- [x] 定义 `Submodule` 结构体
  - [x] 字段：name / path / url / tracked_branch / parent_pointer / local_head / remote_head / status
- [x] 定义 `HealthIssue` 结构体（submodule_name / status / description / suggested_action）

### 1.2 RepoState

- [x] 定义 `RepoState` 结构体（root_path / submodules / total / clean_count / needs_attention）
- [x] 实现 `RepoState::scan(&Path)` 方法
  - [x] 检测 `.gitmodules` 是否存在，不存在则返回空状态
  - [x] `git2::Repository::open` 打开仓库
  - [x] `repo.submodules()` 获取子模块列表
  - [x] 对每个子模块读取：name / path / url / branch / head_id / status
  - [x] 根据 `SubmoduleStatus` flags 判定状态（优先：Uninitialized > Dirty > Clean）
  - [x] 填充 `total`、`clean_count`、`needs_attention` 统计
- [x] 实现 `SubmoduleEditor` trait（所有原子操作接口）
- [x] 实现 `UpdateStrategy` 枚举（FastForward / Rebase / Merge）

### 1.3 CLI 命令

- [x] `kse health-check [path]` — 扫描并表格输出
  - [x] 路径默认为 `.`
  - [x] 输出仓库路径、子模块总数、干净数、需关注列表
  - [x] 逐行输出：名称 / 状态 / 跟踪分支
  - [x] 无子模块时输出"没有子模块"提示
- [x] 错误处理：路径不存在 → 友好提示 + exit(1)
- [x] 错误处理：非 Git 仓库 → 友好提示 + exit(1)

### 1.4 单元测试

**模型层测试**（不依赖 Git 仓库）：

- [x] `test_status_priority_ordering` — 验证优先级排序正确
- [x] `test_clean_is_lowest_priority` — Clean 优先级最低
- [x] `test_commit_hash_display_truncates` — Display 截断 7 位
- [x] `test_commit_hash_equality` — PartialEq 正确

**验证**：`cargo test` 全部通过

---

## Iteration 2：原子操作命令集

### 2.1 add_submodule

- [x] `git submodule add <url> <path>` 封装
- [ ] 验证 URL 可达性（待完善）
- [x] 处理路径冲突（已存在）
- [ ] 处理重复添加（待完善）

### 2.2 init / update

- [x] `init_all` — 批量初始化所有未初始化子模块
- [x] `update_single` 支持 FastForward 策略
- [x] `update_single` 支持 Rebase 策略
- [x] `update_single` 支持 Merge 策略
- [x] 错误处理：本地有未提交修改时阻止更新

### 2.3 sync_to_parent

- [x] 提交子模块目录的修改
- [x] 更新父仓库 commit 指针
- [x] `sync_all_to_parent` 批量同步
- [ ] 跳过已 Clean 的子模块（待优化）

### 2.4 retire_submodule

- [x] `git submodule deinit <name>`
- [x] 从 `.gitmodules` 移除条目
- [ ] 记录退役信息（时间、原因）→ 预留 SQLite 接口

### 2.5 checkout / create branch

- [x] `checkout_branch` — 切换到指定分支
- [x] `create_branch` — 创建并切换到新分支
- [ ] 批量操作支持（待完善）

### 2.6 集成测试

- [ ] 创建临时 Git 仓库 + 子模块（使用 `git2` 或 `tempfile`）
- [ ] 模拟多子模块多状态场景
- [ ] 测试全流程：add → update → sync → retire
- [ ] 验证父仓库指针正确更新
- [ ] 清理临时目录

**验证**：`cargo test -- --include-ignored`（集成测试标记为 `#[ignore]`）

---

## Iteration 3：Tauri 外壳与状态驱动 UI

### 3.1 初始化 Tauri

- [x] 创建 `src-tauri/Cargo.toml`（tauri 依赖 + kse_core 引用）
- [x] 配置 `tauri.conf.json`（窗口标题、尺寸、标识符）
- [x] 将 core crate 作为路径依赖引入
- [ ] 验证 `cargo tauri dev` 启动（需本地 Tauri CLI）

### 3.2 后端命令绑定

- [x] `health_check` + `scan_repo` 暴露为 Tauri command
- [x] `update_single` / `update_all` 暴露为 command
- [x] `sync_to_parent` / `sync_all_to_parent` 暴露为 command
- [x] `init_all` / `retire_submodule` 暴露为 command
- [x] 统一错误处理：Rust Error → String

### 3.3 UI 布局

- [x] 侧边栏：仓库路径显示 + 刷新按钮 + 批量操作
- [x] 主表格：名称 / 状态（颜色圆点） / 分支 / 操作按钮
- [x] 状态颜色映射：绿色(Clean) / 黄色(Ahead, Behind) / 红色(Dirty, Detached, Orphaned) / 灰色(Uninitialized)
- [x] 响应式布局（flex 自适应）

### 3.4 详情面板

- [x] 选中子模块展示详情
- [x] 三个 commit 对比列（parent_pointer / local_head / remote_head）
- [x] 状态说明文本 + 建议操作按钮
- [ ] 显示与远程的 commit 差异数（待完善）

### 3.5 批量操作

- [x] "全部更新"按钮
- [x] "全部同步"按钮
- [ ] 操作进度提示（待完善）

### 3.6 代码拆分

- [x] 创建 `src/lib.rs` 作为共享库
- [x] `src/main.rs` 保持不变（CLI 二进制）
- [x] `src-tauri/` 作为独立二进制，依赖 `kse_core`

**验证**：`cargo tauri build` 通过（需本地 Tauri CLI）

---

## Iteration 4：操作历史与异常处理

### 4.1 SQLite schema

- [x] 设计 `operations` 表（id / time / action / submodule_name / detail / success）
- [x] 设计 `retired_submodules` 表（name / url / path / retired_at / reason）
- [x] 使用 `rusqlite`（bundled）实现数据库初始化

### 4.2 操作历史记录

- [x] 每个原子操作后写入 `operations` 表（通过 `GitSubmoduleEditor` 自动记录）
- [x] `kse history` CLI 命令列出最近操作（支持 `--limit`、`--submodule`）
- [x] UI 操作历史面板

### 4.3 异常状态修复引导

- [x] Detached：检测游离 HEAD → 状态标记 → 建议 checkout（已在前端 health_check 实现）
- [x] Dirty：检测未提交修改 → 阻止更新 → 引导提交或 stash
- [ ] UI 中修复按钮联动操作（待完善）

### 4.4 Orphaned 告警

- [ ] 检测 parent_pointer 在远程已不存在（待实现完整远程检测）
- [x] 红色高亮标记 + 告警提示（前端 health_check issue 面板）
- [x] 建议操作引导（手动干预）

### 4.5 操作历史 UI

- [x] 历史记录显示在侧边栏（时间 / 操作 / 子模块 / 结果）
- [ ] 按时间范围筛选（待完善）
- [x] 按子模块名称筛选（CLI 支持）
- [ ] 撤销指引（提示用户使用 `git reflog`）（待完善）

**验证**：`cargo test` + UI 手动测试

---

## Iteration 5：分批灰度与打包分发

### 5.1 批量选择 + 分批执行

- [ ] 按依赖拓扑排序子模块（待实现）
- [ ] UI 多选 + 全选（待实现）
- [ ] 分批执行 + 进度显示（待实现）

### 5.2 dry-run 预览

- [x] `--dry-run` flag：全局参数，所有命令支持预览
- [x] CLI 模式预览：打印将要执行的操作
- [ ] UI 模式预览弹窗（待实现）

### 5.3 导出 CI 脚本

- [x] 导出为 shell 脚本（`export-ci.sh` 格式）
- [x] 导出为 GitHub Actions YAML 片段
- [x] 导出为 GitLab CI YAML 片段
- [x] CLI `kse export-ci` 命令
- [x] Tauri `export_ci` 命令
- [x] Web UI 侧边栏 "导出 CI" 按钮（复制到剪贴板）

### 5.4 跨平台打包

- [x] Tauri 打包配置：macOS .dmg（tauri.conf.json 已配置）
- [x] Tauri 打包配置：Linux .AppImage
- [x] Tauri 打包配置：Windows .msi
- [ ] 验证各平台安装包可安装运行（需本地 Tauri CLI）

### 5.5 用户文档

- [x] CLI `--help` 完整输出（clap 自动生成）
- [x] README 用户手册（安装 / 使用 / 配置）
- [x] CHANGELOG.md

### 5.6 端到端测试 + 发布

- [ ] 真实仓库场景测试（需本地 Rust 工具链）
- [ ] 边界场景：空仓库、无子模块、超大子模块
- [x] 更新版本号 → v1.0.0
- [ ] GitHub Release

**验证**：全链路 E2E 测试通过

---

## 已完成记录

<!-- 完成后将复选框移至此处，并注明完成日期 -->
