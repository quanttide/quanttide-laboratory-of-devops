# 架构

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
| CLI | Rust | 单机 DevOps 命令，增量发布、本地审计 | `apps/qtcloud-devops/src/cli/` |
| Provider | Go | 服务端能力：批量扫描、自动修复、跨仓库协调 | 待定 |
| 测试 | Python | 集成测试编排，用 `subprocess` 调 CLI、HTTP 调 provider | 无（仅实验室） |
| Packages | 待定 | 共享逻辑，CLI 和 provider 共用的数据模型 | `packages/quanttide-devops-toolkit/` |

## 分界线

| 场景 | 归属 | 原因 |
|------|------|------|
| 增量发布、本地审计 | CLI | 只需看当前工作区，延迟敏感 |
| 批量 artifact 扫描修复 | Provider | 需要全局视角和持久状态 |
| 跨仓库协调 | Provider | 需要编排多个仓库 |
| 审计日志持久化 | Provider | 需要存储 |
| 集成测试 | Python | pytest 编排最灵活 |

## 数据流

```
AI / 开发者 → CLI（本地操作）→ GitHub（tag/release）
                              ↕
                   Provider（后台扫描）→ 发现不一致 → 自动修复
                              ↕
                   Python 测试 ← 模拟各种绕过场景 → 验证收敛
```
