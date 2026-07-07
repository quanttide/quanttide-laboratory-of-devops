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

## 生产映射

实验室的代码结构对应实际产品仓库，用于预演生产环境可能遇到的问题：

| 实验室路径 | 对应产品仓库 | 说明 |
|-----------|-------------|------|
| `cli/` | `apps/qtcloud-devops/src/cli/` | CLI 原型，验证后直接迁移 |
| `src/`（Go） | 待定（未来 provider 服务） | 服务端能力验证，技术选型确认后确定仓库 |
| `tests/`（Python） | 无对应产品仓库 | 集成测试编排层，仅实验室存在 |

### 模拟场景

- **CI 断裂** — provider 不可用时，CLI 能否独立完成增量发布？
- **artifact 不一致** — 模拟有人绕过 CLI 直接操作 GitHub API 后的修复
- **跨仓库发布** — 子模块发布后父仓库指针更新的协调延迟
- **高并发扫描** — provider 同时扫描 20+ scope 的 artifact 一致性时的性能
- **网络分区** — 发布中网络中断，部分 artifact 已创建部分未创建

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
