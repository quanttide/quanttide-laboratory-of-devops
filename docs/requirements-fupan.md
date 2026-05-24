# 需求复盘

基于本对话经验，梳理 `release status`、`plan`、`build`、`test` 四个命令的原始需求来源。

## release status

### 需求来源

Iter 1 开发过程中，每次执行 `stage`/`publish` 后没有直观方式查看当前发布状态。journal 文件是原始 JSONL，不可读。

具体场景：

```
# 调试 bug 时需要查 journal
cat .quanttide/devops/release-journal.jsonl  # JSONL 不可读

# 查看当前版本状态
ls .quanttide/devops/                         # 无法快速了解

# 多个 rc 版本后，哪些已发布、哪些已取消
# 只能手动 grep
```

### 核心要求

1. 从 `release-journal.jsonl` 读取并格式化输出
2. 显示每个版本的当前状态（Staged / Published / Cancelled / Retired）
3. 支持 `--json` 供程序消费
4. 每次操作前后执行，形成 diff

### 与本对话的关联

- rc.3 失败（tag 指向未提交代码）后，如果有 `release status` 就能提前发现 journal 中版本号与 Cargo.toml 不一致
- rc.4 失败（缺少 CHANGELOG 条目），`release status` 可以显示当前 journal 中记录的版本列表，帮助检查

---

## plan

### 需求来源

本对话中管理了多个规划文件：ROADMAP.md、TODO.md、CHANGELOG.md、BUGS.md、AGENTS.md。没有工具汇总这些信息。

具体场景：

```
# 开发前需要了解：当前迭代做什么？有哪些已知 bug？
# 需要分别打开 ROADMAP.md → TODO.md → BUGS.md → CHANGELOG.md
# 读完已经忘了前面看过什么

# 发布前需要检查：CHANGELOG 是否更新？BUGS 是否有关键问题？
# 同样需要手动翻多个文件
```

### 核心要求

1. 扫描 BUGS.md / ROADMAP.md / TODO.md 等项目管理文件
2. 输出汇总摘要（BUGS 数量、迭代进度、TODO 完成率）
3. 只读，不修改任何文件
4. 支持 `--json`

### 与本对话的关联

- 8 个 rc 版本中，多个问题是"忘记更新 CHANGELOG"、"忘记更新版本号"类型的遗漏。`plan` 可以在发布前做一次完整性检查
- AGENTS.md 中的"发布纪律"可以通过 `plan` 的预检查来自动化一部分

---

## build

### 需求来源

Iter 1 的 Rust 构建和 CI 构建是两套流程。本地测试时需要手动执行 `cargo build`、`cargo test`；CI 用 `maturin build`、`cargo build --release --target xxx`。没有统一的本地构建入口。

具体场景：

```
# 本地只想构建，不想记住具体命令
# 本地构建 vs CI 构建的参数不同
# 构建完成后需要手动查看产物路径
```

### 核心要求

1. 统一本地构建入口，自动检测项目类型选择构建方式
2. 输出构建结果摘要（成功/失败、产物路径、耗时）
3. 支持 `--release`
4. 与 CI 构建共享配置

### 与本对话的关联

- rc.6 的 maturin 构建问题（pyproject.toml 路径错误）如果在本地用 `build` 命令验证就能提前发现
- `scripts/preflight.sh` 是 `build` 的雏形——它顺序执行构建验证。`build` 命令可以替代 preflight 脚本

---

## test

### 需求来源

当前测试分散：`cargo test`（单元 + 集成）、`tests/code.rs`、`tests/release.rs`。没有统一的测试结果展示。

具体场景：

```
# 运行后输出原始，需要自己数通过/失败
# 失败用例需要自己找文件和行号
# 测试分散在多个目标中，需要分别执行
```

### 核心要求

1. 统一测试入口
2. 输出结构化结果（总数 / 通过 / 失败 / 跳过）
3. 列出失败用例（文件 + 行号 + 错误信息）
4. 支持 `--name <pattern>` 过滤
5. 支持 `--json`

### 与本对话的关联

- CI 中 16 个 code 测试失败，但输出是原始 `cargo test` 日志，需要手动翻找失败原因。`test` 命令可以聚合输出、高亮失败

---

## 共同原则

1. **Rust 实现** — 与现有 `stage`/`publish`/`cancel`/`retire` 一致
2. **原子操作** — 每个命令只做一件事
3. **无副作用** — `release status`、`plan`、`test` 只读；`build` 只执行构建
4. **`--json` 支持** — 所有命令的输出应支持 JSON 格式，供工具和 AI 消费
5. **统一 CLI 风格** — 参数命名、缩写、输出格式与现有命令一致
