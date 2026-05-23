# TODO

## Iteration 9：命令清理与核心价值聚焦

### 9.1 移除无新贡献的命令

**移除的 CLI 子命令**（共 8 个）：

| 命令 | 删除位置 |
|------|----------|
| `kse add` | main.rs Commands::Add, editor.rs add_submodule, trait |
| `kse init` | main.rs Commands::Init, editor.rs init_all, trait |
| `kse update` | main.rs Commands::Update, editor.rs update_single, trait |
| `kse update-all` | main.rs Commands::UpdateAll, editor.rs update_all, trait |
| `kse checkout` | main.rs Commands::Checkout, editor.rs checkout_branch, trait |
| `kse branch` | main.rs Commands::Branch, editor.rs create_branch, trait |
| `kse checkout-all` | main.rs Commands::CheckoutAll, editor.rs checkout_all, trait |
| `kse branch-all` | main.rs Commands::BranchAll, editor.rs branch_all, trait |

**移除的 Tauri commands**（共 5 个）：

| Tauri command | 删除位置 |
|---------------|----------|
| `init_all` | src-tauri/src/main.rs |
| `update_single` | src-tauri/src/main.rs |
| `update_all` | src-tauri/src/main.rs |
| `checkout_all` | src-tauri/src/main.rs |
| `branch_all` | src-tauri/src/main.rs |

**移除的 trait 方法**：

| 方法 | trait | impl |
|------|-------|------|
| `add_submodule()` | commands/mod.rs | editor.rs |
| `init_all()` | commands/mod.rs | editor.rs |
| `update_single()` | commands/mod.rs | editor.rs |
| `update_all()` | commands/mod.rs | editor.rs |
| `checkout_branch()` | commands/mod.rs | editor.rs |
| `checkout_all()` | commands/mod.rs | editor.rs |
| `create_branch()` | commands/mod.rs | editor.rs |
| `branch_all()` | commands/mod.rs | editor.rs |

**移除的测试**：

| 测试文件 | 移除内容 |
|----------|----------|
| tests/integration.rs | 所有涉及 add/init/checkout/branch 及批量变体的测试 |

### 9.2 `health-check` → `status` 重命名

- [ ] `commands/mod.rs`: `SubmoduleEditor` trait `health_check()` → `status()`
- [ ] `commands/editor.rs`: impl 实现 → `status()`, `health_check()` 改为 `status()` 的 proxy
- [ ] `src/main.rs`: `Commands::HealthCheck` → `Commands::Status`; 保留 `HealthCheck` 作为隐藏 alias 输出迁移提示
- [ ] `src-tauri/src/main.rs`: command `health_check` → `status`; 保留 `health_check` 作为别名
- [ ] `web-ui/src/app.js`: `invoke('health_check')` → `invoke('status')`
- [ ] `tests/integration.rs`: 更新 `health_check()` 调用 → `status()`

### 9.3 `sync` 命令族精简

保留的：

| 新命令 | 原身 |
|--------|------|
| `kse sync parent <name>` | `sync_to_parent`（保留） |
| `kse sync parent --all` | `sync_all_to_parent`（保留） |

新增的：

| 新命令 | 说明 |
|--------|------|
| `kse sync platform <name> --env <env>` | 跨环境版本对齐占位 |

- [ ] trait 新增 `sync_platform(name, env)` + 骨架实现
- [ ] CLI 注册 `Commands::Sync { subcommand: SyncParent | SyncPlatform }`
- [ ] `kse sync` 默认等价于 `kse sync parent`
- [ ] `kse sync-all` 保留为 alias → `kse sync parent --all`
- [ ] Tauri command 仅保留 `sync_to_parent`, `sync_all_to_parent`
- [ ] Web UI 移除无用按钮

### 9.4 测试

- [ ] 删除涉及已移除命令的集成测试
- [ ] 保留 `sync_to_parent`, `retire_submodule`, `history`, `scan` 测试
- [ ] 新增 `sync_platform` 骨架测试
- [ ] `cargo test && cargo clippy -- -D warnings` 通过

### 9.5 文档

- [ ] `docs/user-guide.md`: CLI 命令从上到下全部重写
- [ ] `docs/dev.md`: 更新 trait 接口定义
- [ ] `CHANGELOG.md`: 记录破坏性变更

---

## 待完成（需本地环境）

| 任务 | 命令 |
|------|------|
| Iter 9 编译验证 | `cargo build && cargo test && cargo clippy -- -D warnings` |
| CI 触发验证 | `git push origin main` |
| GitHub Release | `gh release create v1.0.0 ...` |
