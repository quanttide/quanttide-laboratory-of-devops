# release status

查看当前项目的发布状态。

## 用法

```bash
qtcloud-devops-code release-status
```

## 输出

```
发布状态报告
----------------------------------------
待发布: 2
  v0.5.0-rc.1 (尝试: 57434c33)
  v1.0.0 (尝试: 0be25d2b)
已发布: 0

最新发布:
  v0.5.0-rc.1               Staged       1779621403
  v1.0.0                    Staged       1779621403
```

### 各字段含义

| 字段 | 说明 |
|------|------|
| 待发布 | Staged 状态的版本列表 |
| 已发布 | Published 状态的版本列表 |
| 最新发布 | 按时间倒序排列的最新 5 条记录 |

## 无发布记录时

```
当前无发布记录
```

## 内部行为

1. 打开当前目录下的 `.quanttide/devops/release-journal.jsonl`（如果存在）
2. 逐行读取，每行解析为一个 `ReleaseEntry`（JSON 格式）
3. 按 version 去重投影为 `ReleaseRecord`（后出现的 entry 覆盖先出现的，`created_at` 取第一个 entry）
4. 按 `updated_at` 倒序排列
5. 统计 Staged / Published / Cancelled / Retired 各状态的数量
