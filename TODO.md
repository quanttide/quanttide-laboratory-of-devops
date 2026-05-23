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

- [ ] `git submodule add <url> <path>` 封装
- [ ] 验证 URL 可达性
- [ ] 处理路径冲突（已存在）
- [ ] 处理重复添加

### 2.2 init / update

- [ ] `init_all` — 批量初始化所有未初始化子模块
- [ ] `update_single` 支持 FastForward 策略
- [ ] `update_single` 支持 Rebase 策略
- [ ] `update_single` 支持 Merge 策略
- [ ] 错误处理：本地有未提交修改时阻止更新

### 2.3 sync_to_parent

- [ ] 提交子模块目录的修改
- [ ] 更新父仓库 commit 指针
- [ ] `sync_all_to_parent` 批量同步
- [ ] 跳过已 Clean 的子模块

### 2.4 retire_submodule

- [ ] `git submodule deinit <name>`
- [ ] 从 `.gitmodules` 移除条目
- [ ] 记录退役信息（时间、原因）→ 预留 SQLite 接口

### 2.5 checkout / create branch

- [ ] `checkout_branch` — 切换到指定分支
- [ ] `create_branch` — 创建并切换到新分支
- [ ] 批量操作支持

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

- [ ] `cargo tauri init` 在 workspace 中创建 `src-tauri/`
- [ ] 配置 `tauri.conf.json`（窗口标题、尺寸、标识符）
- [ ] 将 core crate 作为 workspace 依赖引入
- [ ] 验证 `cargo tauri dev` 启动

### 3.2 后端命令绑定

- [ ] `health_check` 暴露为 Tauri command
- [ ] `update_single` / `update_all` 暴露为 command
- [ ] `sync_to_parent` 暴露为 command
- [ ] 统一错误处理：Rust Error → 前端字符串

### 3.3 UI 布局

- [ ] 侧边栏：仓库路径显示 + 刷新按钮
- [ ] 主表格：名称 / 状态（颜色圆点） / 分支 / 操作按钮
- [ ] 状态颜色映射：绿色(Clean) / 黄色(Ahead, Behind) / 红色(Dirty, Detached, Orphaned) / 灰色(Uninitialized)
- [ ] 响应式布局

### 3.4 详情面板

- [ ] 选中子模块展示详情
- [ ] 三个 commit 对比列（parent_pointer / local_head / remote_head）
- [ ] 状态说明文本 + 建议操作按钮
- [ ] 显示与远程的 commit 差异数

### 3.5 批量操作

- [ ] "全部更新"按钮
- [ ] "全部同步"按钮
- [ ] 操作进度条或 loading 指示器

**验证**：`cargo tauri build` 通过

---

## Iteration 4：操作历史与异常处理

### 4.1 SQLite schema

- [ ] 设计 `operations` 表（id / time / action / submodule_name / detail / success）
- [ ] 设计 `retired_submodules` 表（name / url / path / retired_at / reason / retired_by）
- [ ] 使用 `rusqlite` 或 `sqlx` 实现数据库初始化

### 4.2 操作历史记录

- [ ] 每个原子操作后写入 `operations` 表
- [ ] `kse history` CLI 命令列出最近操作（支持 `--limit`、`--submodule`）
- [ ] UI 操作历史面板

### 4.3 异常状态修复引导

- [ ] Detached：检测游离 HEAD → 建议 checkout 到跟踪分支 → "修复"按钮
- [ ] Dirty：显示未提交文件列表 → 引导提交或 stash → "修复"按钮
- [ ] UI 中修复按钮联动操作

### 4.4 Orphaned 告警

- [ ] 检测 parent_pointer 在远程已不存在
- [ ] 红色高亮标记 + 告警提示
- [ ] 建议操作引导（手动干预）

### 4.5 操作历史 UI

- [ ] 历史列表展示（时间 / 操作 / 子模块 / 结果）
- [ ] 按时间范围筛选
- [ ] 按子模块名称筛选
- [ ] 撤销指引（提示用户使用 `git reflog`）

**验证**：`cargo test` + UI 手动测试

---

## Iteration 5：分批灰度与打包分发

### 5.1 批量选择 + 分批执行

- [ ] 按依赖拓扑排序子模块
- [ ] UI 多选 + 全选
- [ ] 分批执行 + 进度显示

### 5.2 dry-run 预览

- [ ] `--dry-run` flag：仅输出计划，不执行
- [ ] CLI 模式预览列表
- [ ] UI 模式预览弹窗

### 5.3 导出 CI 脚本

- [ ] 导出为 shell 脚本（`export-ci.sh`）
- [ ] 导出为 GitHub Actions YAML 片段
- [ ] 导出为 GitLab CI YAML 片段

### 5.4 跨平台打包

- [ ] Tauri 打包配置：macOS .dmg
- [ ] Tauri 打包配置：Linux .AppImage
- [ ] Tauri 打包配置：Windows .msi
- [ ] 验证各平台安装包可安装运行

### 5.5 用户文档

- [ ] CLI `--help` 完整输出
- [ ] README 用户手册（安装 / 使用 / 配置 / FAQ）
- [ ] CHANGELOG.md

### 5.6 端到端测试 + 发布

- [ ] 真实仓库场景测试（例如用本仓库）
- [ ] 边界场景：空仓库、无子模块、超大子模块
- [ ] 更新版本号 → v1.0.0
- [ ] GitHub Release

**验证**：全链路 E2E 测试通过

---

## 已完成记录

<!-- 完成后将复选框移至此处，并注明完成日期 -->
