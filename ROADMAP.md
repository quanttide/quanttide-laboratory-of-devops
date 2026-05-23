# Git Submodule 专用编辑器 — 迭代计划

## 已完成迭代

| 迭代 | 提交 | 交付 |
|------|------|------|
| Iter 0-8 | 22 个提交 | CLI + Tauri + PyO3 + 全部测试通过 |

---

## Iteration 9：命令清理与核心价值聚焦

### 动机

70% 的 CLI 命令是 `git` 命令的 Rust 翻译版。本质上就是：

```rust
// 例如 add_submodule:
std::process::Command::new("git")
    .args(["submodule", "add", ...])
    .output()
```

这类包装没有建模层面的新贡献，只是把 shell 命令搬到了 Rust 里。保留它们会增加维护成本（git2 API 版本兼容问题已有前车之鉴），且对用户来说仍然是学会了一个新工具，不是学会了一个新概念。

真正的新贡献只有三块：

| 贡献 | 说明 |
|------|------|
| `RepoState::scan()` + 7 种状态分类 | 三路 commit 比对，`git submodule status` 做不到 |
| `sync_to_parent` | 子模块 → 父仓库指针更新的原子操作，git 没有这条命令 |
| `retire_submodule`（半个） | `git submodule deinit` 后的 `.gitmodules` + index 清理自动化 |

### 9.1 移除无新贡献的命令

| 命令 | 原因 | 替代方案 |
|------|------|----------|
| `kse add <url> <path>` | 纯 `git submodule add` 包装 | `git submodule add <url> <path>` |
| `kse init` | 纯 `git submodule init` 循环 | `git submodule update --init` |
| `kse update <name>` | 纯 `git submodule update --remote` 包装 | `git submodule update --remote <name>` |
| `kse update-all` | 同上，批量版 | `git submodule update --recursive --remote` |
| `kse checkout <name> <branch>` | 纯 `git checkout` 包装 | `git -C <path> checkout <branch>` |
| `kse branch <name> <branch>` | 纯 `git checkout -b` 包装 | `git -C <path> checkout -b <branch>` |
| `kse checkout-all <branch>` | 批量版，同上 | shell 循环 |
| `kse branch-all <branch>` | 批量版，同上 | shell 循环 |

### 9.2 保留并聚焦的命令

| 命令 | 状态 |
|------|------|
| `kse status [path]`（原 `health-check`） | 核心贡献，保留并重命名 |
| `kse sync parent <name>`（原 `sync`） | 核心贡献，保留并纳入 sync 族 |
| `kse sync parent --all`（原 `sync-all`） | 核心贡献，保留 |
| `kse retire <name>` | 半贡献，保留 |
| `kse history [--limit] [--submodule] [--start] [--end]` | 保留 |
| `kse export-ci [-f format]` | 保留（CI 与 devops 集成相关） |

### 9.3 `health-check` → `status` 重命名

| 现名 | 新名 |
|------|------|
| `kse health-check` | `kse status` |
| `health_check()` trait 方法 | `status()` |
| Tauri command `health_check` | `status` |

`kse health-check` 保留为隐藏 alias，输出迁移提示。

### 9.4 `sync` 命令族

| 新命令 | 原身 | 方向 |
|--------|------|------|
| `kse sync parent <name>` | `kse sync <name>` | 子模块 → 父仓库（核心贡献） |
| `kse sync parent --all` | `kse sync-all` | 批量版 |
| `kse sync platform <name> --env <env>` | 新增 | 跨环境子模块版本对齐（CI 场景） |

`kse sync` 保留为 `kse sync parent` 的快捷别名。

### 9.5 清理后 CLI 命令集

```
kse status [path]                     # 👈 重命名，唯一入口
kse sync parent <name> [--all]        # 👈 核心贡献
kse sync platform <name> --env <env>  # 👈 新增
kse retire <name>                     # 👈 保留
kse history [--limit] [--submodule] [--start] [--end]  # 👈 保留
kse export-ci [-f format] [-o file]   # 👈 保留
# 所有命令支持 --dry-run
```

从 14 个子命令精简为 6 个子命令，全部指向核心贡献。

### 9.6 受影响文件

| 文件 | 变更 |
|------|------|
| `src/commands/mod.rs` | trait: 移除 add/init/update/checkout/branch，health_check→status |
| `src/commands/editor.rs` | 移除对应实现，status 替代 health_check，sync_parent/sync_platform 实现 |
| `src/main.rs` | CLI: 移除 8 个子命令，重命名，新增 sync platform |
| `src-tauri/src/main.rs` | Tauri commands: 移除 5 个，重命名 |
| `web-ui/index.html` | 移除无用按钮，sync 族重排 |
| `web-ui/src/app.js` | 移除无用 invoke 调用 |
| `tests/integration.rs` | 移除对已删除命令的测试 |
| `docs/user-guide.md` | 全部重写 |
| `docs/dev.md` | 更新 trait 定义 |

### 9.7 任务分解

| 任务 | 预估 |
|------|------|
| 9.1 trait + 实现清理（commands/mod.rs + editor.rs） | 0.3d |
| 9.2 CLI 清理 + 重命名（main.rs） | 0.2d |
| 9.3 Tauri command 清理（src-tauri） | 0.1d |
| 9.4 Web UI 清理（index.html + app.js） | 0.1d |
| 9.5 sync platform 子命令骨架 | 0.2d |
| 9.6 测试更新 + 编译验证 | 0.3d |
| 9.7 文档（user-guide.md + dev.md） | 0.3d |

---

## 清理前 vs 清理后

```
清理前 (14 子命令):                             清理后 (6 子命令):
  add              →   git submodule add         status 👈 核心
  init             →   git submodule init        sync parent 👈 核心
  update           →   git submodule update       sync platform 👈 新增
  update-all       →   同上 (批量)                retire
  checkout         →   git checkout              history
  branch           →   git checkout -b           export-ci
  checkout-all     →   同上 (批量)
  branch-all       →   同上 (批量)
  sync             →   🏆 核心贡献 ← 保留
  sync-all         →   🏆 核心贡献 ← 保留
  retire           →   🏆 半贡献 ← 保留
  health-check     →   🏆 核心贡献 ← 重命名
  history          →   保留
  export-ci        →   保留
```

---

## 待完成（需本地环境）

| 任务 | 命令 |
|------|------|
| Iter 9 编译验证 | `cargo build && cargo test && cargo clippy -- -D warnings` |
| CI 触发验证 | `git push origin main` |
| GitHub Release | `gh release create v1.0.0 ...` |

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
