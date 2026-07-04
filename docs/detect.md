# detect — 版本号自动检测

读取最新 tag → 扫描提交 → 按 devops-release 规则推断版本号。

```bash
cargo run --bin detect -- <repo-path>
```

## 规则

- 有 `feat:` → minor，走预发布（`-rc.1`）
- 仅 `fix: / refactor: / test:` → patch，直发正式
- 已在预发布系列 → 同阶段递增序号
- 仅 chore/docs → 不发版
- breaking change → 拒绝，交给人类

## 实现

核心逻辑在 `detect_version()`：

1. `detect_scope()` — 从 contract.yaml + changed files 推断 scope
2. `collect_tags_with_scope()` — 按 scope 分组，semver 排序
3. 扫描 tag→HEAD 提交，分类 feat/fix/chore
4. 输出 `scope/vX.Y.Z` 或 `scope/vX.Y.Z-rc.N`
