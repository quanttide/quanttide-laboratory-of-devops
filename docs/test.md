# test

执行测试。

## 用法

```bash
qtcloud-devops-code test
qtcloud-devops-code test --name test_foo
```

## 参数

| 参数 | 说明 |
|------|------|
| `--name <pattern>` | 按名称过滤测试（传递给 `cargo test -- <pattern>`） |

## 输出

全部通过：

```
测试结果
----------------------------------------
  总数: 81
  通过: 81
  失败: 0
```

有失败用例时：

```
测试结果
----------------------------------------
  总数: 81
  通过: 79
  失败: 2

失败用例:
  test_broken_feature
  test_regression_case
```

## 内部行为

1. 检查当前目录是否存在 `Cargo.toml`（不存在则报错）
2. 执行 `cargo test`（指定 `--name` 时附加 `-- <pattern>`）
3. 扫描输出中 `... ok` 行计数为通过
4. 扫描输出中 `... FAILED` 行计数为失败，并提取测试名称
5. 如果 `cargo test` 返回非零退出码，视为整体失败
