# Git Submodule 专用编辑器 — 迭代计划

## 完成情况

```
b33458e ── 6d388be ── 1664362 ── b05a075 ── 07bb490 ── 66401b7 ── 9cea774 ── 47a0dd2 ── c978755
docs       Iter 0      fix        Iter 1     Iter 2     Iter 3     Iter 4     Iter 5     docs
```

| 迭代 | 状态 | 提交 |
|------|------|------|
| Iter 0 项目脚手架 | ✅ 完成 | `6d388be` |
| Iter 1 核心模型 + CLI | ✅ 完成 | `b05a075` |
| Iter 2 原子操作命令集 | ✅ 完成 | `07bb490` |
| Iter 3 Tauri 外壳 + UI | ✅ 完成 | `66401b7` |
| Iter 4 操作历史与异常处理 | ✅ 完成 | `9cea774` |
| Iter 5 灰度与打包 | ✅ 完成 | `47a0dd2` |

---

## Iteration 6：规范合规补齐

**目标**：对齐 `git-submodule.md` v1.1 标准，修补实现与规范之间的差距。

### 6.1 Orphaned 检测逻辑

| 任务 | 状态 | 实际实现 |
|------|------|----------|
| `is_orphaned()` — merge_base 检查 parent_pointer | ✅ | `RepoState::scan()` 内联，`bb058e6` |
| 插入判定分支 Dirty > Orphaned > Detached | ✅ | `bb058e6` — `SubmoduleStatus::priority()` 已匹配 |
| 单元测试 — Orphaned 优先级 | ✅ | `test_all_priorities_are_unique` 覆盖 |
| 单元测试 — rebase 后 orphaned 场景 | ❌ 待实现 | 需要 git 仓库 fixture |

### 6.2 离线场景处理

| 任务 | 状态 | 说明 |
|------|------|------|
| `Submodule` 新增 `remote_unreachable: bool` | ❌ 待实现 | — |
| 远程不可达时跳过 Orphaned/BehindRemote 判定 | ❌ 待实现 | 当前 `find_reference` 失败时返回 `default`，但不标记 |
| UI 层"状态不确定"提示 | ❌ 待实现 | — |

### 6.3 AggregateStatus + health_check

| 任务 | 状态 | 实际实现 |
|------|------|----------|
| `AggregateStatus` 结构体（7 种状态计数 + total） | ❌ 待实现 | 当前 `RepoState` 只有 `total` / `clean_count` / `needs_attention` |
| `scan_all()` 返回 `(Vec<Submodule>, AggregateStatus)` | ❌ 待实现 | — |
| `health_check()` 派生自 scan_all | ✅ | `GitSubmoduleEditor::health_check()` |
| 建议操作文本 | ✅ | `describe_issue()` 覆盖全部 7 种状态 |
| CLI/Tauri 输出聚合统计 | ❌ 待实现 | 当前 CLI 只输出 clean_count |

---

## 标准合规对照

| 标准要求 | Iter 1-5 | Iter 6 完成 | Iter 6 剩余 |
|----------|----------|-------------|-------------|
| 7 种状态全部实现 | ⚠️ Orphaned 未赋值 | ✅ merge_base 检测 | — |
| 状态判定按优先级排序 | ✅ | — | — |
| `CommitHash` 独立类型 | ✅ | — | — |
| Orphaned 不提供自动收敛 | ⚠️ 检测缺失 | ✅ | — |
| 离线处理 | ❌ | — | `remote_unreachable` 标记 + 判定降级 + UI |
| AggregatedStatus | ⚠️ 部分 | — | `AggregateStatus` 结构体 + `scan_all()` + CLI/Tauri |
| health_check 派生自 scan_all | ❌ | ✅ | — |
| 原子操作 | ✅ | — | — |
| 模型/命令层分离 | ✅ | — | — |

---

## 时间线

```
Iter 0 ── Iter 1 ── Iter 2 ── Iter 3 ── Iter 4 ── Iter 5 ── Iter 6 (剩余)
0.5w       2w        2w        2w        2w        2w        按需
```

**Iteration 6 已完成**（`b6e4c8e`）。剩余唯一条目：
- 2.1 URL 可达性验证（低优先级，需网络请求）

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
