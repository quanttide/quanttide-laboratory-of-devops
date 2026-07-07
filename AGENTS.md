# AGENTS.md

## 项目定位

实验室是 CLI 功能和 Provider 服务的原型验证场。所有 roadmap 上的功能先在这里验证，再推进到对应的产品仓库。

## 三线结构

```
examples/default/
├── pyproject.toml    # Python — 测试编排、集成测试
├── tests/            # pytest 集成测试
├── src/              # Go — 服务端/provider 原型
│   ├── go.mod
│   └── main.go
└── cli/              # Rust — CLI 原型
    ├── Cargo.toml
    └── src/
```

### 测试编排（Python / 根目录）

`pytest` 写集成测试，用 `subprocess` 调 CLI、HTTP 请求调 provider。

### Provider 原型（Go / src/）

服务端能力原型验证：
- artifact 完整性扫描与批量修复
- 跨仓库发布协调
- 审计日志持久化
- 需要全局视角或持久状态的操作

### CLI 原型（Rust / cli/）

CLI 侧的原型验证继续遵循现有模式：先在 lab 验证，再迁移到 `apps/qtcloud-devops/src/cli/`。

依赖 crates.io 的 `qtcloud-devops-cli`，不设路径依赖，模拟真实第三方使用场景。

## 分界线

| 场景 | 归属 |
|------|------|
| 只需看当前工作区就能决定的 | CLI 侧 |
| 需要全局视角/持久状态的 | Provider 侧 |
| 一次操作就能修复的 | CLI 侧 |
| 需要批量收敛大量不一致的 | Provider 侧 |
