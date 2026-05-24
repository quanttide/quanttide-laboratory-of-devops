# ROADMAP — 软件发布生命周期管理

参考 `docs/roadmap/specification/release.md` 定义的状态机与命令契约，在 examples/default 中实现符合规范的发布生命周期管理 CLI。

## Iter 0：CLI 脚手架

| 交付 | 状态 |
|------|------|
| 移除已迁移的 `code` 子模块代码 | ✓ |
| `release` 单步命令（tag + GitHub Release） | ✓ |
| clap 参数骨架 | ✓ |

## Iter 1：状态机核心命令

### 状态定义

```
[*] → Staged : stage
Staged → Published : publish
Staged → Cancelled : cancel
Cancelled → Staged : stage (复用已有制品)
Published → Retired : retire
Retired → [*]
```

状态枚举：`Staged`, `Published`, `Cancelled`, `Retired`

### 原子命令

| 命令 | 转换 | 前置条件 |
|------|------|----------|
| `stage <version>` | → Staged | 未 Published；可重复执行（刷新部署） |
| `publish <version>` | Staged → Published | 必须 Staged；通过审批门禁 |
| `cancel <version>` | Staged → Cancelled | 必须 Staged |
| `retire <version>` | Published → Retired | 必须 Published |

### 审计约束

- 不存在 `delete` 命令
- 每次状态转换记录：操作人、时间戳、版本号、发布尝试 ID、旧状态、新状态、操作原因
- 事件溯源，不可变日志

### 实现方案

| 文件 | 操作 | 内容 |
|------|------|------|
| `src/model/release.rs` | **新增** | `ReleaseStatus` 枚举、`ReleaseAttempt` 结构体、状态转换校验 |
| `src/commands/mod.rs` | 修改 | 新增 `stage`、`publish`、`cancel`、`retire` 模块声明 |
| `src/commands/stage.rs` | **新增** | `stage` 原子命令（预发布/灰度部署） |
| `src/commands/publish.rs` | **新增** | `publish` 原子命令（正式上线，GitHub Release） |
| `src/commands/cancel.rs` | **新增** | `cancel` 原子命令（取消发布，环境回滚） |
| `src/commands/retire.rs` | **新增** | `retire` 原子命令（标记退役，停止服务） |
| `src/main.rs` | 修改 | 注册新子命令 |
| `src/release.rs` | 删除（单步 release 被替代） | |

### 基本假设

- 遵循 Semantic Versioning 2.0.0
- 发布尝试 ID 由 `stage` 生成，为 UUID
- 状态持久化存储（SQLite 或文件）
- 角色分离建议按 spec 第 7 节执行
- `cancel` 回滚行为由实现层保证
- `Published` 是单向门，不可退回
