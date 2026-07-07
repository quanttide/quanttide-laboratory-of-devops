# Provider 原型开发

Provider 侧验证的是 CLI 无法覆盖的能力：批量扫描、自动修复、跨仓库协调。

## 核心模型

### Artifact 三角

每个发布的完整性由三个 artifact 共同决定：

```
Tag ← 事实源，不可移动
 ↓
CHANGELOG ← 规范事实源，派生制品
 ↓
GitHub Release ← 派生制品，可重建
```

扫描结果为三者状态的组合：

| CHANGELOG | Tag | Release | 判定 |
|-----------|-----|---------|------|
| ✅ | ✅ | ✅ | 正常 |
| ❌ | ✅ | ✅ | 缺 CHANGELOG |
| ✅ | ✅ | ❌ | 缺 Release |
| ✅ | ❌ | ❌ | 未发布 |
| ❌ | ✅ | ❌ | 只有 tag |

### 修复规则

| 问题 | 修复方式 | 事实源 |
|------|---------|--------|
| 缺 Release | `gh release create` 从 CHANGELOG 补 | CHANGELOG |
| 缺 CHANGELOG | 从 git log 补写 | git log |
| 缺 tag | 不可自动修复，标记搁置 | — |

## 扫描接口

```go
type ArtifactState struct {
    Scope     string
    Version   string
    HasTag    bool
    HasChangelog bool
    HasRelease   bool
}

type ScanResult struct {
    Artifacts []ArtifactState
    Summary   ScanSummary
}
```

## 模拟场景

见 AGENTS.md 的模拟场景列表。
