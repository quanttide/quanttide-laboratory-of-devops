# 量潮 DevOps 实验室

当前实验模块：

| 模块 | 说明 | 状态 |
|------|------|------|
| `bin/detect` | 版本号自动检测 — 从 git 历史推断 scope/minor/patch/预发布 | [docs/detect.md](docs/detect.md) | 原型 |
| `preflight` | 发布前检查 — 依次执行 build → test → dry-run | — | 原型 |
| `release` | 发布流程编排 — 封装 precheck → publish → postcheck | — | 原型 |

## preflight — 发布前检查

```bash
cargo run --bin quanttide-lab
```

依次执行：
1. `cargo check` — 语法校验
2. `cargo test` — 运行测试
3. `cargo metadata` — 发布 dry-run（代替 `cargo publish --dry-run`）

## release — 发布流程编排

```bash
cargo run --bin quanttide-lab -- release <status|precheck|publish>
```

封装 `qtcloud_devops_cli::release` 的 precheck → publish → status 三步流程。
