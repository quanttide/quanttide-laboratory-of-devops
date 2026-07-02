# Lab 模块实现全景

实验室在 `src/` 下实现了 5 个模块，覆盖 DevOps 四维契约模型的核心能力。本文是对所有模块的完整介绍。

## 目录

- [contract.rs — 四维契约模型](#1-contractrs--四维契约模型)
- [build.rs — 构建状态](#2-buildrs--构建状态)
- [code.rs — 代码状态（子模块三分法）](#3-coders--代码状态子模块三分法)
- [test.rs — 测试状态与覆盖率](#4-testrs--测试状态与覆盖率)
- [preflight.rs — 发布前检查](#5-preflightrs--发布前检查)

---

## 1. `contract.rs` — 四维契约模型

### 定位

按照四维架构（Stages / Platforms / Sources / Scopes）设计，从 `.quanttide/devops/contract.yaml` 加载完整契约。

理论来源：`docs/essay/contract/index.md`

### 核心数据结构

```rust
Contract { stages, platforms, sources, scopes: Vec<Scope> }
```

四个维度各对应一套配置：

| 维度 | Rust 类型 | 默认值 |
|---|---|---|
| **Stages** | `Stages { build, test { threshold: 70.0 }, release { changelog, pre_publish } }` | 全局默认，scope 可覆盖 |
| **Platforms** | `Platforms { source_control, ci, artifact_registry }` | github + github_actions + None |
| **Sources** | `Sources { version { source_type: Auto, path } }` | 自动检测 |
| **Scopes** | `Vec<Scope> { name, dir, language, build_tool, registry, release, test_threshold }` | 空列表 |

### 辅助枚举

```rust
Language      → Rust | Python | Go | Dart | TypeScript | Unknown(String)
BuildTool     → Cargo | Uv | Go | Flutter | Npm | Unknown(String)
Registry      → Crates | PyPI | PubDev | Npm | GitHubReleases | Docker | None
SourceType    → Cargo | Python | Go | Dart | Node | Auto
```

**兜底策略**：所有 Enum 都有 `Unknown` 或 `None` 变体。`language: zig` 不会让解析崩溃，而是存为 `Language::Unknown("zig")`，后续功能不对它生效。

### 加载逻辑

```rust
pub fn load(repo_path: &Path) -> Contract
```

读取 `.quanttide/devops/contract.yaml`：

1. 文件不存在 → 返回 `default_contract()`（四个维度全默认值，scopes 为空）
2. 存在 → 先尝试按新格式（四维架构）解析
3. 新格式失败 → 兼容旧格式 `scopes: { cli: src/cli }`
4. 都失败 → 返回 `default_contract()`

### 版本一致性检查

```rust
pub fn version_status(repo_path: &Path, scope: &Scope) -> VersionStatus
```

- **tag 版本**：`git tag --sort=-version:refname` 获取最新 tag，按 scope 前缀过滤（如 `cli/`），去 `v` 前缀和 scope 前缀后返回纯版本号
- **配置版本**：按语言读取 `Cargo.toml` / `pyproject.toml` / `package.json` / `pubspec.yaml`
- **一致性**：两者都存在时比较是否相等；都为空视为一致；一个为空一个有时视为不一致

### 便捷函数

```rust
// scope 级覆盖 → 全局默认
pub fn scope_release(contract, scope) → &StageRelease
pub fn scope_test_threshold(contract, scope) → f64

// 语言检测
pub fn resolve_language(scope, scope_dir) → Language
pub fn detect_by_files(dir) → Language

// 向下兼容
pub fn load_scopes(repo_path) → Vec<Scope>
pub fn detect_language(dir) → Language
```

---

## 2. `build.rs` — 构建状态

### 定位

按 scope 检查构建状态，只读模式，不触发构建。对应 roadmap `build-command.md` 蓝图。

### 实现：`status()`

```rust
pub fn status(repo_path: &Path)
```

无返回值，直接打印格式化输出。

#### 流程

1. 加载契约：`contract::load(repo_path)`
2. 无 scope：构造 root Scope，走 `contract::version_status()` + `contract::scope_release()`
3. 有 scope：遍历 scopes，调同一套接口

#### 每 scope 三路检查

| 检查项 | 实现 | 说明 |
|--------|------|------|
| CI 状态 | `gh --version` 检测 | 不真正调 API，仅检查 CLI 是否可用 |
| 语法校验 | `cargo check` | 按 scope 目录找 `Cargo.toml`，目前只处理 Rust |
| 版本一致 | `contract::version_status()` | 复用契约模块的逻辑，不做二次实现 |

#### 输出示例

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

### 取舍

- **不触发构建**：CI 自动执行，不替代 `cargo build` 或 GitHub Actions
- **不调 CI API**：只检测 `gh` 是否存在。真实场景需调 `gh run list`
- **root scope 临时构造**：无 `contract.yaml` 时构造一个 `name: "(root)"` 的 Scope，全走同一套接口——避免 if-else 分叉

---

## 3. `code.rs` — 代码状态（子模块三分法）

### 定位

子模块健康状态诊断模型。对应 CLI 中 `code status` 命令的数据模型。

### 核心模型：三分法

```
SubmoduleStatus
  ├── Synchronized        → 已同步，无需操作
  ├── OutOfSync(kind)     → 可自动修复
  │    ├── AheadOfRemote  → git push (Info)
  │    ├── BehindRemote   → git pull (Info)
  │    ├── Dirty          → 先手动提交/暂存 (Warning)
  │    └── Diverged       → 手动 merge 或 rebase (Error)
  └── Anomaly(kind)       → 需人工介入
       ├── DetachedHead   → git switch main (Error)
       ├── Orphaned       → 远程不存在该 commit，手动检查 (Error)
       ├── Missing        → git submodule update --init (Error, 可自动修复)
       ├── Unregistered   → 手动清理或重新注册 (Warning)
       └── Unknown(msg)   → 未知错误 (Error)
```

### 核心设计

#### 严重程度决定行为

```rust
pub enum Severity { Info, Warning, Error }
```

| 严重程度 | 自动修复 | 人工介入 | 示例 |
|---------|---------|---------|------|
| Info | ✅ `code sync` 自动处理 | 不需要 | AheadOfRemote 自动推送 |
| Warning | ❌ 需先手动提交 | 需要 | Dirty 要先 commit 或 stash |
| Error | ⚠ 部分可（Missing） | 需要 | Diverged 需要手动 merge |

#### 对偶性设计

- **OutOfSync**：**同步偏差**。知道应该做什么（push / pull / commit），通常可自动修复。
- **Anomaly**：**结构异常**。不确定当前状态，需要人类判断。

### 检测函数

```rust
pub fn scan_submodules(repo_path: &Path) -> Vec<HealthIssue>
```

1. 读 `.gitmodules` 解析子模块列表
2. 对每个子模块：检查目录存在 → 检查 `.git` → 检查工作区干净 → 检查 ahead/behind
3. 按严重程度排序返回（Error 优先）
4. 无 `.gitmodules` 时返回空列表，不做额外扫描

---

## 4. `test.rs` — 测试状态与覆盖率

### 定位

按 scope 输出测试结果和覆盖率，检查是否达到阈值。对应 roadmap `test-command.md` 蓝图。

### 实现：`status()`

```rust
pub fn status(repo_path: &Path, c: &contract::Contract)
```

#### 流程

1. 从 `contract::load_scopes()` 获取 scope 列表
2. 无 scope：检测语言，汇总测试，用全局 `c.stages.test.threshold` 作为覆盖率阈值
3. 有 scope：遍历 scopes，用 `contract::scope_test_threshold(c, &scope)` 获取各 scope 的阈值

### 阈值优先级

```
scope.test_threshold? → Some → 使用 scope 级 (如 90%)
                     → None → 使用 stages.test.threshold (全局默认 70%)
```

### 测试结果解析

```rust
fn parse_test_summary(content: &str) -> TestSummary
```

解析 `cargo test` 的输出行：`"test result: ok. 10 passed; 0 failed; 2 ignored; 0 measured"`。

按 `;` 分割后取每个片段末尾的 `(数字, kind)` 对，不依赖固定位置——容错性好，测试框架输出微调也不崩。

### 覆盖率解析

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

覆盖率 = 命中行数 / 总行数（简单按行统计，不按分支或函数）。

### 输出示例

```
测试状态
────────────────────────────────────────────────
  (root)        Rust
    测试数:       42 ✅ 全部通过
    覆盖率:       85.3%✅（阈值 70%）
```

---

## 5. `preflight.rs` — 发布前检查

### 定位

在发布前依次执行检查，对应 `scripts/preflight.sh`。确保版本、构建、测试、dry-run 全部通过再发版。

### 实现：`preflight()`

```rust
pub fn preflight(repo_path: &Path, _contract: &contract::Contract) -> PreflightResult
```

#### 四步检查

| 步骤 | 实现 | 失败是否阻断 |
|------|------|-------------|
| 版本检查 | `contract::version_status()` 遍历 scopes | ❌ 只警告，不阻断。tag 落后配置文件是常见开发状态 |
| 构建 | `cargo check` | ✅ 阻断。编译不过不能发布 |
| 测试 | `cargo test` | ✅ 阻断。测试不过不能发布 |
| dry-run | `cargo metadata --no-deps` | ✅ 阻断。元数据错误意味着包配置有问题 |

### Output

```
preflight
  (root): 0.1.0 ✅ tag:0.1.0 = 配置:0.1.0

--- cargo build ---  ✅
--- cargo test ---  42 passed; 0 failed; 0 ignored
--- cargo metadata --no-deps ---  ✅（metadata 检查通过）

preflight passed
```

### 关键决策

- **版本不一致不阻断**：因为开发阶段 tag 落后配置文件是常态。真正阻断的是构建/测试失败。
- **dry-run 简化为 metadata 检查**：真正发布时才需要 `cargo publish --dry-run`（涉及网络），preflight 只检查元数据能否正常解析。
- **无 `contract.yaml` 也正常工作**：构造一个 root scope 遍历，测试时传入默认 Contract。

### 返回值

```rust
pub struct PreflightResult {
    pub build_ok: bool,
    pub test_ok: bool,
    pub dry_run_ok: bool,
    pub version: String,
}
```

方便 CI 或脚本调用方判断——所有步骤通过才能发版。

---

## 模块依赖关系

```
main.rs
  │
  ├── contract::load()      ← 所有模块的配置来源
  │
  ├── build::status(c)      ← 用 c.scopes, contract::version_status(), scope_release()
  ├── code::scan_submodules ← 独立于 contract（纯 git 操作）
  ├── test::status(c)       ← 用 contract::scope_test_threshold()
  └── preflight::preflight(c) ← 用 contract::version_status()
```

`code.rs` 是唯一不依赖契约模块的——子模块状态检测是纯 git 操作，与项目配置无关。
