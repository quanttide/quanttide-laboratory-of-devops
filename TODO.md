# TODO — 软件发布生命周期管理

## Iter 1：状态机核心命令 ✓

### 1. 新建 `src/model/release.rs`

- [x] `ReleaseStatus` 枚举：`Staged`, `Published`, `Cancelled`, `Retired`
- [x] `ReleaseAttempt` 结构体：id (UUID), version, status, created_at, updated_at, reason
- [x] `TransitionError` 枚举：非法状态转换校验
- [x] `validate_transition(from: &ReleaseStatus, to: &ReleaseStatus) -> Result<()>` 函数
- [x] 单元测试：所有合法/非法转换路径
- [x] 单元测试：状态 `Debug + Clone + PartialEq`

### 2. 状态持久化

- [x] `Storage` trait：`save(attempt)`, `load(version) -> Option<ReleaseAttempt>`, `list() -> Vec<ReleaseAttempt>`
- [x] `FileStorage` 实现：JSON 文件存储，路径 `.qtcloud/releases.json`
- [x] 事件溯源：每次转换追加记录到 `.qtcloud/release-events.jsonl`
- [x] 单元测试：持久化读写 + 事件追加

### 3. `stage <version>` 命令

- [x] `src/commands/stage.rs` — `run(version, reason) -> Result<String>`
- [x] 前置条件：版本未 `Published`（`Published` 拒绝）
- [x] 已 `Cancelled` 时复用现有记录（不产生新尝试 ID）
- [x] 已 `Staged` 时视为刷新部署（幂等）
- [x] 生成发布尝试 ID（UUID v4）
- [x] 调用 `Storage::save()` 持久化
- [x] CLI 注册：`qtcloud-devops-code stage <version> [--reason <reason>]`

### 4. `publish <version>` 命令

- [x] `src/commands/publish.rs` — `run(version) -> Result<String>`
- [x] 前置条件：版本必须 `Staged`
- [x] 执行 GitHub Release（复用现有 `create_release` + `create_tag`）
- [x] 状态变更为 `Published`，持久化
- [x] 不可逆：成功后不可 `cancel` 或退回
- [x] CLI 注册：`qtcloud-devops-code publish <version>`

### 5. `cancel <version>` 命令

- [x] `src/commands/cancel.rs` — `run(version) -> Result<String>`
- [x] 前置条件：版本必须 `Staged`
- [x] 回滚行为：删除远程 tag（`rollback_tag`） + 删除 GitHub Release（`gh release delete`）
- [x] 状态变更为 `Cancelled`，持久化
- [x] CLI 注册：`qtcloud-devops-code cancel <version> --reason <reason>`

### 6. `retire <version>` 命令

- [x] `src/commands/retire.rs` — `run(version) -> Result<String>`
- [x] 前置条件：版本必须 `Published`
- [x] 仅标记状态，不删除制品
- [x] 状态变更为 `Retired`，持久化
- [x] 不可逆：退役后只能通过 hotfix 发布新版本
- [x] CLI 注册：`qtcloud-devops-code retire <version> --reason <reason>`

### 7. 修改 `src/main.rs`

- [x] 删除 `Commands::Release`（被 `stage+publish+cancel+retire` 替代）
- [x] 注册 `Commands::Stage`, `Commands::Publish`, `Commands::Cancel`, `Commands::Retire`
- [x] 错误处理：状态转换失败时输出清晰错误信息

### 8. 修改 `src/lib.rs`

- [x] `pub mod model;`

### 9. 集成测试

- [x] `test_stage_publish_flow` — stage → publish 完整流程（publish 测试）
- [x] `test_stage_cancel_flow` — stage → cancel 流程（cancel 测试）
- [x] `test_publish_from_non_staged_rejected` — 非 Staged 状态 publish 拒绝
- [x] `test_cancel_from_non_staged_rejected` — 非 Staged 状态 cancel 拒绝
- [x] `test_retire_from_published` — Published → Retired
- [x] `test_retire_from_non_published_rejected` — 非 Published 状态 retire 拒绝
- [x] `test_stage_already_published_rejected` — 已 Published 版本重新 stage 拒绝
- [x] `test_stage_idempotent` — Staged 重复 stage 视为刷新（不产生新 ID）
- [x] `test_cancelled_can_restage` — Cancelled 后可重新 stage

### 10. 编译验证

- [x] `cargo build` 通过
- [x] `cargo test` 全部通过（61 tests）
- [x] `cargo clippy -- -D warnings` 通过

## P1 — 增强

- [ ] 审计日志彩色输出（`--verbose`）
- [ ] `list` 命令：列出所有发布记录及其状态
- [ ] `status <version>` 命令：查询单个版本状态
- [ ] `--dry-run` 支持所有命令
- [ ] `--json` 输出格式
- [ ] GitHub Release notes 从 `stage` 时注册的 changelog 自动生成

## P2 — 灰度与编排

- [ ] `stage --ratio <0.0-1.0>` 灰度比例参数
- [ ] Hotfix 编排脚本
- [ ] CI 集成插件（GitHub Action）
