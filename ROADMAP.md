# Git Submodule 专用编辑器 — 迭代计划

## Iteration 1：核心模型与 CLI 原型

**目标**：实现子模块状态模型和 `health-check` CLI 命令，可扫描并展示仓库所有子模块的状态。

| 任务 | 预估 |
|------|------|
| 1.1 初始化 Rust 项目，配置 git2 依赖 | 0.5d |
| 1.2 实现 `Submodule` 结构体和 `SubmoduleStatus` 枚举 | 0.5d |
| 1.3 实现 `RepoState` 扫描逻辑（遍历 `.gitmodules`，比对三个 commit） | 1d |
| 1.4 实现 `health_check()` 命令 — CLI 输出表格 | 1d |
| 1.5 单元测试覆盖所有状态判定逻辑 | 1d |

**交付物**：`submodule-editor-core` crate + `kse health-check` CLI 命令

---

## Iteration 2：原子操作命令集

**目标**：补全所有原子操作，支持对单个子模块的增、删、改、同步。

| 任务 | 预估 |
|------|------|
| 2.1 实现 `add_submodule` | 0.5d |
| 2.2 实现 `init_all` / `update_single`（三种策略：FastForward / Rebase / Merge） | 1.5d |
| 2.3 实现 `sync_to_parent`（提交子模块并更新父仓库指针） | 1d |
| 2.4 实现 `retire_submodule`（软删除） | 0.5d |
| 2.5 实现 `checkout_branch` / `create_branch` | 0.5d |
| 2.6 集成测试：模拟多子模块场景，验证全流程 | 1d |

**交付物**：完整的 `kse` CLI 命令集

---

## Iteration 3：Tauri 外壳与状态驱动 UI

**目标**：用 Tauri 封装 CLI 逻辑，搭建界面框架，实现子模块列表渲染。

| 任务 | 预估 |
|------|------|
| 3.1 初始化 Tauri 项目，集成 core crate | 1d |
| 3.2 实现后端命令绑定（Tauri commands 包裹 Rust 函数） | 1d |
| 3.3 实现 UI 侧边栏 + 子模块列表表格（状态颜色标识） | 1.5d |
| 3.4 实现详情面板（三个 commit 对比、建议操作） | 1d |
| 3.5 实现"健康检查"和"全部更新"等批量操作 | 0.5d |

**交付物**：可运行的 Tauri 桌面应用（macOS / Linux）

---

## Iteration 4：操作历史与异常处理

**目标**：引入 SQLite 持久化操作历史，覆盖异常状态的恢复路径。

| 任务 | 预估 |
|------|------|
| 4.1 SQLite schema 设计（操作记录、退役清单） | 0.5d |
| 4.2 实现操作历史记录与查询 | 1d |
| 4.3 实现 Detached / Dirty 状态的修复引导 UI | 1d |
| 4.4 实现 Orphaned 检测与告警 | 0.5d |
| 4.5 UI 操作历史面板 | 0.5d |

**交付物**：具备审计能力和异常引导的桌面应用

---

## Iteration 5：分批灰度与 CI 集成

**目标**：支持多仓库批量更新策略，对接 CI/CD 流水线。

| 任务 | 预估 |
|------|------|
| 5.1 实现批量选择 + 分批执行（按依赖顺序） | 1d |
| 5.2 实现 `--dry-run` 预览模式 | 0.5d |
| 5.3 导出操作计划为 CI 可执行的脚本/配置 | 1d |
| 5.4 对接 GitHub Actions / GitLab CI 模板 | 1d |

**交付物**：可集成到 CI/CD 的生产级工具

---

## Iteration 6：打包分发与文档

**目标**：跨平台打包，完善用户文档。

| 任务 | 预估 |
|------|------|
| 6.1 Tauri 打包配置（macOS .dmg / Linux .AppImage / Windows .msi） | 1d |
| 6.2 用户手册 + CLI 帮助文档 | 1d |
| 6.3 端到端测试（真实仓库场景） | 1d |
| 6.4 发布 v1.0.0 | 0.5d |

**交付物**：正式发布的 v1.0.0 安装包

---

## 时间线

```
Iteration 1 ── Iteration 2 ── Iteration 3 ── Iteration 4 ── Iteration 5 ── Iteration 6
   2w             2w              2w              2w              2w              1w
                                                                                   
├─────────────────────────────────────────────────────────────────────────────────┤
                                          11w
```

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
