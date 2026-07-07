# 架构：三线结构 + 约束模型

## 三线结构

```
docs/              # 文档
packages/          # 共享库原型
src/
├── cli/           # CLI 原型（Rust）
└── provider/      # Provider 原型（Go）
tests/             # 集成测试（Python/pytest）
```

| 层 | 语言 | 职责 | 对应生产仓库 |
|------|------|------|-------------|
| **CLI** | Rust | 单机 DevOps 命令，增量发布、本地审计 | `apps/qtcloud-devops/src/cli/` |
| **Provider** | Go | 服务端能力：批量扫描、自动修复、跨仓库协调 | 待定 |
| **测试** | Python | 集成测试编排，`subprocess` 调 CLI、HTTP 调 provider | 无（仅实验室） |
| **Packages** | 待定 | 共享逻辑，CLI 和 provider 共用的数据模型 | `packages/quanttide-devops-toolkit/` |

## 约束模型

当前架构的核心矛盾：

```
行为约束（旧思路）: CLI 命令 → 流程步骤 → 假设会遵守 → ❌ AI 绕过
结果约束（新思路）: 任意操作 → 高频扫描 → 发现不一致 → 自动收敛
```

新的架构设计围绕后者展开：

```
AI / 开发者 → CLI（本地操作）→ GitHub（tag/release）
                              ↕
                   Provider（后台高频扫描）→ 发现不一致 → 自动修复
                              ↕
                   反脆弱策略层 ← 状态空间判定 → 不可修复 → 标记搁置
```

## 数据流

```
CLI 创建: CHANGELOG → tag → Release
AI 绕过: tag → Release（缺 CHANGELOG）
Provider: 扫描发现断裂 → 从 git log 补 CHANGELOG → 状态收敛
```

## 架构决策

### devops vs code 边界

| 层级 | 工具 | 做什么 | 判断标准 |
|------|------|--------|----------|
| **门禁** | `qtcloud-devops code audit` | 文本级统计 | 红/绿，CI 阻断 |
| **诊断** | `qtcloud-code review` | AST 级分析 | 精确到行号，给出修复建议 |

边界线：**是否需要 parser**。devops 的新指标采纳门槛是"能否在不引入 tree-sitter 的前提下实现"。

### git 库分工

| 操作类型 | 使用 | 示例 |
|----------|------|------|
| 只读查询（本地） | `gix`（优先） | 读 remote URL、查配置、遍历引用 |
| 本地写入 | `git2` | 创建本地 tag、删除本地引用 |
| 网络操作 | `git` CLI | push、fetch、pull、rebase、clone |

## 本地 vs 云端分界线

| 场景 | 归属 | 原因 |
|------|------|------|
| 增量发布、本地审计 | CLI | 只需看当前工作区，延迟敏感 |
| 批量 artifact 扫描修复 | Provider | 需要全局视角和持久状态 |
| 跨仓库协调 | Provider | 需要编排多个仓库 |
| 审计日志持久化 | Provider | 需要存储 |
| LLM 调用（CHANGELOG 汇总、版本决策） | Provider | 统一管理 API key、缓存、成本 |
| 集成测试 | Python | pytest 编排最灵活 |
