# 计划阶段设计

## 意图

让 AI 接管日常维护工作。流程：

```
人类发现/收集问题 → 更新 ROADMAP 和 TODO → doctor/audit 修复和审计 → 进入编码
```

收集可以发生在各个阶段（code review、build、test、release），不一定在专门的计划阶段。然后在计划阶段统一整理清楚。

## 命令参考

| 命令 | 用途 | 典型输出 |
|------|------|---------|
| `plan status [scope]` | 查看各版本完成进度 | 版本号 + 完成数/总数 + 百分比 |
| `plan clean [scope]` | 删除已完成条目 | 提示 + 级联清理空分类/空版本 |
| `plan doctor [scope]` | 修复格式问题 | LLM 修复 + 规则修复（v 前缀、大小写、checkbox） |
| `plan audit [scope]` | 检查规划文件结构问题 | 路径检查/粒度检查/孤儿检查 |

## 已知问题

来自 `insight/stage/plan.md` 的巡视记录：

### 1. ROADMAP 状态脱节

已实现的条目（约 4 项）在 ROADMAP 中仍标记 `[ ]`：

- `plan clean` 已支持同时清理 ROADMAP 和 TODO
- `plan audit` 已实现三项结构检查
- `plan doctor` LLM prompt 已覆盖 ROADMAP + TODO
- 已补充 `release/audit.rs`、`contract.rs` 测试

**根因**：缺少 TODO [x] → ROADMAP [x] 的自动同步步骤。

### 2. 工具缺陷

| 缺陷 | 状态 |
|------|------|
| `extract_line_paths` 不支持 `:N` 行号 | ✅ 已修 |
| `CATEGORIES` 缺少 `### Refactor` | ✅ 已修 |
| `plan audit` 不支持 scope — 无法审计子模块内规划文件 | ❌ 未修 |
| `determine_submodule_status` 长参数重构不完整 | ❌ 需重新做 |
| ROADMAP 孤儿条目（2 个） | ❌ 未修 |

### 3. 缺少统一的问题收集入口

当前只有 `qtcloud-code review` 一个输入源。build/test/release 等阶段的发现缺乏集中收集到规划文件的机制。

## ROADMAP 格式约定

```
# ROADMAP

## [0.2.0] — 进行中

### Added
- [x] 已完成功能 A
- [ ] 待办功能 B
```

规则：
- **首行**必须为 `# ROADMAP`
- **版本头**格式：`## [X.Y.Z]` 或 `## [X.Y.Z] — 状态`
- **分类**：`### Added` / `### Changed` / `### Fixed` / `### Removed` / `### Deprecated` / `### Security`
- **条目**：`- [x]` 已完成 / `- [ ]` 待办
