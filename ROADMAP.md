# ROADMAP — 软件发布生命周期管理

examples/default 实现量潮发布规范的参考示例。当前版本基于状态机的发布生命周期管理 CLI。

## 已完成

### Iter 0：CLI 脚手架

- `release` 单步命令（已废弃）
- clap 参数骨架

### Iter 1：状态机核心命令

**状态定义**

```
[*] → Staged : stage
Staged → Published : publish
Staged → Cancelled : cancel
Cancelled → Staged : stage
Published → Retired : retire
Retired → [*]
```

**交付物**

- `src/model/release.rs` — ReleaseStatus / ReleaseRecord / ReleaseEntry / Storage / FileStorage
- `src/commands/stage.rs` — stage 命令
- `src/commands/publish.rs` — publish 命令
- `src/commands/cancel.rs` — cancel 命令
- `src/commands/retire.rs` — retire 命令
- `src/commands/release.rs` — 工具函数（validate_version, create_tag, extract_notes 等）
- `docs/user-guide.md` — 用户文档
- `docs/roadmap/specification/release.md` — 建模报告
- 事件溯源：`.quanttide/devops/release-journal.jsonl`
- 67 测试，全部通过

## 待规划

### P1 — 体验增强

- [ ] `list` 命令：列出所有发布记录及其状态
- [ ] `status <version>` 命令：查询单个版本状态
- [ ] `--dry-run` 支持所有命令
- [ ] `--json` 输出格式

### P2 — 灰度与编排

- [ ] `stage --ratio <0.0-1.0>` 灰度比例参数
- [ ] Hotfix 编排脚本
- [ ] CI 集成插件（GitHub Action）
