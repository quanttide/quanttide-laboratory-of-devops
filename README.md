# 量潮DevOps实验室

`qtcloud-devops` CLI 的原型验证场。

## 快速开始

```bash
cargo run        # 演示所有模块
cargo test       # 运行全部 36+ 测试
```

## 模块

| 模块 | `cargo run` 演示 | `cargo test` 测试数 | 说明 |
|------|-----------------|--------------------|------|
| `contract` | scope 解析、语言检测 | 10 | contract.yaml 解析、Rust/Python/Go/Dart 语言识别、版本一致性 |
| `build` | 构建状态（CI、语法、版本） | 0 | 按 scope 输出构建状态 |
| `code` | 子模块扫描（无 .gitmodules 时跳过） | 11 | 三分法状态模型：Synchronized / OutOfSync / Anomaly |
| `test` | 测试状态（结果、覆盖率） | 6 | 测试报告解析、lcov 覆盖率解析 |
| `validate` | CHANGELOG + 版本一致性验证 | 7 | 包装 `qtcloud-devops-cli` 的 release API |
| `preflight` | 发布前检查（构建、测试、dry-run） | 2 | CI preflight 的 Rust 版本 |

## 开发原则

- **先在 lab 验证，再进 CLI**：所有 roadmap 命令先在这里实现、跑通、修复问题，再迁移到 `apps/qtcloud-devops/src/cli/`
- **依赖 crates.io 的 qtcloud-devops-cli**：不设路径依赖，模拟真实第三方使用场景
- **测试覆盖核心逻辑**：36 个测试覆盖状态模型、解析器、边界条件

## 依赖

- `qtcloud-devops-cli` — crates.io
- `serde` + `serde_yaml` — contract.yaml 解析
- `tempfile` — 测试用临时目录
