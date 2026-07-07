# 发布流程设计

## 两种发布场景

### 增量发布（预发布系列）

有真实新提交时的常规发布。工作流：

```
git log（新提交）→ 版本 bump → 追加 CHANGELOG → tag → Release
```

这是 CLI 擅长且正确处理的场景。`collect_git_log` 以最新 tag 为基准，扫描 tag→HEAD 的新提交。

### 汇总发布（正式版）

预发布系列（alpha → beta → rc）完成后的阶段晋级。工作流：

```
预发布 CHANGELOG 条目 → 汇总合并 → 版本晋级 → tag → Release
```

**关键区别**：正式版 CHANGELOG 的内容来源**不是 git log**，而是**预发布周期内已写入 CHANGELOG 的条目**。因为预发布周期的所有变更已经被 CHANGELOG 记录过，git log 只是这些内容的原材料。

## 当前缺陷

`release publish` 以最新预发布 tag 为比较基准，而不是上一个正式版。这导致：

- 如果预发布周期无新提交 → `collect_git_log` 返回空 → CHANGELOG 为空 → 发布失败
- 深层问题：工具设计假设**每次发布之间都有新提交**，但正式版发布恰恰是例外

## 修复方向

### 命令调整

| 场景 | 比较基准 | CHANGELOG 来源 | 命令 |
|------|---------|---------------|------|
| 预发布 | 上一个同 scope tag | git log 增量 | `release publish --pre` |
| 正式版 | 上一个正式版 tag | 预发布 CHANGELOG 汇总 | `release publish --formal` |

### Audit 扩展

`release audit` 应检查：

1. **基础检查**（当前已有）：版本号、配置文件、工作区、标签冲突、远程可达性、Release 存在性
2. **一致性检查**（待实现）：Tag/CHANGELOG/Release 三角关系
3. **汇总完整性检查**（待实现）：正式版发布前，确认预发布周期的所有 CHANGELOG 条目已完整

## 本地 vs 云端

根据 `insight/stage/release-publish.md` 的分析：

### 保留在本地

- 增量发布——延迟敏感，不需要网络
- 发布前检查——6+1 项中的大部分可在本地完成
- 开发者确认交互——默认交互式确认

### 迁移到云端

- 批量修复——AI 绕过后产生的海量 artifact 不一致
- 跨仓库协调——父仓库子模块指针更新、关联仓库版本依赖更新
- 审计日志与可观测性——持久化记录谁绕过了流程、如何修复的
- LLM 调用——统一管理 API key、缓存结果、控制成本
