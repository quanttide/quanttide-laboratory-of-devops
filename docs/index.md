# qtcloud-devops-code 用户文档

`qtcloud-devops-code` 是量潮发布规范的参考实现。提供发布管理、开发辅助等命令。

## 安装

```bash
cd examples/default
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

## 命令一览

### 发布管理

| 命令 | 说明 |
|------|------|
| `stage -v <version>` | 标记版本 → Staged |
| `publish -v <version> [-y]` | 发布上线（标签 + GitHub Release） |
| `cancel -v <version>` | 取消发布 |
| `retire -v <version>` | 退役版本（终态） |
| [release-status](release-status.md) | 查看发布状态 |

### 开发辅助

| 命令 | 说明 |
|------|------|
| [plan](plan.md) | 扫描项目管理文件，生成规划摘要 |
| [build](build.md) | 执行项目构建 |
| [test](test.md) | 执行测试 |

## 状态机

```
stage  → Staged
publish → Published
cancel  → Cancelled
retire  → Retired（终态）
```

所有状态变更追加记录到 `.quanttide/devops/release-journal.jsonl`。
