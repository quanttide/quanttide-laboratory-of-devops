# 量潮 DevOps 实验室

当前实验模块：

| 模块 | 说明 | 报告 | 状态 |
|------|------|------|------|
| `bin/detect` | 版本号自动检测 — 从 git 历史推断 scope/minor/patch/预发布 | [docs/detect.md](docs/detect.md) | 原型 |
| `git-exp` | git2 vs gix API 与性能对比 | [报告](../../data/report/lab/git-exp.md) | 实验 |

已推进到平台（实验室不再维护副本）：

```bash
# 统一状态查看（替代 preflight）
qtcloud-devops status

# 发布流程（替代 release）
qtcloud-devops release publish -v <version> -y
```
