# 量潮DevOps实验室

`qtcloud-devops` CLI 的原型验证场。已集成的模块已清理，只保留实验性模块。

## 快速开始

```bash
cargo run --bin quanttide-lab      # 演示 preflight + release
cargo run --bin detect -- <path>   # 版本号自动检测
cargo test                         # 运行全部测试
```

## 实验模块

| 模块 | 说明 | 状态 |
|------|------|------|
| `bin/detect` | 版本号自动检测 — 从 git 历史推断版本 | 原型 |
| `preflight` | 发布前检查 — build → test → dry-run | 原型 |
| `release` | 发布流程编排 — precheck → publish → postcheck | 原型 |

详细用法见 [docs/README.md](docs/README.md)。

## 开发原则

- **先在 lab 验证，再进 CLI**：实验性功能先在 lab 验证，再迁移到 `apps/qtcloud-devops/src/cli/`
- **依赖 crates.io 的 qtcloud-devops-cli**：不设路径依赖，模拟真实第三方使用场景

## 依赖

- `git2` — git 仓库操作
- `qtcloud-devops-cli` — crates.io
- `serde` + `serde_yaml` — YAML 解析
- `tempfile` — 测试用临时目录
