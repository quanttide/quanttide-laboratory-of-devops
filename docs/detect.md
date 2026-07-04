# detect — 版本号自动检测

读取最新 tag → 扫描提交 → 按 devops-release 规则推断版本号。

```bash
cargo run --bin detect -- <repo-path>
```

## 规则

规则基于以下假设（来自 devops-release skill）：

- 有 `feat:` → **minor**，走预发布（`-rc.1`）
  - 假设：新功能需要验证期，不能直接上正式版
- 仅 `fix: / refactor: / test:` → **patch**，直发正式
  - 假设：修复已知问题风险低，已有信心，无需走预发布
- 已在预发布系列 → **同阶段递增序号**
  - 假设：一旦进入预发布周期，继续当前验证阶段，不跳跃
- 仅 chore/docs → **不发版**
  - 假设：非用户可见的逻辑改动不值得发版本号
- breaking change → **拒绝，交给人类**
  - 假设：major 版本决策需要人类判断，AI 不越权

## 实现

核心逻辑在 `detect_version()`：

1. `detect_scope()` — 从 contract.yaml + changed files 推断 scope
2. `collect_tags_with_scope()` — 按 scope 分组，semver 排序
3. 扫描 tag→HEAD 提交，分类 feat/fix/chore
4. 输出 `scope/vX.Y.Z` 或 `scope/vX.Y.Z-rc.N`

## 未覆盖的规则

当前原型做了简化，以下规则尚未实现：

| 规则 | SKILL.md 要求 | 现状 |
|------|-------------|------|
| 变更规模判断 | 重大变更（新命令/重构）→ minor，小 feat → patch | 所有 `feat:` 都算 minor，未区分规模 |
| 预发布起始阶段 | 可选 alpha / beta / rc，取决于变更规模 | 永远从 `-rc.1` 开始 |
| 阶段晋级 | alpha → beta → rc，阶段切换时序号重置 | 仅同阶段递增，不切换 |
| 紧急 bug 修复 | 从正式版发 `patch-rc.1` 验证后转正式 | 直接推断 patch 直发正式，无 rc 验证路径 |
| 多 scope 冲突 | 询问用户如何处理 | 直接报错退出，无交互 |
| 不确定时问用户 | 兜底原则 | 无交互能力 |
