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

## 实现：`preflight()`

```rust
pub fn preflight(repo_path: &Path, _contract: &contract::Contract) -> PreflightResult
```

返回值：

```rust
pub struct PreflightResult {
    pub build_ok: bool,
    pub test_ok: bool,
    pub dry_run_ok: bool,
    pub version: String,
}
```

## 四步检查

| 步骤 | 实现 | 失败是否阻断 |
|------|------|-------------|
| 版本检查 | `contract::version_status()` 遍历 scopes | ❌ 只警告，不阻断。tag 落后配置文件是常见开发状态 |
| 构建 | `cargo check` | ✅ 阻断。编译不过不能发布 |
| 测试 | `cargo test` | ✅ 阻断。测试不过不能发布 |
| dry-run | `cargo metadata --no-deps` | ✅ 阻断。元数据错误意味着包配置有问题 |

## 输出示例

```
preflight
  (root): 0.1.0 ✅ tag:0.1.0 = 配置:0.1.0

--- cargo build ---  ✅
--- cargo test ---  42 passed; 0 failed; 0 ignored
--- cargo metadata --no-deps ---  ✅（metadata 检查通过）

preflight passed
```

## 经验教训

- 最初自实现 `read_self_version()` 解析 Cargo.toml，与 `contract::read_config_version()` 逻辑重复。改用 `contract::version_status()` 后，版本从多个来源（tag + 配置文件）交叉验证，更健壮。
- 版本不一致不阻断：因为开发阶段 tag 落后配置文件是常态。真正阻断的是构建/测试失败。
- dry-run 简化为 metadata 检查：真正发布时才需要 `cargo publish --dry-run`（涉及网络），preflight 只检查元数据能否正常解析。
- 无 `contract.yaml` 也正常工作：构造一个 root scope 遍历，测试时传入默认 Contract。
