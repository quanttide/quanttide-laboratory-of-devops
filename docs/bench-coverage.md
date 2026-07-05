# 覆盖率工具性能对比实验

比较 cargo-llvm-cov、cargo-tarpaulin、grcov、Rust 内置四种方案的性能。

## 被测工具

| 工具 | 原理 | 安装方式 | 预期特点 |
|------|------|---------|---------|
| **cargo-llvm-cov**（当前） | LLVM 插桩编译 | `cargo install` | 快，但编译插桩时可能崩 |
| **cargo-tarpaulin** | ptrace 运行时追踪 | `cargo install` | 慢但稳定，不参与编译 |
| **grcov** | 解析编译产物 | `cargo install` + `-Cinstrument-coverage` | 后处理，不影响编译 |
| **Rust 内置** | llvm-tools + profdata | 组件安装 `rustup component add llvm-tools-preview` | 零额外依赖 |

## 指标

- **耗时** — 首次运行（clean build）和增量运行（warm build）的 wall time
- **内存峰值** — 运行过程中的最大内存占用
- **成功率** — 是否崩溃、segfault、OOM
- **覆盖率** — 报告的行覆盖率百分比（验证结果一致性）

## 运行

```bash
bash docs/bench-coverage.sh              # 全部工具
bash docs/bench-coverage.sh llvm-cov     # 单个工具
```

结果输出到 `docs/bench-results/summary.csv`。
