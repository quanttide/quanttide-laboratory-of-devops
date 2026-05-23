# Git Submodule 专用编辑器 — 迭代计划

## 全部完成

所有 6 个迭代的开发工作已完成。剩余唯一条目需在有网络的环境中执行。

| 迭代 | 提交 | 交付 |
|------|------|------|
| Iter 0 项目脚手架 | `6d388be` | Rust 项目骨架 + CI + 目录结构 |
| Iter 1 核心模型 + CLI | `b05a075` + `1664362` | SubmoduleStatus 7 种状态 + RepoState::scan + health-check |
| Iter 2 原子操作命令集 | `07bb490` | 9 个原子操作 + GitSubmoduleEditor + UpdateStrategy 3 策略 |
| Iter 3 Tauri 外壳 + UI | `66401b7` | src-tauri + web-ui 仪表盘 + src/lib.rs 共享库 |
| Iter 4 操作历史 | `9cea774` | SQLite 持久化 + history 命令 + UI 面板 |
| Iter 5 灰度与分发 | `47a0dd2` | --dry-run + export-ci + CHANGELOG + v1.0.0 |
| Iter 6 规范合规 | `b6e4c8e` | Orphaned merge_base 检测 + remote_unreachable 离线降级 + AggregateStatus + scan_all |
| 迭代间补齐 | 后 9 个提交 | 批量选择 UI、commit 差异数、URL 验证、重复添加检测、集成测试、日期筛选等 |
| Iter 7 文档与发布 | `54a37da` | 完整用户指南 docs/user-guide.md（555 行）— 安装、所有 CLI 命令详解、Web UI 使用、CI 集成、工作流示例、故障排除 |
| Iter 7 后续修复 | `e8bdfc8` | 用户指南评审修复：8 处 CLI 示例错误、新增项目简介/目录/系统要求/卸载指南/撤销风险提示 |
| 实际验证 | — | 在 quanttide 主仓库（17 个子模块）上完成全流程验证：health-check → checkout-all → sync-all → git push，全部修复至 Clean |

## 所有已实现的 CLI 命令

```
kse health-check [path]         # 扫描状态 + 聚合统计
kse add <url> <path> [-b main]  # 添加子模块（含 URL 验证 + 重复检测）
kse init [path]                 # 初始化所有未初始化子模块
kse update <name> [-s strategy] # 更新单个子模块
kse update-all [-s strategy]    # 更新所有子模块
kse sync <name>                 # 同步子模块指针到父仓库
kse sync-all                    # 全部同步
kse checkout <name> <branch>    # 切换分支
kse branch <name> <branch>      # 创建并切换分支
kse checkout-all <branch>       # 批量切换
kse branch-all <branch>         # 批量创建
kse retire <name>               # 退役子模块
kse history [--limit] [--submodule] [--start] [--end]  # 操作历史
kse export-ci [-f format] [-o file]    # 导出 CI 脚本
# 所有变异命令支持 --dry-run 预览
```

## 测试

- 44 个单元测试（model / commands / editor / history / export）
- 32 个集成测试（通过 `cargo test -- --include-ignored` 运行）

## 待完成（需本地 Rust 工具链 + 网络）

| 任务 | 命令 | 状态 |
|------|------|:----:|
| 本地编译验证 | `cargo build && cargo test && cargo clippy -- -D warnings` | ⏳ |
| CI 触发验证 | `git push origin main` | ⏳ |
| Tauri 桌面应用启动 | `cargo tauri dev` | ❌ |
| Tauri 跨平台打包 | `cargo tauri build` | ❌ |
| GitHub Release | `gh release create v1.0.0 ...` | ❌ |

## 未来规划

### Tauri 桌面应用（从蓝图到实现）

当前已存在 `src-tauri/` 外壳和 `web-ui/` 骨架，但尚未接入后端命令。需完成：

| 阶段 | 内容 |
|------|------|
| Tauri 命令桥接 | 将 lib.rs 中的 12 个原子操作逐一注册为 Tauri 命令 |
| Web UI 渲染 | 实现 dev.md 中的布局蓝图：子模块表格、详情面板、聚合统计、操作历史 |
| 状态联动 | 勾选子模块 → 批量操作 → 弹窗预览 → 执行 → 自动刷新 |
| 离线降级 UI | 远程不可达时显示 🛰 标记和横幅 |

### CLI 增强

| 功能 | 说明 |
|------|------|
| 勾选批量操作 | 非全部非单个，支持 `kse update lib-a,lib-b,lib-c` 逗号分隔 |
| 子模块 push 提醒 | sync 后提示用户 push 子模块远程（避免 Orphaned） |
| `kse push-all` | 自动 push 所有有 AheadOfParent 子模块的远程仓库 |

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
