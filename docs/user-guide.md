# KSE 用户指南

KSE（Kernel Submodule Editor）是一个面向多仓库项目的 Git 子模块管理工具。它提供 CLI 和可选的 Tauri 桌面应用，帮助开发者高效管理大量子模块的状态查看、批量更新、分支切换、同步提交等日常操作。核心设计理念：**原子命令、状态驱动、幂等操作**。

> 🌟 适合场景：多子模块 monorepo、微服务仓库、平台工程团队。

## 目录

- [安装](#安装)
- [CLI 快速参考](#cli-快速参考)
- [命令详解](#命令详解)
- [Web UI 使用指南](#web-ui-使用指南)
- [CI 集成](#ci-集成)
- [撤销操作](#撤销操作)
- [常见场景工作流](#常见场景工作流)
- [故障排除](#故障排除)
- [相关文档](#相关文档)

## 安装

### 系统要求

| 项目 | 要求 |
|------|------|
| 操作系统 | Linux / macOS / Windows |
| Rust 版本 | ≥ 1.70.0 |
| 依赖 | libgit2 ≥ 1.3（或使用 `vendored-libgit2` 特性） |
| 推荐磁盘 | 构建需要约 500MB 临时空间 |

### 前置依赖

```bash
# Ubuntu / Debian
sudo apt install libgit2-dev pkg-config cmake

# macOS
brew install libgit2

# Windows (使用 vcpkg)
vcpkg install libgit2
```

### 从源码构建

```bash
git clone <repo-url>
cd examples/default
cargo build --release
```

编译后的二进制位于 `target/release/kse`。建议将路径加入 `PATH`：

```bash
# 临时
export PATH="$PWD/target/release:$PATH"

# 永久（追加到 ~/.bashrc）
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

### Tauri 桌面应用（可选）

```bash
cargo install tauri-cli
cargo tauri dev      # 开发模式
cargo tauri build    # 构建安装包
```

### 卸载

```bash
# 删除二进制
rm $(which kse)

# 可选：清理历史数据
rm -rf .git/kse/history.db

# 可选：删除 Tauri 应用（Linux）
rm -rf ~/.local/share/com.kse.kse
```

---

## CLI 快速参考

```
kse <COMMAND> [选项] [参数]
```

所有修改数据的命令支持 `--dry-run` 预览模式。

### 全局选项

| 选项 | 说明 |
|------|------|
| `--dry-run` | 预览模式：仅输出操作计划，不执行 |
| `--help` | 显示帮助信息 |
| `--version` | 显示版本号 |

---

## 命令详解

### `kse health-check` — 健康检查

扫描仓库中的所有子模块，输出每个子模块的状态、跟踪分支和 commit 差异。

```bash
# 扫描当前目录
kse health-check

# 扫描指定仓库
kse health-check /path/to/repo

# 输出示例
# 仓库: /home/user/my-project
# 子模块总数: 3
# 干净: 1
# 需要关注: lib-a, lib-b
#
# 聚合统计:
#   总数: 3
#   ✅ Clean: 1
#   ⬇ BehindRemote: 1
#   🔴 Dirty: 1
#
# 名称                    状态             分支         差异
# lib-a                  BehindRemote     main        -3
# lib-b                  Dirty            dev
# lib-c                  Clean            main
#
# 健康问题:
#   [lib-a] 远程有更新，本地落后 — 运行 update 获取最新代码
#   [lib-b] 有未提交的修改 — 提交或 stash 当前修改
```

**状态说明**：

| 状态 | 颜色 | 含义 | 建议操作 |
|------|------|------|----------|
| Clean | 绿色 | 三方 commit 一致 | 无需操作 |
| AheadOfParent | 黄色 | 本地有父仓库未记录的新提交 | `kse sync <name>` |
| BehindRemote | 黄色 | 远程有更新，本地落后 | `kse update <name>` |
| Detached | 红色 | 游离 HEAD | `kse checkout <name> <branch>` |
| Dirty | 红色 | 有未提交的修改 | 手动 commit 或 stash |
| Orphaned | 红色 | 父仓库记录的 commit 在远程已不存在 | 手动干预 |
| Uninitialized | 灰色 | 尚未初始化 | `kse init` 或 `kse update <name>` |

当远程仓库不可达时，状态列显示 🛰 标记，详情面板显示"远程不可达"横幅，此时跳过 Orphaned/BehindRemote 判定避免误报。

---

### `kse add` — 添加子模块

```bash
# 基本用法
kse add https://github.com/user/lib.git libs/lib-a

# 指定跟踪分支
kse add https://github.com/user/lib.git libs/lib-a --branch main
kse add https://github.com/user/lib.git libs/lib-a -b develop

# 指定仓库路径
kse add https://github.com/user/lib.git libs/lib-a /path/to/project

# dry-run 预览
kse --dry-run add https://github.com/user/lib.git libs/lib-a
```

添加前自动执行：
1. **重复检测**：同名或同路子模块已存在时拒绝
2. **路径检测**：目标路径已存在时拒绝
3. **URL 验证**：通过 `git ls-remote` 检查 URL 是否可达

---

### `kse init` — 初始化子模块

初始化所有未初始化的子模块：

```bash
kse init
kse init /path/to/project
```

---

### `kse update` — 更新子模块

```bash
# 更新单个子模块（默认 fast-forward）
kse update lib-a

# 指定更新策略
kse update lib-a --strategy rebase
kse update lib-a -s merge
kse update lib-a -s fast-forward

# 指定仓库
kse update lib-a /path/to/project
```

**更新策略**：

| 策略 | 命令 | 行为 |
|------|------|------|
| FastForward | `-s fast-forward` | 仅快进（默认），失败则报错 |
| Rebase | `-s rebase` | 将本地提交变基到远程分支之上 |
| Merge | `-s merge` | 合并远程分支到本地分支 |

**错误处理**：子模块有未提交修改时，更新被阻止并给出提示。

---

### `kse update-all` — 批量更新

```bash
# 全部更新（默认 fast-forward）
kse update-all

# 指定策略
kse update-all -s merge

# 指定仓库
kse update-all /path/to/project
```

批量更新具有容错能力：单个子模块失败不影响其他子模块。

---

### `kse sync` — 同步到父仓库

子模块中有新提交后，将子模块指针更新到父仓库：

```bash
kse sync lib-a
kse sync lib-a /path/to/project
```

执行的操作：
1. 将子模块路径添加到父仓库索引
2. 创建 commit（消息："chore: 更新子模块 'name' 指针"）
3. 记录操作历史

---

### `kse sync-all` — 全部同步

```bash
kse sync-all
kse sync-all /path/to/project
```

---

### `kse checkout` — 切换分支

```bash
# 在子模块中切换到已存在的分支
kse checkout lib-a main
kse checkout lib-a feature-x

# 指定仓库
kse checkout lib-a main /path/to/project
```

---

### `kse branch` — 创建分支

```bash
# 在子模块中基于当前 HEAD 创建并切换到新分支
kse branch lib-a feature-x
kse branch lib-a hotfix-v2

# 指定仓库
kse branch lib-a feature-x /path/to/project
```

---

### `kse checkout-all` — 批量切换分支

将所有子模块切换到指定分支：

```bash
kse checkout-all main
kse checkout-all release-v1 /path/to/project
```

---

### `kse branch-all` — 批量创建分支

在所有子模块中创建并切换到新分支：

```bash
kse branch-all feature-x
kse branch-all hotfix /path/to/project
```

---

### `kse retire` — 退役子模块

从仓库中移除一个子模块（软删除）：

```bash
kse retire lib-old
kse retire lib-old /path/to/project
```

执行的操作：
1. `git submodule deinit -f <name>`
2. 从 `.gitmodules` 移除配置段
3. 从索引中移除
4. 记录退役信息到 SQLite 数据库

> 退役不会删除子模块的克隆目录（`deinit -f` 会清理），如需保留本地数据请先备份。

---

### `kse history` — 查看操作历史

```bash
# 最近 20 条
kse history

# 指定数量
kse history -n 50

# 按子模块筛选
kse history -m lib-a

# 按日期范围筛选
kse history --start 2024-01-01 --end 2024-12-31

# 组合筛选
kse history -n 10 -m lib-b --start 2024-06-01

# 指定仓库
kse history /path/to/project
```

输出示例：

```
最近 3 条操作记录:

  ✓ [2024-01-15 10:30:00] sync: lib-a (同步到父仓库)
  ✓ [2024-01-15 10:28:00] update: lib-a (strategy=FastForward)
  ✗ [2024-01-15 10:25:00] update: lib-b (子模块 'lib-b' 有未提交的修改)
```

> 历史数据存储在 `.git/kse/history.db`（SQLite 格式），可用 `sqlite3` 直接查询。

---

### `kse export-ci` — 导出 CI 脚本

将当前子模块状态导出为可执行的 CI 配置：

```bash
# 输出 shell 脚本（默认）
kse export-ci

# 输出 GitHub Actions
kse export-ci -f github

# 输出 GitLab CI
kse export-ci -f gitlab

# 写入文件
kse export-ci -f shell -o update-submodules.sh

# 指定仓库
kse export-ci /path/to/project -f github
```

仅导出状态为 `BehindRemote`、`Uninitialized`、`Detached`、`Dirty` 的子模块。

---

## Web UI 使用指南

### 启动

```bash
cargo tauri dev
```

### 界面布局

```
┌────────────────────────────────────────────────────┐
│  KSE  [仓库路径: ________________] [刷新]           │
├────────────┬───────────────────────────────────────┤
│  概览      │  子模块列表                            │
│  总数: 3   │  ☑ 名称    │ 状态 │ 分支  │ 操作      │
│  干净: 1   │  ☐ lib-a   │ ●落后 │ main  │ [更新]   │
│  关注: 2   │  ☑ lib-b   │ ●脏   │ dev   │ [查看]   │
│            │  ☐ lib-c   │ ●干净 │ main  │           │
│  聚合统计  ├───────────────────────────────────────┤
│  落后: 1   │  详情面板                               │
│  脏: 1     │  lib-b ●脏                              │
│            │  差异: 同步                              │
│  批量操作  │  有未提交的修改。建议: 手动 commit...    │
│  [更新选中]│  ┌──────┬──────┬──────┐                │
│  [同步选中]│  │父指针 │ HEAD │ 远程 │                │
│  [全部更新]│  │abc1234│abc1234│abc1234│               │
│  [全部同步]│  └──────┴──────┴──────┘                │
│            │  [更新] [同步] [退役]                    │
│  导出 CI   ├───────────────────────────────────────┤
│  [Shell]   │  操作历史                               │
│  [GitHub]  │  ✓ 2024-01-15 sync: lib-a              │
│  [GitLab]  │  ✓ 2024-01-15 update: lib-a            │
└────────────┴───────────────────────────────────────┘
```

### 功能说明

| 区域 | 功能 |
|------|------|
| 仓库路径 | 输入要扫描的仓库路径，自动触发扫描 |
| 刷新按钮 | 手动触发扫描 |
| 概览 | 显示总数、干净数、关注数及聚合统计 |
| 子模块表格 | 勾选多行 → 批量操作；点击行 → 查看详情 |
| 状态列 | 颜色圆点 + 文本，离线时显示 🛰 标记 |
| 操作列 | 按状态显示上下文按钮（更新/同步/退役/查看） |
| 详情面板 | 三个 commit 对比 + 差异数 + 状态引导 + 操作按钮 |
| 批量操作 | "更新选中"、"同步选中"、"全部更新"、"全部同步" |
| 导出 CI | 一键复制 shell/GitHub/GitLab CI 脚本到剪贴板 |
| 操作历史 | 显示最近操作记录，可筛选日期范围 |

### 操作流程示例

```
1. 输入仓库路径 → 自动扫描
2. 查看表格：红色状态需要关注
3. 点击行 → 详情面板显示问题描述和建议
4. 勾选待更新的子模块 → 点"更新选中"
5. 弹窗预览操作计划 → 确认执行
6. 进度条显示执行状态
7. 完成后自动重新扫描
```

---

## CI 集成

### GitHub Actions

```yaml
# .github/workflows/submodules.yml
name: Update Submodules

on:
  workflow_dispatch:
  schedule:
    - cron: '0 6 * * 1'  # 每周一 06:00 UTC

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: true
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Build KSE
        run: cargo build --release
      - name: Update submodules
        run: ./target/release/kse update-all
      - name: Sync to parent
        run: ./target/release/kse sync-all
```

也可以通过 `kse export-ci -f github` 自动生成。

---

## 撤销操作

> ⚠️ **风险提示**：以下 `git reset --hard` 会**丢弃所有未提交的修改**，请先确认工作区干净或已 stash。

每次原子操作都被 Git 的 reflog 记录：

```bash
# 查看父仓库 reflog
git reflog

# 恢复到操作前的状态
git reset --hard HEAD@{1}

# 查看子模块 reflog
cd <子模块路径>
git reflog
```

操作历史数据库位于 `.git/kse/history.db`，可直接查询：

```bash
sqlite3 .git/kse/history.db "SELECT * FROM operations ORDER BY id DESC LIMIT 10;"
```

---

## 常见场景工作流

### 日常开发

```bash
# 1. 检查状态
kse health-check

# 2. 更新所有子模块到最新
kse update-all

# 3. 在子模块中工作
cd libs/lib-a
git checkout -b my-feature
# ... 修改代码 ...
git add . && git commit -m "feat: ..."

# 4. 同步子模块指针到父仓库
cd ..
kse sync lib-a

# 5. 提交父仓库
git add libs/lib-a
git commit -m "chore: update lib-a"
```

### 多子模块批量操作

```bash
# 批量创建分支
kse branch-all feature-x

# 批量切换
kse checkout-all feature-x

# 查看状态差异
kse health-check

# 批量更新
kse update-all -s merge
```

### 清理不再使用的子模块

```bash
# 1. 确认要退役的子模块
kse health-check

# 2. 退役
kse retire lib-old

# 3. 提交父仓库变更
git add .gitmodules
git commit -m "chore: retire submodule lib-old"
```

---

## 故障排除

| 问题 | 原因 | 解决 |
|------|------|------|
| `无法打开 Git 仓库` | 指定路径不是 Git 仓库 | 确认路径并确保存在 `.git` 目录 |
| `URL 不可达` | 远程仓库地址无效或网络不通 | 检查 URL 拼写、网络连接、SSH key |
| `子模块 XX 已存在` | 同名子模块已存在 | 使用不同的名称，或先退役旧的 |
| `路径已存在` | 目标路径已被占用 | 使用不同的路径，或先移除该路径 |
| `有未提交的修改` | 子模块工作区不干净 | 先 commit 或 stash |
| `找不到子模块 XX` | 子模块名称错误 | 运行 `kse health-check` 查看正确名称 |
| 状态显示 🛰 | 远程仓库不可达 | 检查网络连接，或在本地先运行 `git fetch` |
| 状态显示为 Orphaned | 父仓库记录的 commit 在远程已不存在 | 远程分支可能被 rebase 或删除，需手动修复 |
| Tauri 应用启动失败 | 缺少系统依赖 | 安装 `libgtk-3-dev` / `libwebkit2gtk-4.0-dev` 等 |
| `cargo build` 失败 | git2 编译需要 libgit2 | 安装 `libgit2-dev` 或使用 `vendored-libgit2` 特性 |

### 获取帮助

```bash
kse --help          # 全部命令概览
kse <COMMAND> --help  # 单个命令详情
```

---

## 相关文档

- [开发蓝图](dev.md) — 架构设计和技术选型
- [迭代计划](../ROADMAP.md) — 版本规划
- [变更日志](../CHANGELOG.md) — 版本历史
