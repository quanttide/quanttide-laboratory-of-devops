# AGENTS.md

## 项目定位

实验室是 CLI 功能的原型验证场。所有 roadmap 上的命令先在这里实现、跑通、发现问题，再推进到 `apps/qtcloud-devops/src/cli/`。

## 当前模块

```
src/
├── main.rs        — 各模块演示入口
├── contract.rs    — scope 解析（YAML）、语言检测、版本状态
├── build.rs       — build status（CI、语法、版本一致性）
├── code.rs        — code status（子模块三分法状态模型）
├── test.rs        — test status（测试结果、覆盖率）
├── validate.rs    — CI 验证（CHANGELOG、版本一致性）
└── preflight.rs   — 发布前检查（构建、测试、dry-run）
```

## 开发经验记录

### 公共能力先提取，命令只是薄层

`contract` 模块一旦写好（scope 解析、语言检测、版本状态），`build status`、`test status` 都直接复用。不用每个命令各自轮一遍 `contract.yaml` 怎么读、`Cargo.toml` 版本怎么取。

### 只读命令（status）比写入命令（publish）好设计得多

`build status`、`test status`、`code status` 都是读现有数据，出错不破坏东西。`release publish` 涉及 tag、push、GitHub Release，每一步失败都要回滚。status 类命令作为第一步实现压力小很多。

### 三分法的解耦价值

平面枚举把所有状态混在一起，下游代码只能 switch-case 逐个处理。三分法（Synchronized / OutOfSync / Anomaly）天然分出"可自动修"和"需人看"两个路径，严重程度分级也让输出更友好。`HealthIssue` 诊断报告和 sync 的安全守卫共用同一套模型。

### CI 脚本转 Rust 的价值有限

简单的 grep/sed 脚本（`validate-changelog.sh`、`validate-version.sh`）用 Rust 包装一遍并没有带来好处，反而增加了编译时间。preflight 的 `cargo check` / `cargo test` 也是直接调外部命令，Rust 层只是糊了一层 stdout 解析。脚本就让它保持脚本更合理。

### normalize_version 的坑

scope 前缀和 `v` 前缀的剥离顺序很重要。`"cli/v0.1.0"` 要先 split `/` 取最后一段 `"v0.1.0"`，再 strip `v` 得 `"0.1.0"`。反过来做会得到 `"cli/0.1.0"`。

### test 输出解析的坑

`cargo test` 输出 `"test result: ok. 10 passed; 0 failed; 2 ignored; 0 measured"`。按 `;` 分割后第一段包含 `"test result: ok."`，不能用简单的 `strip_suffix(" passed")`。正确的做法是用 `split_whitespace()` 取倒数第二个 word 作为数值。

## 依赖

- `qtcloud-devops-cli` — crates.io 依赖，使用其 release API
- `serde` + `serde_yaml` — 解析 contract.yaml
- `tempfile` — dev-dependency，测试用
