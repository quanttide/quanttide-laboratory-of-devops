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
| `--name <pattern>` | 按名称过滤测试 |

## 输出

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
