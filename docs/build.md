# 构建状态模块

## 定位

`build.rs` 对应 roadmap 中 `build-command.md` 的蓝图。只读模式，不触发构建。

核心价值：按 scope 检查三件事——CI 结果、本地语法、版本一致性。三路检查合在一起构成完整的"构建状态"。

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

## 三路检查

`build status` 不做本机构建。真正的构建由 CI 完成，`build status` 展示构建管线的整体状态——从三个维度覆盖：

| 检查 | 功能 | 数据来源 | 回答的问题 |
|------|------|---------|-----------|
| **CI 状态** | 展示 CI 最近一次构建结果 | `gh run list` | "CI 上构建通过了吗？" |
| **语法校验** | 本地快速检查能否编译 | `cargo check` / `uv check` 等 | "本地代码有语法/类型错误吗？" |
| **版本一致** | tag 版本 vs 配置文件版本 | `contract::version_status()` | "待发布的版本号对齐了吗？" |

三路检查回答了三个不同的问题，合在一起才是完整的"构建状态"。

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

### 为什么不做本机构建

| 理由 | 说明 |
|------|------|
| **CI 才是真相** | 本地编译通过不代表 CI 能过（环境差异、依赖版本）。CI 的构建结果才是门禁标准。 |
| **耗时不对称** | `cargo check` 秒级，`cargo build` 分钟级。`status` 是只读诊断命令，不应让用户等。 |
| **只读语义** | `status` 不修改文件系统。`cargo build` 会产生构建产物，`cargo check` 只读。 |
| **各语言差异大** | Rust 有 `cargo check`，Go 有 `go vet`，但 Python/JS 没有等价的"编译但不产生产物"命令。强行统一为"构建"会破坏一致性。 |

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
