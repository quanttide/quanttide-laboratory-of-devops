# AGENTS.md

## 项目定位

实验室是 CLI 功能和 Provider 服务的原型验证场。所有 roadmap 上的功能先在这里验证，再推进到对应的产品仓库。

## 双线结构

```
examples/default/
├── Cargo.toml        — workspace root
├── cli/              — CLI 原型（对应 apps/qtcloud-devops/src/cli）
│   ├── Cargo.toml
│   └── src/
└── provider/         — 服务端原型（对应未来服务端组件）
    ├── Cargo.toml
    └── src/
```

### CLI 线

CLI 侧的原型验证继续遵循现有模式：命令先在 lab 的 `cli/` 包中实现、跑通、发现问题，再迁移到 `apps/qtcloud-devops/src/cli/`。依赖 crates.io 的 `qtcloud-devops-cli`，不设路径依赖。

### Provider 线

Provider 侧的原型验证用于：
- artifact 完整性扫描与批量修复
- 跨仓库发布协调
- 审计日志持久化
- 需要全局视角或持久状态的操作

这些功能不适合 CLI 的单机模型，需要在 lab 中以服务端形态验证设计，再决定是融入 `packages/quanttide-devops-toolkit` 还是独立为新的服务组件。

## 依赖策略

| 依赖 | CLI 线 | Provider 线 |
|------|--------|------------|
| `qtcloud-devops-cli` | crates.io | 不直接依赖 |
| `quanttide-devops-toolkit` | crates.io | 路径依赖，本地开发 |
| `serde` / `serde_yaml` | 通用 | 通用 |
| `actix-web` / `axum` | 不需要 | 可能用于服务端 |

## 分界线

| 场景 | 归属 |
|------|------|
| 只需看当前工作区就能决定的 | CLI 侧 |
| 需要全局视角/持久状态的 | Provider 侧 |
| 一次操作就能修复的 | CLI 侧 |
| 需要批量收敛大量不一致的 | Provider 侧 |
