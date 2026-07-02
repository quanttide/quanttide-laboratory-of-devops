# 测试状态模块

## 定位

`test.rs` 对应 roadmap 中 `test-command.md` 的蓝图。按 scope 输出测试结果和覆盖率，检查是否达到阈值。

## 与四维契约模型的关系

| 维度 | 来源 | 说明 |
|------|------|------|
| `stages.test.threshold` | 全局默认 70.0 | 未定义 scope 覆盖时的兜底值 |
| `scopes[i].test_threshold` | scope 级覆盖 | 允许为不同组件设不同门槛 |
| `contract::scope_test_threshold()` | 便捷函数 | scope 有覆盖时用覆盖值，否则用全局默认 |

## 实现：`status()`

```rust
pub fn status(repo_path: &Path, c: &contract::Contract)
```

### 流程

1. 从 `contract::load_scopes()` 获取 scope 列表
2. 无 scope：检测语言，汇总测试，用全局 `c.stages.test.threshold` 作为覆盖率阈值
3. 有 scope：遍历 scopes，用 `contract::scope_test_threshold(c, &scope)` 获取各 scope 的阈值

## 测试结果解析

```rust
fn parse_test_summary(content: &str) -> TestSummary { total, passed, failed, skipped }
```

解析 `cargo test` 的输出行：

```
test result: ok. 10 passed; 0 failed; 2 ignored; 0 measured; 12 filtered out
```

按 `;` 分割后取每个片段末尾的 `(数字, kind)` 对。不依赖固定位置——容错性好，测试框架输出微调也不崩。

## 覆盖率解析

```rust
fn parse_lcov_coverage(content: &str) -> Option<f64>
```

解析 lcov.info 格式：

```
SF:src/lib.rs
DA:1,1
DA:2,0
end_of_record
```

覆盖率 = 命中行数 / 总行数（只统计 `DA:` 行，按行计数而非分支或函数）。

## 输出示例

```
测试状态
────────────────────────────────────────────────
  [(root)]      Rust
    测试数:       42 ✅ 全部通过
    覆盖率:       85.3%✅（阈值 70%）
```

## 阈值优先级

```
scope.test_threshold? → Some → 使用 scope 级 (如 90%)
                      → None → 使用 stages.test.threshold（全局默认 70%）
```

## 经验教训

- 最初硬编码 `threshold = 70.0`，所有项目一刀切。接入四维契约后从 `contract::scope_test_threshold()` 读取，不同组件可以设不同的门槛。
- lcov 覆盖率解析按行命中率计算（`命中行 / 总行数`），而非按分支/函数。足够做门禁检查，但不适合精确覆盖率分析。
- 测试结果解析用 `;` 分割而非正则，避免依赖 `test result:` 消息的精确格式。
