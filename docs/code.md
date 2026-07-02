# 代码状态模块（子模块三分法）

## 定位

`code.rs` 对应 CLI 中 `code status` 命令的数据模型。核心是**子模块状态三分法**——将子模块的健康状态归为三类，对应不同的处理策略。

## 状态三分法

```
SubmoduleStatus
  ├── Synchronized    → Info, 无需操作
  ├── OutOfSync       → Info/Warning/Error, 可自动修复
  │    ├── AheadOfRemote    Info
  │    ├── BehindRemote     Info
  │    ├── Dirty            Warning
  │    └── Diverged         Error
  └── Anomaly         → Error, 需人工介入
       ├── DetachedHead
       ├── Orphaned
       ├── Missing          可自动修复
       ├── Unregistered
       └── Unknown(msg)
```

## 与四维契约模型的关系

`code.rs` 不直接依赖 contract 模块。子模块状态检测是纯 git 操作，不涉及契约配置。

这意味着：
- `code` 是唯一在所有仓库中行为一致的模块——无论项目是 Rust/Python/Go、单仓/monorepo，检测逻辑不变
- 这也是 roadmap 中 `contract` 模块未来可能扩展的方向：将子模块的预期状态（期望的 branch、是否允许 dirty）纳入契约

## 严重程度设计

```rust
pub enum Severity { Info, Warning, Error }

impl PartialOrd for Severity  // Error > Warning > Info
```

| 严重程度 | 自动修复 | 人工介入 | 示例 |
|---------|---------|---------|------|
| Info | ✅ `code sync` 自动处理 | 不需要 | AheadOfRemote 自动推送 |
| Warning | ❌ 需先手动提交 | 需要 | Dirty 要先 commit 或 stash |
| Error | ⚠ 部分可（Missing） | 需要 | Diverged 需要手动 merge |

问题列表按严重程度排序输出（Error 优先），用户先看到需要处理的问题。

## 对偶性设计

- **OutOfSync**：**同步偏差**。知道应该做什么（push / pull / commit），通常可自动修复。
- **Anomaly**：**结构异常**。不确定当前状态，需要人类判断。

## 检测函数

```rust
pub fn scan_submodules(repo_path: &Path) -> Vec<HealthIssue>
```

1. 读 `.gitmodules` 解析子模块列表（`[submodule "name"]` + `path = ...`）
2. 对每个子模块调用 `check_submodule()`：
   - 目录不存在 → `AnomalyKind::Missing`
   - `.git` 不存在 → `AnomalyKind::Unknown("不是 git 仓库")`
   - `git status --porcelain` 非空 → `OutOfSyncKind::Dirty`
   - `git rev-list --left-right --count HEAD...origin/main` 解析 ahead/behind → Diverged / AheadOfRemote / BehindRemote
   - 全过 → `Synchronized`
3. 无 `.gitmodules` 时返回空列表，不做额外扫描

## 核心模型

```rust
pub enum SubmoduleStatus {
    Synchronized,
    OutOfSync(OutOfSyncKind),
    Anomaly(AnomalyKind),
}

pub enum OutOfSyncKind {
    AheadOfRemote,   // 有本地提交未推送
    BehindRemote,    // 远程有新提交未拉取
    Dirty,           // 工作区有未提交变更
    Diverged,        // 本地与远程历史分叉
}

pub enum AnomalyKind {
    DetachedHead,    // HEAD 游离
    Orphaned,        // 父仓库记录的 commit 在远程不存在
    Missing,         // 子模块目录不存在
    Unregistered,    // 存在但未在 .gitmodules 注册
    Unknown(String), // 未知错误
}
```

## 诊断函数

```rust
pub fn diagnose(submodule: &str, status: &SubmoduleStatus) -> HealthIssue
```

将状态模型转为用户友好的诊断报告，包含 `severity`、`description`、`suggested_action`、`auto_fixable`。

## 经验教训

- 三分法最初来自 CLI 中 `code status` 的实践经验。旧版只有"同步/不同步"二分，无法区分"可自动修复"和"需要人类判断"——这是上线后用才知道的区别。
- `AnomalyKind::Unknown(String)` 兜底：任何未预料的 git 错误不会导致 panic，而是标记为 Unknown 并携带原始信息。
- 排序按严重程度：Error → Warning → Info。用户先看到需要处理的问题。
