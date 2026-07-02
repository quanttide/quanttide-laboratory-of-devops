# 测试状态模块

## 定位

`test.rs` 对应 roadmap 中 `test-command.md` 的蓝图。按 scope 输出测试结果和覆盖率，检查是否达到阈值。

## 与四维契约模型的关系

| 维度 | 来源 | 说明 |
|------|------|------|
| `stages.test.threshold` | 全局默认 70.0 | 未定义 scope 覆盖时的兜底值 |
| `scopes[i].test_threshold` | scope 级覆盖 | 允许为不同组件设不同门槛 |
| `contract::scope_test_threshold()` | 便捷函数 | scope 有覆盖时用覆盖值，否则用全局默认 |

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
scope.test_threshold? → Some → 使用 scope 级
                      → None → 使用 stages.test.threshold（全局默认）
```

## 经验教训

- 最初硬编码 `threshold = 70.0`，所有项目一刀切。接入四维契约后从 `contract::scope_test_threshold()` 读取，不同组件可以设不同的门槛。
- lcov 覆盖率解析按行命中率计算（`命中行 / 总行数`），而非按分支/函数。足够做门禁检查，但不适合精确覆盖率分析。
