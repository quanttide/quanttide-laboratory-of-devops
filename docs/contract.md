# 契约设计记录

## 核心数据结构

```rust
Contract { stages, platforms, sources, scopes: Vec<Scope> }
```

四个维度各对应一套配置：

| 维度 | Rust 类型 | 默认值 |
|------|----------|--------|
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

检查是否被支持：

```rust
if !language.is_supported() {
    eprintln!("⚠ scope {}: 语言 {:?} 未知", name, language);
}
```

## 加载逻辑

```rust
pub fn load(repo_path: &Path) -> Contract
```

读取 `.quanttide/devops/contract.yaml`：

1. 文件不存在 → 返回 `default_contract()`（四个维度全默认值，scopes 为空）
2. 存在 → 先尝试按新格式（四维架构）解析
3. 新格式失败 → 兼容旧格式 `scopes: { cli: src/cli }`
4. 都失败 → 返回 `default_contract()`

## 版本一致性检查

```rust
pub fn version_status(repo_path: &Path, scope: &Scope) -> VersionStatus { tag_version, config_version, consistent }
```

- **tag 版本**：`git tag --sort=-version:refname` 获取最新 tag，按 scope 前缀过滤（如 `cli/`），去 `v` 前缀和 scope 前缀后返回纯版本号
- **配置版本**：按语言读取 `Cargo.toml` / `pyproject.toml` / `package.json` / `pubspec.yaml`
- **一致性**：两者都存在时比较是否相等；都为空视为一致；一个为空一个有时视为不一致

## 覆盖语义（浅覆盖）

Scope 对全局配置的覆盖是**浅覆盖**：scope 级有值就用 scope 的，没有就用全局的。不是深度合并。

示例：
- `scope.test_threshold = Some(90)` → 使用 90（覆盖全局 70）
- `scope.test_threshold = None` → 使用全局 `stages.test.threshold`（70）
- `scope.release.changelog = "src/cli/CHANGELOG.md"` → 只覆盖 changelog，`pre_publish` 走全局

判断逻辑在 `scope_release()` 中：对比 scope 的 release 是否等于默认值。是则返回全局引用（零拷贝），否则返回 scope 的。

## 便捷函数

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

## 四维架构映射

理论（`docs/essay/contract/index.md`）→ 代码（`contract.yaml`）

| 维度 | `contract.yaml` | 说明 |
|------|----------------|------|
| **Stages**（时序） | `stages.build` / `stages.test` / `stages.release` | 生命周期各阶段的配置。不规定"怎么做"，只规定"什么时候检查什么"。 |
| **Platforms**（载体） | `platforms.source_control` / `platforms.ci` / `platforms.artifact_registry` | 外部治理载体。负责"外部合规"。 |
| **Sources**（事实源） | `sources.version.type` / `sources.version.path` | 真相的中心。版本号从哪读、格式是什么。 |
| **Scopes**（上下文） | `scopes.<name>` | 规则的边界。每个 scope 继承全局设置，只声明差异部分。 |

## 关键设计决策

### 1. 全局默认值 + scope 覆盖

不是每个 scope 重复声明全部字段。顶层 `stages` / `platforms` / `sources` 是全局默认值，scope 只写覆盖：

```yaml
stages:
  test:
    threshold: 70

scopes:
  cli:
    dir: src/cli
    language: rust
    test_threshold: 90   # 覆盖全局
```

这样才是"厚基层"——一次配置，所有 scope 自动继承。

### 2. 旧格式兼容

旧格式 `scopes: { cli: src/cli }` 不能断。先用 `serde_yaml` 尝试按新格式解析，失败则尝试旧格式，都失败返回默认值。两阶段解析。

### 3. 两阶段解析（YAML → 中间结构 → 业务模型）

```
YAML → ContractYaml（serde 直接反序列化，字段名匹配 YAML）
     → into_contract() → Contract（业务模型，语义清晰）
```

好处：YAML 字段名和 Rust 字段名不需要一致，`into_contract()` 集中做转换逻辑。

### 4. Enum 用 Unknown 变体兜底

`language: zig` 不应让解析失败。`Language::Unknown(String)` 兜底后，非标准语言通过解析，后续功能不对它生效。契约应该包容，在运行时而非解析时报错。

## 经验教训

- 先写理论（essay）再写代码（contract），映射关系自然形成。反过来如果先写代码再抽象理论，容易过度设计。
- `Scope` 的 `release` 覆盖粒度要够：scope 级可以改 `changelog` 路径（如 monorepo 中 `src/studio/CHANGELOG.md`），不改的走全局默认。
- 测试策略：所有解析逻辑通过构造 YAML 字符串验证 `Contract` 字段，不依赖真实文件系统（`tempfile`），不依赖 `contract.yaml` 真实路径。

## 实验室各模块适配状态

四维架构已全面接入：

| 模块 | 适配状态 | 说明 |
|------|---------|------|
| `build.rs` | ✅ | 使用 `contract::version_status()`、`contract::scope_release()` |
| `test.rs` | ✅ | 使用 `contract::scope_test_threshold()` 读取阈值 |
| `preflight.rs` | ✅ | 使用 `contract::version_status()` 检查版本一致性 |
| `validate.rs` | — | 已删除，功能已合并至 contract/preflight |
| `code.rs` | — | 独立模块，不依赖契约 |

便捷函数 `scope_release()`、`scope_test_threshold()` 均有调用方。

各模块文档见同目录下 `build.md`、`code.md`、`test.md`、`preflight.md`。
