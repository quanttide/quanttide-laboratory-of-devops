# build

执行项目构建。

## 用法

```bash
qtcloud-devops-code build
qtcloud-devops-code build --release
```

## 参数

| 参数 | 说明 |
|------|------|
| `--release` | release 模式构建 |

## 输出

构建成功：

```
构建中...
构建成功 (221ms)
```

构建失败（输出前 10 行错误）：

```
构建中...
构建失败:
  error[E0308]: mismatched types
    --> src/main.rs:42:8
     |
  42 |     let x: i32 = "hello";
     |            ^^^ expected i32, found &str
```

## 内部行为

1. 检查当前目录是否存在 `Cargo.toml`（不存在则报错）
2. 执行 `cargo build`（`--release` 时附加 `--release` 参数）
3. 计时
4. 成功时输出耗时，失败时截取前 10 行 stderr
