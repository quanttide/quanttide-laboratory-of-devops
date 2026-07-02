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

## 关键设计

### 严重程度决定行为

| 严重程度 | 自动修复 | 人工介入 | 示例 |
|---------|---------|---------|------|
| Info | ✅ `sync` | 不需要 | AheadOfRemote 自动推送 |
| Warning | ❌ 需先手动提交 | 需要 | Dirty 要先 commit 或 stash |
| Error | ⚠ 部分可 | 需要 | Missing 可 `git submodule update --init` |

### 对偶性（OutOfSync vs Anomaly）

- **OutOfSync**：同步偏差——知道该怎么修（push / pull / commit），通常可自动处理。
- **Anomaly**：结构异常——不确定当前状态，需要人类判断（游离 HEAD、丢失的 commit、未知错误）。

## 经验教训

- 三分法最初来自 CLI 中 `code status` 的实践经验。旧版只有"同步/不同步"二分，无法区分"可自动修复"和"需要人类判断"——这是上线后用才知道的区别。
- `AnomalyKind::Unknown(String)` 兜底：任何未预料的 git 错误不会导致 panic，而是标记为 Unknown 并携带原始信息。
- 排序按严重程度：Error → Warning → Info。用户先看到需要处理的问题。
