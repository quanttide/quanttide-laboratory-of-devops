# 架构决策记录（ADR）

## ADR-001: devops vs code 边界

- **状态**: 已采纳
- **日期**: 2026-07-06

### 背景

`qtcloud-devops`（DevOps 生命周期协调）和 `qtcloud-code`（代码静态分析）都能做代码审计。需要明确边界以避免重复造轮子和依赖膨胀。

### 决策

| 层级 | 工具 | 做什么 | 判断标准 |
|------|------|--------|----------|
| **门禁** | `qtcloud-devops code audit` | 文本级统计 | 红/绿，CI 阻断 |
| **诊断** | `qtcloud-code review` | AST 级分析 | 精确到行号，给出修复建议 |

边界线：**是否需要 parser**。devops 的新指标采纳门槛是"能否在不引入 tree-sitter 的前提下实现"。

联动：`qtcloud-code review . --status` 输出 `STATUS.md`，`qtcloud-devops code audit` 读到就聚合展示。两者独立发布、独立演进。

## ADR-002: git 库使用范围

- **状态**: 已采纳
- **日期**: 2026-07-06

### 背景

项目依赖两个 git 库（`gix` 和 `git2`）以及系统 `git` 命令，三者在功能上有重叠。需要明确各自的使用范围。

### 决策

| 操作类型 | 使用 | 示例 |
|----------|------|------|
| 只读查询（本地） | `gix`（优先） | 读 remote URL、查配置、遍历引用 |
| 本地写入 | `git2` | 创建本地 tag、删除本地引用 |
| 网络操作 | `git` CLI | push、fetch、pull、rebase、clone |

原则：
1. `gix` 优先用于只读操作——纯 Rust，无 C 依赖，编译快
2. `git2` 只做本地写入——API 成熟，但避免网络认证问题
3. 网络操作一律走系统 `git` CLI——通过 `gh auth setup-git` 配置的 credential helper 正常认证

## 待决策

以下问题在 `data/insight` 中被提出，但尚未形成正式决策：

| 问题 | 来源 |
|------|------|
| 如果 AI 可以绕过 publish，那 publish 存在的意义是什么？ | release-audit 巡视 |
| 是否应该让 publish 成为创建 CHANGELOG/tag/Release 的唯一入口？ | release-audit 巡视 |
| 是否所有阶段晋级（alpha→beta→rc→正式）都应该从已有 CHANGELOG 汇总？ | release-publish 巡视 |
