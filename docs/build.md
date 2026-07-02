# 构建状态模块

## 定位

`build.rs` 对应 roadmap 中 `build-command.md` 的蓝图。只读模式，不触发构建。

核心价值：按 scope 检查三件事——CI 结果、本地语法、版本一致性。

## 与四维契约模型的关系

`build.status()` 全过程依赖 `contract::load()`：

```
contract.yaml → Contract
                   ├── scopes[i].dir          → 确定检查哪个目录
                   ├── scopes[i].language      → 决定语法校验命令
                   ├── platforms.artifact_registry → 显示制品库
                   ├── stages.test.threshold   → 显示测试阈值
                   └── stages.release.changelog → 显示 CHANGELOG 路径
```

## 实现：`status()`

```rust
pub fn status(repo_path: &Path)
```

无返回值，直接打印格式化输出。

### 流程

1. 加载契约：`contract::load(repo_path)`
2. 无 scope：构造 root Scope，走 `contract::version_status()` + `contract::scope_release()`
3. 有 scope：遍历 scopes，调同一套接口

## 每 scope 三路检查

| 检查项 | 实现 | 说明 |
|--------|------|------|
| CI 状态 | `gh --version` 检测 | 不真正调 API，仅检查 CLI 是否可用 |
| 语法校验 | `cargo check` | 按 scope 目录找 `Cargo.toml`，目前只处理 Rust |
| 版本一致 | `contract::version_status()` | 复用契约模块的逻辑，不做二次实现 |

## 输出示例

```
构建状态
────────────────────────────────────────────────
  [cli]         Rust
    CI:         gh 可用（需配置）
    syntax:     ✅ cargo check 通过
    version:    ✅ 0.6.1（一致）
    registry:   crates.io
    changelog:  CHANGELOG.md

  工作区:       ✅ 干净
```

## 关键设计

### scope 复用 contract 模块

根 scope（无 `contract.yaml`）和命名 scope 都走同一套 `contract::version_status()` + `contract::scope_release()` 接口，不重复实现。

### 不做的

- 不触发构建（由 CI 自动执行）
- 不调 CI API：只检测 `gh` 是否存在。真实场景需调 `gh run list`
- 不做全量编译（`cargo check` 而非 `cargo build`）

## 经验教训

- `build.rs` 最初手写了 `version_status_root()`、`read_simple_version()`、`latest_tag_for_scope()` 三套重复逻辑。删除后全部改为调 `contract::` 接口，代码量减半且更健壮。
- root scope 需要手动构造一个同名 `Scope` 结构体才能复用 `contract::version_status()`。这是合理的——root 也是一个 scope，只是没写进 yaml。
- CI 状态用 `gh --version` 而非 `gh run list`：防止断网场景 panic，且减少对外部服务的依赖。
