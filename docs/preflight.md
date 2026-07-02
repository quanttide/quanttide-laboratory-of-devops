# 发布前检查模块

## 定位

`preflight.rs` 对应 `scripts/preflight.sh`，在发布前依次执行：版本状态检查 → 构建校验 → 测试校验 → dry-run。

## 与四维契约模型的关系

```rust
contract::version_status(repo_path, &scope)
  ├── tag_version:    最新 git tag 的版本号
  ├── config_version: Cargo.toml / pyproject.toml 中的版本号
  └── consistent:    两者是否一致
```

版本一致性是 Sources 维度的核心职责——"Git tag 是发布的事实源"，preflight 确保 tag 与配置文件对齐。

## 四步检查

| 步骤 | 命令 | 失败影响 |
|------|------|---------|
| 版本检查 | `contract::version_status()` | 输出版本不一致警告，不阻断 |
| 构建 | `cargo check` | 阻断 |
| 测试 | `cargo test` | 阻断 |
| dry-run | `cargo metadata` | 阻断 |

## 经验教训

- 最初自实现 `read_self_version()` 解析 Cargo.toml，与 `contract::read_config_version()` 逻辑重复。改用 `contract::version_status()` 后，版本从多个来源（tag + 配置文件）交叉验证，更健壮。
- preflight 不阻断版本不一致（只警告）。因为 tag 落后配置文件是常见开发状态，真正阻断的是构建/测试失败。
