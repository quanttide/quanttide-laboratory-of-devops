# release status 需求复盘

## 需求来源

Iter 1 开发过程中，每次执行 `stage`/`publish` 后没有直观方式查看当前发布状态。journal 文件是原始 JSONL，不可读。

## 场景

```
# 调试 bug 时需要查 journal
cat .quanttide/devops/release-journal.jsonl  # JSONL 不可读

# 查看当前版本状态
ls .quanttide/devops/                         # 无法快速了解

# 多个 rc 版本后，哪些已发布、哪些已取消
# 只能手动 grep
```

## 核心要求

1. 从 `release-journal.jsonl` 读取并格式化输出
2. 显示每个版本的当前状态（Staged / Published / Cancelled / Retired）
3. 每次操作前后执行，形成 diff

## 对话中的关联

- rc.3 失败（tag 指向未提交代码）后，如果有 `release status` 就能提前发现 journal 中版本号与 Cargo.toml 不一致
- rc.4 失败（缺少 CHANGELOG 条目），`release status` 可以显示当前 journal 中记录的版本列表，帮助检查
