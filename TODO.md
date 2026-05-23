# TODO

## Iteration 1：核心模型与 CLI 原型

- [ ] **1.1** 初始化 Rust 项目，配置 git2 依赖
  - [ ] `cargo init submodule-editor-core`
  - [ ] 添加 `git2`、`serde`、`clap` 依赖
  - [ ] 搭建目录结构：`src/model/`、`src/commands/`
- [ ] **1.2** 实现 `Submodule` 结构体和 `SubmoduleStatus` 枚举
  - [ ] 定义 `CommitHash` 新类型
  - [ ] 定义 `SubmoduleStatus`（Clean / AheadOfParent / BehindRemote / Detached / Dirty / Orphaned / Uninitialized）
  - [ ] 定义 `Submodule` 结构体（name / path / url / tracked_branch / parent_pointer / local_head / remote_head / status）
- [ ] **1.3** 实现 `RepoState` 扫描逻辑
  - [ ] 解析 `.gitmodules` 获取子模块列表
  - [ ] 读取父仓库中记录的 commit（`git submodule status`）
  - [ ] 读取子模块本地 HEAD
  - [ ] 读取远程跟踪分支最新 commit
  - [ ] 比对三者判定每个子模块的状态
  - [ ] 处理未初始化子模块
- [ ] **1.4** 实现 `health_check()` CLI 命令
  - [ ] 用 clap 注册 `kse health-check` 子命令
  - [ ] 输出表格（名称 / 状态 / 分支 / 建议操作）
  - [ ] 状态列用颜色标识
- [ ] **1.5** 单元测试覆盖所有状态判定逻辑
  - [ ] 测试 Clean（三方一致）
  - [ ] 测试 AheadOfParent（本地领先）
  - [ ] 测试 BehindRemote（远程领先）
  - [ ] 测试 Detached（游离 HEAD）
  - [ ] 测试 Dirty（有未提交修改）
  - [ ] 测试 Orphaned（parent_pointer 远程已不存在）

## Iteration 2：原子操作命令集

- [ ] **2.1** 实现 `add_submodule`
  - [ ] `git submodule add <url> <path>`
  - [ ] 验证 URL 可访问
  - [ ] 处理路径已存在等冲突
- [ ] **2.2** 实现 `init_all` / `update_single`
  - [ ] `update_single` 支持 FastForward 策略
  - [ ] `update_single` 支持 Rebase 策略
  - [ ] `update_single` 支持 Merge 策略
  - [ ] `init_all` 批量初始化所有子模块
  - [ ] 错误处理：本地有未提交修改时阻止更新
- [ ] **2.3** 实现 `sync_to_parent`
  - [ ] 提交子模块目录的修改
  - [ ] 更新父仓库中记录的 commit 指针
  - [ ] `sync_all_to_parent` 批量同步
- [ ] **2.4** 实现 `retire_submodule`
  - [ ] `git submodule deinit <name>`
  - [ ] 从 `.gitmodules` 移除
  - [ ] 记录退役信息（时间、原因）
- [ ] **2.5** 实现 `checkout_branch` / `create_branch`
  - [ ] 切换到指定分支（`git checkout`）
  - [ ] 创建新分支（`git checkout -b`）
  - [ ] 支持批量操作
- [ ] **2.6** 集成测试
  - [ ] 创建临时仓库 + 子模块
  - [ ] 模拟多子模块场景
  - [ ] 测试全流程：add → update → sync → retire
  - [ ] 验证父仓库指针正确更新

## Iteration 3：Tauri 外壳与状态驱动 UI

- [ ] **3.1** 初始化 Tauri 项目
  - [ ] `cargo tauri init`
  - [ ] 配置 `tauri.conf.json`
  - [ ] 将 core crate 作为 workspace 依赖引入
- [ ] **3.2** 实现后端命令绑定
  - [ ] 将 `health_check` 暴露为 Tauri command
  - [ ] 将 `update_single` / `update_all` 暴露为 command
  - [ ] 将 `sync_to_parent` 暴露为 command
  - [ ] 错误处理与前端反馈
- [ ] **3.3** UI 侧边栏 + 子模块列表表格
  - [ ] 侧边栏：仓库路径显示 + 刷新按钮
  - [ ] 表格：名称 / 状态（颜色圆点） / 分支 / 操作按钮
  - [ ] 状态图标：绿色(clean)、黄色(ahead/behind)、红色(dirty/detached/orphaned)
- [ ] **3.4** 详情面板
  - [ ] 选中子模块后展示详情
  - [ ] 三个 commit 对比（parent_pointer / local_head / remote_head）
  - [ ] 状态说明 + 建议操作
  - [ ] 显示与远程的 commit 差异数
- [ ] **3.5** 批量操作按钮
  - [ ] "全部更新"按钮
  - [ ] "全部同步"按钮
  - [ ] 操作进度提示

## Iteration 4：操作历史与异常处理

- [ ] **4.1** SQLite schema 设计
  - [ ] `operations` 表（id / time / action / submodule_name / detail）
  - [ ] `retired_submodules` 表（name / url / path / retired_at / reason）
- [ ] **4.2** 实现操作历史记录与查询
  - [ ] 每次原子操作后写入 `operations` 表
  - [ ] `history` 命令列出最近操作
  - [ ] UI 操作历史面板
- [ ] **4.3** Detached / Dirty 状态修复引导
  - [ ] Detached：检测并建议 checkout 到跟踪分支
  - [ ] Dirty：显示未提交文件列表，引导提交或 stash
  - [ ] UI 中显示"修复"按钮
- [ ] **4.4** Orphaned 检测与告警
  - [ ] 检测 parent_pointer 在远程已不存在
  - [ ] 告警提示 + 建议操作（手动干预）
  - [ ] UI 中红色高亮标记
- [ ] **4.5** 操作历史 UI 面板
  - [ ] 列表展示历史操作
  - [ ] 支持按时间/子模块筛选
  - [ ] 支持撤销提示（指引用户使用 reflog）

## Iteration 5：分批灰度与 CI 集成

- [ ] **5.1** 批量选择 + 分批执行
  - [ ] 按依赖顺序排序子模块
  - [ ] 分批选择 UI（多选 + 全选）
  - [ ] 分批执行并显示进度
- [ ] **5.2** `--dry-run` 预览模式
  - [ ] 预览将要执行的操作列表
  - [ ] 不实际执行，仅输出计划
  - [ ] CLI 和 UI 均支持
- [ ] **5.3** 导出操作计划
  - [ ] 导出为 shell 脚本
  - [ ] 导出为 CI 配置文件（YAML）
- [ ] **5.4** 对接 CI/CD
  - [ ] GitHub Actions 模板
  - [ ] GitLab CI 模板
  - [ ] 文档说明集成方式

## Iteration 6：打包分发与文档

- [ ] **6.1** 跨平台打包
  - [ ] Tauri 打包配置
  - [ ] macOS: .dmg
  - [ ] Linux: .AppImage
  - [ ] Windows: .msi
- [ ] **6.2** 用户文档
  - [ ] CLI 帮助文档（`--help`）
  - [ ] 用户手册（README.md）
  - [ ] 常见问题 FAQ
- [ ] **6.3** 端到端测试
  - [ ] 在真实仓库中测试全流程
  - [ ] 测试多平台兼容性
  - [ ] 边界场景测试（空仓库、无子模块、超大仓库）
- [ ] **6.4** 发布 v1.0.0
  - [ ] 更新版本号
  - [ ] 更新 CHANGELOG
  - [ ] GitHub Release
