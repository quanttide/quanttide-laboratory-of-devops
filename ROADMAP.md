# Git Submodule 专用编辑器 — 迭代计划

## 时间线

```
Iteration 0 ── Iteration 1 ── Iteration 2 ── Iteration 3 ── Iteration 4 ── Iteration 5
   0.5w           2w              2w              2w              2w              2w
```

---

## Iteration 0：项目脚手架（0.5w）

**目标**：搭建可编译、可测试、可 CI 的基础工程骨架。

**完成定义**：
- `cargo build` 通过
- `cargo test` 通过（至少一个有效测试）
- CI 绿通过
- 目录结构稳定，后续迭代不需要重构目录

| 任务 | 预估 |
|------|------|
| 0.1 初始化 Rust 项目，配置 git2/clap/serde 依赖（vendored） | 0.5d |
| 0.2 添加 `.gitignore`、`rustfmt.toml` | 0.1d |
| 0.3 搭建目录结构 `src/model/`、`src/commands/` | 0.1d |
| 0.4 配置 GitHub Actions CI（cargo check + test + clippy） | 0.5d |
| 0.5 实现一个可运行的 `main.rs` 空 CLI（仅 `--help`） | 0.3d |

**交付物**：
- 编译通过的 Rust crate + CI 配置
- 可执行 `kse --help` 输出帮助信息

---

## Iteration 1：核心模型与 CLI 原型（2w）

**目标**：实现子模块状态模型和 `health-check` CLI 命令，可扫描并展示仓库所有子模块的状态。

**完成定义**：
- 7 种状态判定逻辑有单元测试覆盖
- `kse health-check` 在含 `.gitmodules` 的仓库中可正确输出
- 无 `.gitmodules` 时优雅降级

| 任务 | 预估 | 前置依赖 |
|------|------|----------|
| 1.1 实现 `CommitHash` 新类型 + `SubmoduleStatus` 枚举（含优先级排序） | 0.5d | 0.1, 0.3 |
| 1.2 实现 `Submodule` 结构体 + `RepoState` 结构体 | 0.5d | 1.1 |
| 1.3 实现 `RepoState::scan()` 扫描逻辑（git2 操作） | 1d | 1.2 |
| 1.4 实现 `health_check()` CLI 命令（表格输出 + 颜色标识） | 1d | 1.3 |
| 1.5 单元测试覆盖所有状态判定逻辑 | 1d | 1.1 |

**交付物**：`kse health-check` CLI 命令

---

## Iteration 2：原子操作命令集（2w）

**目标**：补全所有原子操作，支持对单个子模块的增、删、改、同步。

**完成定义**：
- 每个原子操作有独立的单元测试
- `update_single` 三种策略全部实现
- 集成测试覆盖完整生命周期：add → update → sync → retire

| 任务 | 预估 | 前置依赖 |
|------|------|----------|
| 2.1 实现 `add_submodule` | 0.5d | 1.3 |
| 2.2 实现 `init_all` / `update_single`（FastForward / Rebase / Merge） | 1.5d | 1.3 |
| 2.3 实现 `sync_to_parent` / `sync_all_to_parent` | 1d | 1.3 |
| 2.4 实现 `retire_submodule`（软删除） | 0.5d | 1.3 |
| 2.5 实现 `checkout_branch` / `create_branch` | 0.5d | 1.3 |
| 2.6 集成测试：临时仓库 + 多子模块场景 + 全流程验证 | 1d | 2.1–2.5 |

**交付物**：完整的 `kse` CLI 命令集

---

## Iteration 3：Tauri 外壳与状态驱动 UI（2w）

**目标**：用 Tauri 封装 CLI 逻辑，搭建界面框架，实现子模块列表渲染。

**完成定义**：
- Tauri 应用可在 macOS/Linux 启动
- 子模块列表表格正确渲染状态颜色
- 详情面板展示三个 commit 对比

| 任务 | 预估 | 前置依赖 |
|------|------|----------|
| 3.1 初始化 Tauri 项目，集成 core crate | 1d | — |
| 3.2 实现后端命令绑定（Tauri commands） | 1d | 1.0 |
| 3.3 实现 UI 侧边栏 + 子模块列表表格 | 1.5d | 3.2 |
| 3.4 实现详情面板（三个 commit 对比、建议操作） | 1d | 3.3 |
| 3.5 实现"健康检查"和"全部更新"等批量操作 | 0.5d | 3.3 |

**交付物**：可运行的 Tauri 桌面应用（macOS / Linux）

---

## Iteration 4：操作历史与异常处理（2w）

**目标**：引入 SQLite 持久化操作历史，覆盖异常状态的恢复路径。

**完成定义**：
- 每次原子操作写入 `operations` 表
- UI 操作历史面板可按时间/子模块筛选
- Detached/Dirty/Orphaned 状态有修复引导

| 任务 | 预估 | 前置依赖 |
|------|------|----------|
| 4.1 SQLite schema 设计 + 实现 | 0.5d | — |
| 4.2 实现操作历史记录与查询（`history` 命令 + UI 面板） | 1d | 4.1 |
| 4.3 实现 Detached / Dirty 状态的修复引导 UI | 1d | 3.0 |
| 4.4 实现 Orphaned 检测与告警 | 0.5d | 1.3 |
| 4.5 操作历史 UI 面板（筛选、搜索、ref log 指引） | 0.5d | 4.2 |

**交付物**：具备审计能力和异常引导的桌面应用

---

## Iteration 5：分批灰度与打包分发（2w）

**目标**：支持多仓库批量更新策略，对接 CI/CD，跨平台打包。

**完成定义**：
- 批量选择 + 分批执行可用
- `--dry-run` 预览模式可用
- 导出 CI 可执行脚本
- 跨平台安装包可分发

| 任务 | 预估 | 前置依赖 |
|------|------|----------|
| 5.1 实现批量选择 + 分批执行（按依赖顺序） | 1d | 2.0 |
| 5.2 实现 `--dry-run` 预览模式（CLI + UI） | 0.5d | 2.0 |
| 5.3 导出操作计划为 CI 可执行脚本 | 1d | 5.2 |
| 5.4 Tauri 跨平台打包配置（.dmg / .AppImage / .msi） | 1d | 3.0 |
| 5.5 用户手册 + CLI 帮助文档 | 1d | — |
| 5.6 端到端测试 + v1.0.0 发布 | 1d | 5.1–5.5 |

**交付物**：正式发布的 v1.0.0 安装包

---

## 关键依赖图

```
Iter 0 ──→ Iter 1 ──→ Iter 2 ──→ Iter 3 ──→ Iter 4 ──→ Iter 5
                ↓                      ↑            ↑
            (CI 基础)          (Tauri 外壳)   (异常处理)
```

- Iteration 0 是所有后续迭代的前提
- Iteration 3 需要 Iteration 1 的模型和 Iteration 2 的命令集
- Iteration 4 依赖 Iteration 3 的 UI 框架
- Iteration 5 是最终集成与分发

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
