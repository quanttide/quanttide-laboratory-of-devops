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

| 任务 | 说明 |
|------|------|
| 实现 `is_orphaned()` 函数 | 检查 `parent_pointer` 在远程是否仍存在（通过 `revwalk` 或 `odb.exists()`） |
| 插入判定分支 | 在 `Dirty` 之后、`Detached` 之前插入 `Orphaned` 判定 |
| 单元测试 | 模拟远程分支 rebase 后被删除的场景 |

### 6.2 离线场景处理

| 任务 | 说明 |
|------|------|
| `Submodule` 新增 `remote_unreachable: bool` | 标记远程是否可达 |
| 远程不可达时跳过 Orphaned 判定 | 不将子模块误报为 Orphaned |
| 远程不可达时跳过 BehindRemote 判定 | 不将子模块误报为 BehindRemote |
| 结果中返回 `remote_unreachable` 标识 | UI 层据此展示"状态不确定"提示 |

### 6.3 AggregateStatus + health_check

| 任务 | 说明 |
|------|------|
| 定义 `AggregateStatus` 结构体 | 包含全部 7 种状态的计数 + total |
| 实现 `scan_all()` | 返回 `(Vec<Submodule>, AggregateStatus)` |
| 实现 `health_check()` | 过滤 `status != Clean` 的子模块，附上建议操作 |
| 明确为 `scan_all` 的派生视图 | 不引入独立的状态判定逻辑 |

---

## 标准合规对照

| 标准要求 | Iter 1-5 | Iter 6 |
|----------|----------|--------|
| 7 种状态全部实现 | ⚠️ Orphaned 在枚举中但未被赋值 | ✅ |
| 状态判定按优先级排序 | ✅ | — |
| `CommitHash` 独立类型 | ✅ | — |
| Orphaned 不提供自动收敛 | ⚠️ 检测缺失 | ✅ |
| 离线处理 | ❌ | ✅ |
| AggregatedStatus | ⚠️ 部分 | ✅ |
| health_check 为 scan_all 派生视图 | ❌ | ✅ |
| 原子操作 | ✅ | — |
| 模型/命令层分离 | ✅ | — |

---

## 时间线

```
Iter 0 ── Iter 1 ── Iter 2 ── Iter 3 ── Iter 4 ── Iter 5 ── Iter 6
0.5w       2w        2w        2w        2w        2w        1w
```

详细开发蓝图见 [docs/dev.md](docs/dev.md)。
