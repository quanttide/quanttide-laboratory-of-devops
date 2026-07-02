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

## 三路检查

| 检查项 | 数据来源 | 说明 |
|--------|---------|------|
| CI 状态 | `gh run list` | 通过 GitHub CLI 查最近一次 CI 结果 |
| 语法校验 | `cargo check` / `go vet` 等 | 根据语言调对应本地命令 |
| 版本一致性 | `contract::version_status()` | 最新 tag vs 配置文件版本号 |

## 关键设计

### scope 复用 contract 模块

根 scope（无 `contract.yaml`）和命名 scope 都走同一套 `contract::version_status()` + `contract::scope_release()` 接口，不重复实现。

### 不做的

- 不触发构建（由 CI 自动执行）
- 不做全量编译（`cargo check` 而非 `cargo build`）

## 经验教训

- `build.rs` 最初手写了 `version_status_root()`、`read_simple_version()`、`latest_tag_for_scope()` 三套重复逻辑。删除后全部改为调 `contract::` 接口，代码量减半且更健壮。
- root scope 需要手动构造一个同名 `Scope` 结构体才能复用 `contract::version_status()`。这是合理的——root 也是一个 scope，只是没写进 yaml。
