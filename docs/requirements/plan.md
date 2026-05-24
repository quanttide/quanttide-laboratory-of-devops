# plan 需求复盘

## 需求来源

本对话中管理了多个规划文件：ROADMAP.md、TODO.md、CHANGELOG.md、BUGS.md、AGENTS.md。没有工具汇总这些信息。

## 场景

```
# 开发前需要了解：当前迭代做什么？有哪些已知 bug？
# 需要分别打开 ROADMAP.md → TODO.md → BUGS.md → CHANGELOG.md
# 读完已经忘了前面看过什么

# 发布前需要检查：CHANGELOG 是否更新？BUGS 是否有关键问题？
# 同样需要手动翻多个文件
```

## 核心要求

1. 扫描 BUGS.md / ROADMAP.md / TODO.md 等项目管理文件
2. 输出汇总摘要（BUGS 数量、迭代进度、TODO 完成率）
3. 只读，不修改任何文件
4. 支持 `--json`

## 对话中的关联

- 8 个 rc 版本中，多个问题是"忘记更新 CHANGELOG"、"忘记更新版本号"类型的遗漏。`plan` 可以在发布前做一次完整性检查
- AGENTS.md 中的"发布纪律"可以通过 `plan` 的预检查来自动化一部分
