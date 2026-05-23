# qtcloud-devops — Git 子模块管理工具

KSE（Kernel Submodule Editor）是一个面向多仓库项目的 Git 子模块管理工具，作为 `qtcloud-devops` CLI 的 `code` 子命令集提供。

KSE **不做** `git` 已有的事（添加、初始化、更新、切换分支等直接用 `git` 命令），只做 `git` **做不到**的事：**三路 commit 比对 + 7 种状态分类** 和 **子模块 → 父仓库指针同步**。

## 安装

```bash
# 构建
cd examples/default
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

## CLI 快速参考

```
qtcloud-devops code <COMMAND> [选项] [参数]
```

所有命令支持 `--dry-run` 预览模式。

## 命令详解

### `status` — 查看子模块状态

```bash
qtcloud-devops code status
qtcloud-devops code status /path/to/repo
```

通过三路 commit 比对（父仓库指针 / 本地 HEAD / 远程 HEAD）判定每个子模块的状态。

**7 种状态**：

| 状态 | 含义 | 建议 |
|------|------|------|
| Clean | 三方 commit 一致 | 无需操作 |
| AheadOfParent | 本地有父仓库未记录的新提交 | `qtcloud-devops code sync parent <name>` |
| BehindRemote | 远程有更新，本地落后 | `git submodule update --remote <name>` |
| Detached | 游离 HEAD | `git checkout <name> <branch>` |
| Dirty | 有未提交的修改 | 手动 commit 或 stash |
| Orphaned | 父仓库记录的 commit 在远程已不存在 | 手动干预 |
| Uninitialized | 尚未初始化 | `git submodule update --init <name>` |

远程不可达时显示 🛰 标记，跳过 Orphaned/BehindRemote 判定。

### `sync parent` — 同步到父仓库

Git 没有的原子操作：子模块有更新后更新父仓库指针。

```bash
qtcloud-devops code sync parent lib-a
qtcloud-devops code sync parent --all
```

### `sync platform` — 跨环境版本对齐

```bash
qtcloud-devops code sync platform lib-a --env production
```

检查子模块在目标环境的状态，输出差异报告（不执行变更）。

### `retire` — 退役子模块

完整自动化反注册：`deinit` + `.gitmodules` + index 清理。

```bash
qtcloud-devops code retire lib-old
```

### `history` — 操作历史

```bash
qtcloud-devops code history -n 50
qtcloud-devops code history -m lib-a --start 2024-01-01
```

### `export-ci` — 导出 CI 脚本

```bash
qtcloud-devops code export-ci -f github -o .github/workflows/submodules.yml
```

## 常见场景

```bash
# 查看状态
qtcloud-devops code status

# 同步所有子模块到父仓库
qtcloud-devops code sync parent --all

# 退役
qtcloud-devops code retire lib-old
```

## 故障排除

```bash
qtcloud-devops code --help
qtcloud-devops code <COMMAND> --help
```
