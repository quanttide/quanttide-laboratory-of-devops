# AGENTS.md

## 项目概览

Git Submodule 专用编辑器 — 一款面向多仓库项目的子模块可视化工具。

- **核心语言**：Rust
- **Git 库**：git2 (libgit2 绑定)
- **UI 框架**：Tauri (Rust + Web 前端)
- **数据持久化**：SQLite
- **CLI 名称**：`kse`

## 目录结构

```
submodule-editor-core/
├── src/
│   ├── model/          # 核心模型（Submodule, SubmoduleStatus, RepoState）
│   ├── commands/       # 原子命令实现
│   └── main.rs         # CLI 入口
├── docs/
│   └── dev.md          # 开发蓝图
├── ROADMAP.md          # 迭代计划
├── TODO.md             # 可执行任务清单
└── AGENTS.md           # 本文件
```

## 开发规范

- **原子命令**：每个命令无副作用、无编排逻辑，对应 UI 中一个按钮。
- **状态驱动**：UI 渲染完全由 `SubmoduleStatus` 驱动，不做额外状态管理。
- **错误处理**：所有 Git 操作返回 `Result<T, Error>`，前端统一展示错误信息。
- **幂等性**：`update` 在 Clean 状态下重复执行不产生副作用。

## 迭代顺序

严格按照 `ROADMAP.md` 定义执行：

1. 核心模型 + CLI 原型（Iteration 1）
2. 原子操作命令集（Iteration 2）
3. Tauri 外壳 + UI（Iteration 3）
4. 操作历史 + 异常处理（Iteration 4）
5. 灰度策略 + CI 集成（Iteration 5）
6. 打包分发（Iteration 6）

## 子模组信息

本项目作为 `quanttide-devops` 仓库的子模组，路径为 `examples/default`。

**操作规范：**
- 在 `examples/default/` 目录下提交修改
- 回到 `quanttide-devops` 根目录执行 `git add examples/default` 更新父仓库指针
- 拉取更新：`git submodule update --remote examples/default`
